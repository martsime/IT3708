use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use std::collections::HashMap;

use crate::problem::{Capacities, Distances, Problem};
use crate::simulation::{Chromosome, Encode, Gene};

pub struct Solution {
    pub routes: Vec<Vec<i32>>,
    pub score: Option<f64>,
}

impl Solution {
    pub fn new(routes: Vec<Vec<i32>>) -> Solution {
        Solution {
            routes,
            score: None,
        }
    }
    pub fn evaluate(&self, distances: &Distances, capacities: &Capacities) -> f64 {
        let chromosome = self.encode();
        chromosome.evaluate(distances, capacities)
    }
}

impl Encode for Solution {
    fn encode(&self) -> Chromosome {
        let mut genes: Vec<Gene> = Vec::new();
        for route in self.routes.iter() {
            let num_stops = route.len();
            if num_stops < 2 {
                panic!("Error in routes");
            }
            let depot = route[0];
            genes.push(Gene::Depot(depot));
            for i in 1..num_stops - 1 {
                genes.push(Gene::Customer(route[i]));
            }
        }
        Chromosome { genes }
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for route in self.routes.iter() {
            if route.len() > 2 {
                writeln!(f, "{:?}", route)?;
            }
        }
        Ok(())
    }
}

pub struct OptimalSolution {
    path: String,
    total_score: f64,
    depots: Vec<i32>,
    vehicles: Vec<i32>,
    scores: Vec<f64>,
    load: Vec<i32>,
    routes: Vec<Vec<i32>>,
}

impl OptimalSolution {
    pub fn new(path: String) -> OptimalSolution {
        let lines = OptimalSolution::load(&path);
        println!("Lines: {:?}", lines);
        let total_score = lines[0][0].parse::<f64>().unwrap();
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

    pub fn get_solution(&self, problem: &Problem) -> Solution {
        let max_vehicles = problem.max_vehicles;
        let num_customers = problem.num_customers;
        let num_depots = problem.num_depots;
        // Create a map from vehicle to route
        let mut route_map: HashMap<(i32, i32), usize> = HashMap::new();
        let num_routes = self.routes.len();

        for i in 0..num_routes {
            let depot = self.depots[i];
            let vehicle = self.vehicles[i];
            route_map.insert((depot, vehicle), i);
        }

        let mut routes = Vec::new();

        for d in 1..=num_depots {
            let depot_number = d + num_customers;
            for v in 1..=max_vehicles {
                let mut route = Vec::new();
                route.push(depot_number);
                let index = (d, v);
                let route_exists = route_map.get(&index);
                match route_exists {
                    Some(route_index) => {
                        let mut existing_route = self.routes[*route_index].clone();
                        route.append(&mut existing_route);
                    }
                    _ => {}
                };
                route.push(depot_number);
                routes.push(route);
            }
        }
        Solution::new(routes)
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

    fn parse_scores(input: &[Vec<String>]) -> Vec<f64> {
        let mut scores = Vec::new();
        for line in input {
            let score = line[2].parse::<f64>().unwrap();
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
            for customer in &line[5..line.len()] {
                let stop = customer.parse::<i32>().unwrap();
                route.push(stop);
            }
            routes.push(route);
        }
        routes
    }
}
