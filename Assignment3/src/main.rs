#[macro_use]
extern crate envconfig_derive;

mod config;
mod parser;

use config::CONFIG;
use parser::*;

fn main() {
    println!("File at: {}", CONFIG.problem_path());
    let problem = parse_problem();
    println!("Problem: {:?}", problem);
}
