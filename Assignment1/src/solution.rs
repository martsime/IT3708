use std::fmt;

use std::collections::HashMap;

use crate::parser;
use crate::problem::{Model, Problem};
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
    pub fn evaluate(&self, model: &Model) -> f64 {
        let mut chromosome = self.encode();
        chromosome.evaluate(model)
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
        Chromosome { genes, score: None }
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
        let lines = parser::load(&path);
        let total_score = parser::parse_column::<f64>(&lines[0..1], 0, 1)[0];
        let lines_slice = &lines[1..];
        let depots = parser::parse_column::<i32>(lines_slice, 0, 2);
        let vehicles = parser::parse_column::<i32>(lines_slice, 1, 2);
        let scores = parser::parse_column::<f64>(lines_slice, 2, 2);
        let load = parser::parse_column::<i32>(lines_slice, 3, 2);

        let routes = parser::parse_columns::<i32>(lines_slice, 5, None, 2);

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
}
