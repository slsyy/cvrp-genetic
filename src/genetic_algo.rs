use crate::data;

use std::cmp;
use std::mem;

use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::FromEntropy;
use rand::Rng;

use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub struct BestPath {
    pub best_path: Vec<data::NodeIndex>,
    pub best_solution_performance: Vec<(usize, data::Cost)>,
}

pub type IterationNumber = usize;

pub fn find_best_path(data: &data::Data, iteration_number: IterationNumber) -> BestPath {
    let population_size = ((0.7 * data.index_to_id.len() as f64).powf(2.0)) as usize;
    let survivor_number = (population_size as f64).sqrt() as usize;
    let elite_survivor_number = 1;

    let crossover_rate = 0.7;
    let mutation_rate = 0.01;

    let mut best_solution_performance = Vec::new();

    let mut best_so_far = (data::Cost::max_value(), Vec::<data::NodeIndex>::new());

    let mut population = initial_population(&data, population_size);

    let mut rng = SmallRng::from_entropy();

    let bar = ProgressBar::new(iteration_number as u64);
    bar.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}]{pos:>7}/{len:7} [{msg}] ({eta})")
    .progress_chars("#>-"));

    for i in 0..iteration_number {
        bar.inc(1);

        let chromosome_cost: Vec<data::Cost> = population
            .par_iter()
            .map(|chromosome| data.calculate_cost(&chromosome))
            .collect();

        let best_in_iteration = chromosome_cost
            .iter()
            .enumerate()
            .min_by_key(|&(_, cost)| cost)
            .map(|(j, &cost)| (cost, &population[j]))
            .unwrap();

        if best_in_iteration.0 < best_so_far.0 {
            best_so_far = (best_in_iteration.0, best_in_iteration.1.to_owned());
            best_solution_performance.push((i, best_in_iteration.0));
            bar.set_message(&format!("Best result: {}", best_so_far.0));
        }

        let new_population = {
            let mut x: Vec<_> = population.iter().enumerate().map(|(i, ch)| (chromosome_cost[i], ch)).collect();
            pdqselect::select_by_key(&mut x, elite_survivor_number, |&(cost, _)| cost); 

            x.truncate(elite_survivor_number);
            let mut res = Vec::with_capacity(elite_survivor_number + survivor_number);
            for &(_, ch) in x.iter() {
                res.push(ch.clone());
            }

            res
        };

        let survivors =
            tournament_selection(survivor_number, &mut population, &chromosome_cost, &mut rng);


        population = new_population;

        // population.clear();

        for (i, parent_a) in survivors.iter().enumerate() {
            for (j, parent_b) in survivors.iter().enumerate() {
                if i == j {
                    continue;
                }

                let mut offspring = if rng.gen_bool(crossover_rate) {
                    pmx_crossover(&parent_a, &parent_b, &mut rng)
                } else {
                    parent_a.clone()
                };

                mutate(&mut offspring, &mut rng, mutation_rate);
                population.push(offspring);
            }
        }
    }
    bar.finish();

    BestPath {
        best_path: best_so_far.1,
        best_solution_performance: best_solution_performance,
    }
}

fn tournament_selection(
    survivor_number: usize,
    population: &mut Vec<Vec<data::NodeIndex>>,
    chrosome_cost: &Vec<data::Cost>,
    rng: &mut impl rand::Rng,
) -> Vec<Vec<data::NodeIndex>> {
    let mut survivors = Vec::with_capacity(survivor_number);

    let mut already_chosen: HashSet<u64> = HashSet::with_capacity(population.len());

    for _ in 0..survivor_number {
        let winner = loop {
            let x = rng.gen_range(0, population.len());
            let y = rng.gen_range(0, population.len());

            if population[x].is_empty() || population[y].is_empty() {
                continue;
            }

            let better_chromosome_index = if chrosome_cost[x] < chrosome_cost[y] {
                x
            } else {
                y
            };

            let hash = {
                let mut s = DefaultHasher::new();
                population[better_chromosome_index].hash(&mut s);
                s.finish()
            };
            if already_chosen.contains(&hash) {
                continue;
            }
            already_chosen.insert(hash);

            let mut winner = Vec::new();
            mem::swap(&mut winner, &mut population[better_chromosome_index]);
            break winner;
        };

        survivors.push(winner);
    }

    survivors
}

fn pmx_crossover(
    parent_a: &Vec<data::NodeIndex>,
    parent_b: &Vec<data::NodeIndex>,
    rng: &mut impl rand::Rng,
) -> Vec<data::NodeIndex> {
    let mut offspring = parent_b.clone();

    let swath_left = rng.gen_range(0, parent_a.len());
    let right_bound = cmp::min(parent_a.len(), swath_left + 5);
    // let right_bound = parent_a.len();
    let swath_right = rng.gen_range(swath_left, right_bound);

    let swath_range = swath_left..=swath_right;
    let a_swath = &parent_a[swath_range.clone()];
    let b_swath = &parent_b[swath_range.clone()];

    offspring[swath_range.clone()].clone_from_slice(a_swath);

    for (i, &allele) in b_swath.iter().enumerate() {
        if a_swath.contains(&allele) {
            continue;
        }

        let mut pos = swath_left + i;
        loop {
            let val_a = parent_a[pos];
            pos = parent_b.iter().position(|&val_b| val_a == val_b).unwrap();
            if pos < swath_left || pos > swath_right {
                break;
            }
        }

        offspring[pos] = allele;
    }

    offspring
}

fn mutate(chromosome: &mut Vec<data::NodeIndex>, rng: &mut impl rand::Rng, mutation_rate: f64) {
    for _ in 0..chromosome.len() {
        if !rng.gen_bool(mutation_rate) {
            continue;
        }

        let x = rng.gen_range(0, chromosome.len());
        let y = rng.gen_range(0, chromosome.len());

        chromosome.swap(x, y);
    }
}

fn initial_population(data: &data::Data, size: usize) -> Vec<Vec<data::NodeIndex>> {
    let mut rng = SmallRng::from_entropy();

    let mut population: Vec<Vec<data::NodeIndex>> = (0..size)
        .map(|_| {
            (0..data.index_to_id.len())
                .filter(|&i| i != data.depot)
                .collect()
        })
        .collect();

    for chromosome in population.iter_mut() {
        chromosome.shuffle(&mut rng);
    }

    population
}
