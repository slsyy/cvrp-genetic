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

mod data;
mod description;

extern crate serde;
extern crate serde_json;

fn main() {
    let input_file_path = parse_args();
    let desc = read_description(&input_file_path).unwrap();
    let data = data::Data::new(&desc);

    let mut rng = SmallRng::from_entropy();

    let mut population: Vec<usize> = (0..data.index_to_id.len()).collect();
    let mut cost = 100000;
    loop {
        population.shuffle(&mut rng);
        let new_cost = data.calculate_cost(&population);
        if new_cost < cost {
            cost = new_cost;
            println!("{:?}", &cost);
        }
    }
}

fn parse_args() -> String {
    let matches = App::new("cvrp-genetic")
        .arg(Arg::from_usage("<input> 'problem description input file'"))
        .get_matches();

    matches.value_of("input").unwrap().to_owned()
}

fn read_description(input_file_path: &str) -> Result<description::Description, Box<dyn Error>> {
    let input_file = File::open(input_file_path)?;
    Ok(serde_json::from_reader(input_file)?)
}
