// use std::alloc::System;

// #[global_allocator]
// static A: System = System;

use serde_derive::{Deserialize, Serialize};
use serde_json;

use clap::App;
use clap::Arg;

use std::error::Error;
use std::fs::File;

mod data;
mod description;
mod genetic_algo;

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

    let genetic_algo::BestPath {
        best_path,
        best_solution_performance,
    } = genetic_algo::find_best_path(&data, iteration_count);

    let output = Output {
        description: desc,
        best_path: data.indices_path_to_index(&best_path),
        best_solution_performance: best_solution_performance,
    };

    println!("{}", serde_json::to_string(&output).unwrap());
}

fn parse_args() -> (String, genetic_algo::IterationNumber) {
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