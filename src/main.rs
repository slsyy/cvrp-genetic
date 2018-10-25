#[macro_use] extern crate serde_derive;


extern crate clap;

use std::fs::File;

use clap::App;
use clap::Arg;

mod description;
use description::Description;

extern crate serde;
extern crate serde_json;

fn main() {
    let matches = App::new("cvrp-genetic")
        .arg(Arg::from_usage("<input> 'problem description input file'"))
        .get_matches();

    let input_file_path = matches.value_of("input").unwrap();
    let input_file = File::open(input_file_path).unwrap_or_else(|error| {
        eprintln!("Error openning file {}: {}", input_file_path, error);
        ::std::process::exit(1)
    });

    let _desc: Description = serde_json::from_reader(input_file).unwrap_or_else(|error| {
        eprintln!("Error parsing file {}: {}", input_file_path, error);
        ::std::process::exit(1)
    });
}
