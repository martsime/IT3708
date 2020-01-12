use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub struct OptimalSolution {
    path: String,
    total_score: f32,
    depots: Vec<i32>,
    vehicles: Vec<i32>,
    scores: Vec<f32>,
    load: Vec<i32>,
    routes: Vec<Vec<i32>>,
}

impl OptimalSolution {
    pub fn new(path: String) -> OptimalSolution {
        let lines = OptimalSolution::load(&path);
        println!("Lines: {:?}", lines);
        let total_score = lines[0][0].parse::<f32>().unwrap();
        let lines_slice = &lines[1..];
        let depots = OptimalSolution::parse_depots(lines_slice);
        let vehicles = OptimalSolution::parse_vehicles(lines_slice);
        let scores = OptimalSolution::parse_scores(lines_slice);
        let load = OptimalSolution::parse_load(lines_slice);
        let routes = OptimalSolution::parse_routes(lines_slice);

        OptimalSolution {
            path,
            total_score,
            depots,
            vehicles,
            scores,
            load,
            routes,
        }
    }

    fn load(path: &String) -> Vec<Vec<String>> {
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

    fn parse_scores(input: &[Vec<String>]) -> Vec<f32> {
        let mut scores = Vec::new();
        for line in input {
            let score = line[2].parse::<f32>().unwrap();
            scores.push(score);
        }
        scores
    }

    fn parse_depots(input: &[Vec<String>]) -> Vec<i32> {
        let mut depots = Vec::new();
        for line in input {
            let depot_num = line[0].parse::<i32>().unwrap();
            depots.push(depot_num);
        }
        depots
    }

    fn parse_vehicles(input: &[Vec<String>]) -> Vec<i32> {
        let mut vehicles = Vec::new();
        for line in input {
            let vehicle_num = line[1].parse::<i32>().unwrap();
            vehicles.push(vehicle_num);
        }
        vehicles
    }

    fn parse_load(input: &[Vec<String>]) -> Vec<i32> {
        let mut loads = Vec::new();
        for line in input {
            let load = line[3].parse::<i32>().unwrap();
            loads.push(load);
        }
        loads
    }

    fn parse_routes(input: &[Vec<String>]) -> Vec<Vec<i32>> {
        let mut routes = Vec::new();
        for line in input {
            let mut route = Vec::new();
            for customer in &line[5..line.len() - 1] {
                let stop = customer.parse::<i32>().unwrap();
                route.push(stop);
            }
            routes.push(route);
        }
        routes
    }
}
