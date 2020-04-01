#[macro_use]
extern crate envconfig_derive;

mod config;

use config::CONFIG;

fn main() {
    println!("Hello, world!");
}
