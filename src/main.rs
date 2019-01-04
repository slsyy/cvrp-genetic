#[macro_use]
extern crate serde_derive;

extern crate clap;
use clap::App;
use clap::Arg;

extern crate nalgebra as na;

use std::cmp;
use std::error::Error;
use std::fs::File;
use std::mem;

extern crate rand;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::FromEntropy;
use rand::Rng;

mod data;
mod description;

extern crate serde;
extern crate serde_json;

extern crate indicatif;
use indicatif::ProgressBar;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Output {
    pub description: description::Description,
    pub best_path: Vec<Vec<description::NodeID>>,
    pub best_solution_performance: Vec<(usize, data::Cost)>,
}

fn main() {
    let (input_file_path, iteration_count) = parse_args();
    let desc = read_description(&input_file_path).unwrap();
    let data = data::Data::new(&desc);

    let population_size = ((0.6  * desc.nodes.len() as f64).powf(2.0)) as usize;
    let survivor_number = (population_size as f64).sqrt() as usize;

    let elitism = 0.05;
    let elite_survivors_number = (elitism * population_size as f64).ceil() as usize;

    let crossover_rate = 0.5;
    let mutation_rate = 0.1;

    let mut best_solution_performance = Vec::new();

    let mut best_so_far = (data::Cost::max_value(), Vec::<data::NodeIndex>::new());

    let mut population = initial_population(&data, population_size);

    let mut rng = SmallRng::from_entropy();

    let bar = ProgressBar::new(iteration_count as u64);
    for i in 0..iteration_count {
        bar.inc(1);
        let cost_population: Vec<(data::Cost, Vec<data::NodeIndex>)> = {
            let mut res: Vec<_> = population
                .into_iter()
                .map(|chromosome| (data.calculate_cost(&chromosome), chromosome))
                .collect();

            res.sort_unstable_by_key(|&(cost, _)| cost);
            res
        };

        {
            let best_in_iteration = cost_population
                .iter()
                .min_by_key(|&(cost, _)| cost)
                .unwrap();

            if best_in_iteration.0 < best_so_far.0 {
                best_so_far = best_in_iteration.to_owned();
                best_solution_performance.push((i, best_in_iteration.0));
            }
        }

        let mut fitness_population: Vec<_> = cost_population
            .into_iter()
            .map(|(cost, chromosome)| (1.0 / cost as f64, chromosome))
            .collect();

        let fitness_sum: f64 = fitness_population.iter().map(|(f, _)| f).sum();

        let mut survivors = Vec::with_capacity(population_size);

        for i in 0..elite_survivors_number {
            survivors.push(fitness_population[i].1.clone());
        }

        for _ in 0..survivor_number {
            let index = loop {
                let possible_index = {
                    let mut value = rng.gen_range(0.0, fitness_sum);
                    let mut i = 0;
                    for (fitness, _) in fitness_population.iter() {
                        value -= fitness;
                        if value < 0.0 {
                            break;
                        }
                        i += 1;
                    }

                    cmp::min(i, fitness_population.len() - 1)
                };

                if !fitness_population[possible_index].1.is_empty() {
                    break possible_index;
                }
            };

            let mut tmp = Vec::new();
            mem::swap(&mut tmp, &mut fitness_population[index].1);
            survivors.push(tmp);
        }

        population = Vec::with_capacity(population_size);

        for (i, parent_a) in survivors.iter().enumerate() { 
            let mut already_pushed_without_crossing = false;
            
            for (j, parent_b) in survivors.iter().enumerate() {
                if i == j {
                    continue;
                }

                if !already_pushed_without_crossing && !rng.gen_bool(crossover_rate) {
                    already_pushed_without_crossing = true;
                    population.push(parent_a.clone());
                    mutate(&mut population.last_mut().unwrap(), &mut rng, mutation_rate);
                    continue;
                }

                // PMX Crossover

                let mut offspring = parent_b.clone();

                let swath_left = rng.gen_range(0, parent_a.len());
                let swath_right = rng.gen_range(swath_left, parent_a.len());


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

                mutate(&mut offspring, &mut rng, mutation_rate);
                population.push(offspring);
            }
        }
    }
    bar.finish();

    let output = Output {
        description: desc,
        best_path: data.indices_path_to_index(&best_so_far.1),
        best_solution_performance: best_solution_performance,
    };

    println!("{}", serde_json::to_string(&output).unwrap());
}

fn mutate(chromosome: &mut Vec<data::NodeIndex>, rng: &mut impl rand::Rng, mutation_rate: f64) {
    for _ in 0..chromosome.len() / 2 {
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

fn parse_args() -> (String, usize) {
    let matches = App::new("cvrp-genetic")
        .arg(Arg::from_usage("<input> 'problem description input file'"))
        .arg(Arg::from_usage("<iteration_count> 'iteration count'"))
        .get_matches();

    (
        matches.value_of("input").unwrap().to_owned(),
        matches
            .value_of("iteration_count")
            .unwrap()
            .parse()
            .unwrap(),
    )
}

fn read_description(input_file_path: &str) -> Result<description::Description, Box<dyn Error>> {
    let input_file = File::open(input_file_path)?;
    Ok(serde_json::from_reader(input_file)?)
}