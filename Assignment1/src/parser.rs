use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use std::str::FromStr;

pub fn load(path: &String) -> Vec<Vec<String>> {
    let path = Path::new(&path);
    let file = File::open(path).unwrap();
    let reader = io::BufReader::new(file);

    let lines: Vec<Vec<String>> = reader
        .lines()
        .map(|line| {
            line.unwrap()
                .split_whitespace()
                .map(|word| word.into())
                .collect()
        })
        .collect();
    lines
}

pub fn parse_column<T>(lines: &[Vec<String>], column: usize, line_number: usize) -> Vec<T>
where
    T: FromStr,
{
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let value = match line[column].parse::<T>() {
                Ok(val) => val,
                Err(_) => panic!(
                    "Error parsing line {}: {}\n{:?}",
                    (line_number + i),
                    column,
                    line
                ),
            };
            value
        })
        .collect()
}

pub fn parse_columns<T>(
    lines: &[Vec<String>],
    column_start: usize,
    column_end: Option<usize>,
    line_number: usize,
) -> Vec<Vec<T>>
where
    T: FromStr,
{
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_slice = match column_end {
                Some(end) => &line[column_start..end],
                None => &line[column_start..],
            };
            let line_values = line_slice
                .iter()
                .enumerate()
                .map(|(column, value)| {
                    let parsed_value = match value.parse::<T>() {
                        Ok(val) => val,
                        Err(_) => panic!(
                            "error parsing line {}: {}\n{:?}",
                            (line_number + i),
                            column + column_start,
                            line
                        ),
                    };
                    parsed_value
                })
                .collect();
            line_values
        })
        .collect()
}
