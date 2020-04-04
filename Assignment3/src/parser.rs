use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn parse_file(path_str: &str) -> Vec<Vec<usize>> {
    let path = Path::new(path_str);
    let file = match File::open(Path::new(&path)) {
        Ok(file) => file,
        Err(err) => {
            panic!("Error opening file {}. Error: {}", path_str, err);
        }
    };
    let reader = io::BufReader::new(file);

    let lines: Vec<Vec<usize>> = reader
        .lines()
        .filter_map(|line| {
            let numbers: Vec<usize> = line
                .unwrap()
                .split_whitespace()
                .map(|number| number.parse::<usize>().unwrap())
                .collect();
            if numbers.len() > 0 {
                return Some(numbers);
            } else {
                return None;
            }
        })
        .collect();
    lines
}
