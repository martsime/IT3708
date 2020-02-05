use std::fmt::{self, Write};
use std::fs::{File, OpenOptions};

use std::io::Write as wr;

use std::collections::HashMap;

use crate::config::CONFIG;
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

    pub fn evaluate(&mut self, model: &Model) {
        let mut chromosome = self.encode();
        self.score = Some(chromosome.evaluate(model));
    }

    fn evaluate_route(&self, route: &Vec<i32>, model: &Model) -> (i32, f64) {
        let start_node = route[0];
        let mut current_node = start_node;
        let mut cap_used = 0;
        let mut score: f64 = 0.0;

        for index in 1..(route.len() - 1) {
            let new_node = route[index];
            score += model.get_distance(current_node as usize, new_node as usize);
            cap_used += model.get_demand(new_node as usize);
            current_node = new_node;
        }

        // Back to depot
        score += model.get_distance(current_node as usize, start_node as usize);

        (cap_used, score)
    }

    fn format_output(&self, problem: &Problem, model: &Model) -> String {
        let mut output = String::new();
        writeln!(&mut output, "{:.2}", self.score.unwrap()).unwrap();

        let mut v_num: i32 = 0;
        let mut depot: i32 = 1;
        for vehicle in problem.vehicles.iter() {
            let new_depot = vehicle.depot - problem.num_customers;
            if depot == new_depot {
                v_num += 1;
            } else {
                v_num = 1;
                depot = new_depot;
            }

            let mut route: Option<&Vec<i32>> = None;

            for r in self.routes.iter() {
                if *r.first().unwrap() == vehicle.number {
                    route = Some(r);
                }
            }

            let route: &Vec<i32> = match route {
                Some(r) => r,
                None => {
                    panic!("No route for vehicle {}", vehicle.number);
                }
            };

            if route.len() == 2 {
                continue;
            }

            let (cap, score) = self.evaluate_route(route, model);
            write!(&mut output, "{}\t", depot).unwrap();
            write!(&mut output, "{}\t", v_num).unwrap();
            write!(&mut output, "{:.2}\t", score).unwrap();
            write!(&mut output, "{}\t", cap).unwrap();
            write!(&mut output, "{}\t", depot).unwrap();
            for i in 1..(route.len() - 2) {
                write!(&mut output, "{} ", route[i]).unwrap();
            }
            let last_stop = route[route.len() - 2];
            write!(&mut output, "{}\n", last_stop).unwrap();
        }

        output
    }

    pub fn write_to_file(&mut self, problem: &Problem, model: &Model) {
        let content = self.format_output(problem, model);
        let content = content.trim();
        println!("{}", content);
        let file_path = CONFIG.solution_path.clone();
        let mut file: File = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
        {
            Ok(file) => file,
            Err(_) => {
                panic!("Failed to open file {}", CONFIG.solution_path);
            }
        };
        match file.write_all(content.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                panic!("Failed to write to solution file");
            }
        }
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

#[allow(dead_code)]
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

    #[allow(dead_code)]
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

        let mut vehicle_num = num_customers + 1;
        for d in 1..=num_depots {
            for v in 1..=max_vehicles {
                let mut route = Vec::new();
                route.push(vehicle_num);
                let index = (d, v);
                let route_exists = route_map.get(&index);
                match route_exists {
                    Some(route_index) => {
                        let mut existing_route = self.routes[*route_index].clone();
                        route.append(&mut existing_route);
                    }
                    _ => {}
                };
                route.push(vehicle_num);
                routes.push(route);
                vehicle_num += 1;
            }
        }
        Solution::new(routes)
    }
}
