use std::collections::HashMap;
use std::f64;
use std::hash::{Hash, Hasher};
use std::i32;

use rayon::prelude::*;

use crate::config::CONFIG;
use crate::heuristic;
use crate::parser;
use crate::simulation::{Chromosome, Encode, Simulation};
use crate::solution::{OptimalSolution, Solution};
use crate::utils::Pos;

pub struct Customer {
    pub number: i32,
    pub pos: Pos,
    service_time: Option<i32>,
    demand: i32,
}

#[derive(Eq, PartialEq)]
pub struct Depot {
    pub capacity: i32,
    pub number: i32,
    pub pos: Pos,
}

impl Hash for Depot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}

pub struct Vehicle {
    pub capacity: i32,
    pub number: i32,
    pub depot: i32,
}

impl Vehicle {
    pub fn get_depot<'a>(&self, depots: &'a Vec<Depot>) -> &'a Depot {
        let mut depot = None;
        for d in depots.iter() {
            if d.number == self.depot {
                depot = Some(d);
            }
        }
        if let None = depot {
            panic!("Unable to find depot for vehicle!");
        }
        depot.unwrap()
    }
}

pub struct Model {
    pub num_nodes: usize,
    pub distances: Vec<f64>,
    pub capacities: Vec<i32>,
}

impl Model {
    pub fn get_distance(&self, from: usize, to: usize) -> f64 {
        let index = (from - 1) * self.num_nodes + (to - 1);
        self.distances[index]
    }

    pub fn get_demand(&self, node: usize) -> i32 {
        self.capacities[node - 1]
    }
}

pub struct Problem {
    pub max_vehicles: i32,  // Maximum number of vehicles available for each depot
    pub num_customers: i32, // Total number of customers
    pub num_depots: i32,    // Number of depots
    customers: Vec<Customer>,
    depots: Vec<Depot>,
    pub vehicles: Vec<Vehicle>,
    pub simulation: Simulation,
    optimal_solution: Option<OptimalSolution>,
    model: Option<Model>,
}

impl Clone for Customer {
    fn clone(&self) -> Self {
        Customer {
            number: self.number,
            pos: self.pos.clone(),
            service_time: self.service_time,
            demand: self.demand,
        }
    }
}

impl Problem {
    pub fn new(path: String) -> Problem {
        let lines = parser::load(&path);

        // Parse problem global settings
        let first_line = parser::parse_line::<i32>(&lines[0], 1);
        let max_vehicles = first_line[0];
        let num_customers = first_line[1];
        let num_depots = first_line[2];

        // Parse depots
        let depot_info_lines = &lines[1..=(num_depots as usize)];
        let depot_pos_start_index = (1 + num_customers + num_depots) as usize;
        let depot_pos_lines =
            &lines[depot_pos_start_index..(depot_pos_start_index + num_depots as usize)];

        let depots: Vec<Depot> = depot_info_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let info_line = parser::parse_line::<i32>(line, i);
                let _max_duration = match info_line[0] {
                    0 => None,
                    val => Some(val),
                };
                let capacity = info_line[1];
                let pos_line = parser::parse_line::<i32>(&depot_pos_lines[i], i);
                let number = pos_line[0];
                let pos = Pos {
                    x: pos_line[1],
                    y: pos_line[2],
                };
                Depot {
                    capacity,
                    number,
                    pos,
                }
            })
            .collect();

        // Parse customers
        let customer_start_index = (1 + num_depots) as usize;
        let customer_lines =
            &lines[customer_start_index..(customer_start_index + num_customers as usize)];

        let customers: Vec<Customer> = customer_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line = parser::parse_line::<i32>(line, i);
                let number = line[0];
                let pos = Pos {
                    x: line[1],
                    y: line[2],
                };
                let service_time = match line[3] {
                    0 => None,
                    val => Some(val),
                };
                let demand = line[4];
                Customer {
                    number,
                    pos,
                    service_time,
                    demand,
                }
            })
            .collect();

        let mut vehicles: Vec<Vehicle> = Vec::new();
        let mut vehicle_number: i32 = num_customers + 1;
        for depot in depots.iter() {
            for _ in 0..max_vehicles {
                vehicles.push(Vehicle {
                    number: vehicle_number,
                    depot: depot.number,
                    capacity: depot.capacity,
                });
                vehicle_number += 1;
            }
        }

        let mut problem = Problem {
            max_vehicles,
            num_customers,
            num_depots,
            depots,
            customers,
            vehicles,
            simulation: Simulation::new(),
            optimal_solution: None,
            model: None,
        };
        problem.create_model();
        return problem;
    }

    pub fn get_vehicle_for_depot(&self, depot: &Depot) -> &Vehicle {
        let mut vehicle: Option<&Vehicle> = None;

        for v in self.vehicles.iter() {
            if v.depot == depot.number {
                vehicle = Some(v);
                break;
            }
        }

        match vehicle {
            Some(v) => v,
            None => {
                panic!("No vehicles for depot!");
            }
        }
    }

    pub fn create_model(&mut self) {
        let num_nodes = self.vehicles.len() + self.customers.len();
        self.model = Some(Model {
            num_nodes: num_nodes,
            distances: self.calculate_distances(),
            capacities: self.calculate_capacities(),
        });
    }

    pub fn calculate_distances(&self) -> Vec<f64> {
        let num_nodes = self.customers.len() + self.vehicles.len();
        let mut distances: Vec<f64> = vec![0.0; num_nodes * num_nodes];

        let mut positions: HashMap<usize, Pos> = HashMap::new();
        for customer in self.customers.iter() {
            positions.insert(customer.number as usize, customer.pos.clone());
        }

        for vehicle in self.vehicles.iter() {
            let depot = vehicle.get_depot(&self.depots);
            let pos = depot.pos.clone();
            positions.insert(vehicle.number as usize, pos);
        }

        for (key1, pos1) in positions.iter() {
            for (key2, pos2) in positions.iter() {
                let index = (key1 - 1) * num_nodes + (key2 - 1);
                let distance = pos1.distance_to(pos2);
                distances[index] = distance;
            }
        }

        distances
    }

    pub fn get_solution(&self) -> Solution {
        let mut solution: Solution;
        if CONFIG.show_solution {
            solution = OptimalSolution::new(CONFIG.solution_path.clone()).get_solution(&self);
        } else if CONFIG.show_optimal_solution {
            solution =
                OptimalSolution::new(CONFIG.optimal_solution_path.clone()).get_solution(&self);
        } else {
            panic!("No solution set!");
        }

        let model = self.model.as_ref().unwrap();
        solution.evaluate(model);

        println!(
            "Generation {}, Score: {}",
            self.simulation.generation,
            solution.score.unwrap(),
        );

        for route in solution.routes.iter_mut() {
            for stop in route.iter_mut() {
                if *stop > self.num_customers {
                    let mut vehicle = None;
                    for v in self.vehicles.iter() {
                        if v.number == *stop {
                            vehicle = Some(v);
                        }
                    }
                    if let None = vehicle {
                        panic!("Vehicle for stop {} not found!", stop);
                    }
                    let depot_number = match vehicle {
                        Some(v) => v.depot,
                        None => {
                            panic!("No vehicle found!");
                        }
                    };
                    *stop = depot_number;
                }
            }
        }

        solution
    }

    pub fn load_optimal_solution(&mut self, path: String) {
        let optimal_solution = OptimalSolution::new(path);
        self.optimal_solution = Some(optimal_solution);
    }

    pub fn calculate_capacities(&self) -> Vec<i32> {
        let num_nodes = self.customers.len() + self.vehicles.len();
        let mut capacities: Vec<i32> = vec![0; num_nodes];
        for c in self.customers.iter() {
            capacities[c.number as usize - 1] = c.demand;
        }

        for v in self.vehicles.iter() {
            capacities[v.number as usize - 1] = v.capacity;
        }

        capacities
    }

    pub fn get_customers(&self) -> HashMap<i32, (i32, i32)> {
        let mut hashmap = HashMap::new();
        for customer in self.customers.iter() {
            hashmap.insert(customer.number, (customer.pos.x, customer.pos.y));
        }
        return hashmap;
    }

    pub fn get_depots(&self) -> HashMap<i32, (i32, i32)> {
        let mut hashmap = HashMap::new();
        for depot in self.depots.iter() {
            hashmap.insert(depot.number, (depot.pos.x, depot.pos.y));
        }
        return hashmap;
    }

    pub fn get_boundaries(&self) -> (i32, i32, i32, i32) {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for customer in self.customers.iter() {
            let (x, y) = (customer.pos.x, customer.pos.y);
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        }

        for depot in self.depots.iter() {
            let (x, y) = (depot.pos.x, depot.pos.y);
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        }

        return (min_x, min_y, max_x, max_y);
    }

    pub fn generate_population(&mut self) {
        if CONFIG.verbose {
            println!("Generating population");
        }
        let model = self.model.as_ref().unwrap();
        while self.simulation.population.size() < CONFIG.population_size {
            let new_chromosomes: Vec<Chromosome> = (0..CONFIG.population_gen_step)
                .into_par_iter()
                .map(|_| {
                    let route = heuristic::savings_init(&model, &self);
                    Solution::new(route).encode()
                })
                .collect();

            self.simulation
                .population
                .chromosomes
                .par_extend(new_chromosomes);

            if CONFIG.verbose {
                let generated = self.simulation.population.size();
                println!(
                    "Generated {} of {} individuals",
                    generated, CONFIG.population_size
                );
            }
        }
        self.simulation.evaluate(model);
    }

    pub fn simulate(&mut self) -> Solution {
        let model = self.model.as_ref().unwrap();
        let mut solution: Solution = self.simulation.get_best_solution();
        for _ in 0..CONFIG.draw_rate {
            self.simulation.run(model);
            solution = self.simulation.get_best_solution();
        }

        println!(
            "Generation {}, Score: {}",
            self.simulation.generation,
            solution.score.unwrap(),
        );

        if self.simulation.generation as usize == CONFIG.generations {
            solution.write_to_file(&self, model);
        }

        for route in solution.routes.iter_mut() {
            for stop in route.iter_mut() {
                if *stop > self.num_customers {
                    let mut vehicle = None;
                    for v in self.vehicles.iter() {
                        if v.number == *stop {
                            vehicle = Some(v);
                        }
                    }
                    if let None = vehicle {
                        panic!("Vehicle for stop {} not found!", stop);
                    }
                    let depot_number = match vehicle {
                        Some(v) => v.depot,
                        None => {
                            panic!("No vehicle found!");
                        }
                    };
                    *stop = depot_number;
                }
            }
        }

        solution
    }

    pub fn map_customers_to_depot(&self) -> HashMap<&Depot, Vec<Customer>> {
        // Assigns customers to the closest depot
        let mut depot_map: HashMap<&Depot, Vec<Customer>> = HashMap::new();

        for customer in self.customers.iter() {
            let mut distance = f64::MAX;
            let mut closest_depot: Option<&Depot> = None;
            for depot in self.depots.iter() {
                let new_distance = customer.pos.distance_to(&depot.pos);
                if new_distance < distance {
                    distance = new_distance;
                    closest_depot = Some(depot);
                }
            }
            match closest_depot {
                None => panic!("Failed to find closest depot!"),
                Some(depot) => match depot_map.get_mut(&depot) {
                    None => {
                        let mut new_depot_list = Vec::new();
                        new_depot_list.push(customer.clone());
                        depot_map.insert(depot, new_depot_list);
                    }
                    Some(depot_list) => {
                        depot_list.push(customer.clone());
                    }
                },
            }
        }
        return depot_map;
    }
}
