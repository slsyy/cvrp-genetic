#[macro_use]
extern crate serde_derive;

extern crate clap;
use clap::App;
use clap::Arg;

extern crate nalgebra as na;

use std::error::Error;
use std::fs::File;

extern crate rand;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::FromEntropy;
use rand::Rng;

mod data;
mod description;

extern crate serde;
extern crate serde_json;

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

    let population_size: usize = 100;
    let survivor_number = (population_size as f64).sqrt() as usize;
    assert_eq!(survivor_number.pow(2), population_size);

    let mut best_solution_performance = Vec::new();

    let mut best_so_far = (data::Cost::max_value(), Vec::<data::NodeIndex>::new());

    let mut offsprings = initial_population(&data, population_size);

    let mut rng = SmallRng::from_entropy();

    for i in 0..iteration_count {

        
        offsprings.sort_by_key(|p| data.calculate_cost(&p));

        let best_cost = data.calculate_cost(&offsprings[0]);
        if best_cost < best_so_far.0 {
            best_so_far = (best_cost, offsprings[0].to_owned());
            best_solution_performance.push((i, best_cost));
        }

        // let survivors: Vec<_> = offsprings.into_iter().take(survivor_number).collect();
        // offsprings = survivors.clone();

        for chromosome in offsprings.iter_mut() {
            let x = rng.gen_range(0, chromosome.len());
            let y = rng.gen_range(0, chromosome.len());
            chromosome.swap(x, y);
        }
    }

    let output = Output {
        description: desc,
        best_path: data.indices_path_to_index(&best_so_far.1),
        best_solution_performance: best_solution_performance,
    };

    println!("{}", serde_json::to_string(&output).unwrap());
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
