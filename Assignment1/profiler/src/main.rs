extern crate genetic;

use genetic::problem::Problem;
use std::{thread, time};

fn main() {
    println!("Starting");
    let ten_millis = time::Duration::from_secs(1);

    thread::sleep(ten_millis);
    println!("Done");
}
