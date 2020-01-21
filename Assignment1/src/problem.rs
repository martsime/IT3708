use std::collections::HashMap;
use std::f64;
use std::fs::File;
use std::i32;
use std::io::{self, BufRead};
use std::path::Path;

use rand::{self, Rng};

use rayon::prelude::*;

use crate::simulation::{Encode, Simulation};
use crate::solution::{OptimalSolution, Solution};
use crate::utils::Pos;

const POPULATION_SIZE: i32 = 1000;

struct Customer {
    pub number: i32,
    pub pos: Pos,
    service_time: Option<i32>,
    demand: i32,
}

struct Depot {
    capacity: i32,
    pub number: i32,
    pub pos: Pos,
}

struct Vehicle {
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
    pub distances: HashMap<(i32, i32), f64>,
    pub capacities: HashMap<i32, i32>,
}

pub struct Problem {
    path: String,
    pub max_vehicles: i32, // Maximum number of vehicles available for each depot
    pub num_customers: i32, // Total number of customers
    pub num_depots: i32,   // Number of depots
    customers: Vec<Customer>,
    depots: Vec<Depot>,
    vehicles: Vec<Vehicle>,
    pub simulation: Simulation,
    optimal_solution: Option<OptimalSolution>,
    positions: HashMap<i32, Pos>,
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
        Problem::load_and_parse(path)
    }

    pub fn load_optimal_solution(&mut self, path: String) {
        let optimal_solution = OptimalSolution::new(path);
        self.optimal_solution = Some(optimal_solution);
    }

    pub fn calculate_distances(&self) -> HashMap<(i32, i32), f64> {
        let mut distances: HashMap<(i32, i32), f64> = HashMap::new();
        let mut positions: HashMap<i32, Pos> = HashMap::new();
        for customer in self.customers.iter() {
            positions.insert(customer.number, customer.pos.clone());
        }

        for vehicle in self.vehicles.iter() {
            let depot = vehicle.get_depot(&self.depots);
            let pos = depot.pos.clone();
            positions.insert(vehicle.number, pos);
        }

        for (key1, pos1) in positions.iter() {
            for (key2, pos2) in positions.iter() {
                let distance_key: (i32, i32) = (*key1, *key2);
                let distance = pos1.distance_to(pos2);
                distances.insert(distance_key, distance);
            }
        }

        distances
    }

    pub fn calculate_capacities(&self) -> HashMap<i32, i32> {
        let mut capacities: HashMap<i32, i32> = HashMap::new();
        for customer in self.customers.iter() {
            capacities.insert(customer.number, customer.demand);
        }

        for vehicle in self.vehicles.iter() {
            let depot = vehicle.get_depot(&self.depots);
            capacities.insert(vehicle.number, depot.capacity);
        }
        capacities
    }

    fn load_and_parse(path: String) -> Problem {
        let lines = Problem::load(&path);

        // Parse problem global settings
        let first_line = lines[0].clone();
        let max_vehicles = first_line[0];
        let num_customers = first_line[1];
        let num_depots = first_line[2];

        let depot_info_lines = &lines[1..=(num_customers as usize)];
        let depot_pos_start_index = (1 + num_customers + num_depots) as usize;
        let depot_pos_lines =
            &lines[depot_pos_start_index..(depot_pos_start_index + num_depots as usize)];

        let depots = Problem::parse_depots(depot_info_lines, depot_pos_lines, num_depots);

        let customer_start_index = (1 + num_depots) as usize;
        let customer_lines =
            &lines[customer_start_index..(customer_start_index + num_customers as usize)];

        let customers = Problem::parse_customers(customer_lines, num_customers);

        let mut positions: HashMap<i32, Pos> = HashMap::new();

        for customer in customers.iter() {
            positions.insert(customer.number, customer.pos.clone());
        }

        let mut vehicles: Vec<Vehicle> = Vec::new();
        let mut vehicle_number: i32 = num_customers + 1;
        for depot in depots.iter() {
            for _ in 0..max_vehicles {
                vehicles.push(Vehicle {
                    number: vehicle_number,
                    depot: depot.number,
                });
                positions.insert(vehicle_number, depot.pos.clone());
                vehicle_number += 1;
            }
        }

        Problem {
            path,
            max_vehicles,
            num_customers,
            num_depots,
            depots,
            customers,
            vehicles,
            positions,
            simulation: Simulation::new(),
            optimal_solution: None,
            model: None,
        }
    }

    fn parse_customers(customer_lines: &[Vec<i32>], num_customers: i32) -> Vec<Customer> {
        let mut customers: Vec<Customer> = Vec::with_capacity(num_customers as usize);

        // Load all customers
        for i in 0..num_customers {
            let line = customer_lines[i as usize].clone();
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
            let customer = Customer {
                number,
                pos,
                service_time,
                demand,
            };
            customers.push(customer);
        }
        return customers;
    }

    fn parse_depots(
        info_lines: &[Vec<i32>],
        pos_lines: &[Vec<i32>],
        num_depots: i32,
    ) -> Vec<Depot> {
        let mut depots: Vec<Depot> = Vec::with_capacity(num_depots as usize);

        for i in 0..num_depots {
            let info_line = info_lines[i as usize].clone();
            let _max_duration = match info_line[0] {
                0 => None,
                val => Some(val),
            };
            let capacity = info_line[1];
            let pos_line = pos_lines[i as usize].clone();
            let number = pos_line[0];
            let pos = Pos {
                x: pos_line[1],
                y: pos_line[2],
            };
            let depot = Depot {
                capacity,
                number,
                pos,
            };
            depots.push(depot);
        }
        return depots;
    }

    fn load(path: &String) -> Vec<Vec<i32>> {
        let path = Path::new(&path);
        let file = File::open(path).unwrap();
        let reader = io::BufReader::new(file);

        let lines: Vec<Vec<i32>> = reader
            .lines()
            .map(|line| {
                line.unwrap()
                    .split_whitespace()
                    .map(|num| num.parse().unwrap())
                    .collect()
            })
            .collect();
        lines
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
        self.create_model();
        self.simulation.population.chromosomes = (0..POPULATION_SIZE)
            .into_par_iter()
            .map(|_| {
                let route = self.custom_initial();
                Solution::new(route).encode()
            })
            .collect();

        let model = self.model.as_ref().unwrap();
        self.simulation.evaluate(model);
    }

    pub fn create_model(&mut self) {
        self.model = Some(Model {
            distances: self.calculate_distances(),
            capacities: self.calculate_capacities(),
        });
    }

    pub fn simulate(&mut self) -> Solution {
        let model = self.model.as_ref().unwrap();
        let mut solution: Solution = self.simulation.get_best_solution();
        for _ in 0..UPDATE_RATE {
            self.simulation.run(model);
            solution = self.simulation.get_best_solution();
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
                    let depot_number = vehicle.unwrap().depot;
                    *stop = depot_number;
                }
            }
        }

        println!(
            "Generation {}, Score: {}",
            self.simulation.generation,
            solution.score.unwrap(),
        );
        solution
    }

    fn random_initial(&self) -> Vec<Vec<i32>> {
        let mut routes = Vec::new();
        let mut unvisited_customers: Vec<Customer> = self.customers.iter().cloned().collect();

        let mut route_map: HashMap<i32, Vec<i32>> = HashMap::new();
        for vehicle in self.vehicles.iter() {
            let mut route = Vec::new();
            route.push(vehicle.number);
            route_map.insert(vehicle.number, route);
        }

        let mut rng = rand::thread_rng();

        while !unvisited_customers.is_empty() {
            let vehicle = &self.vehicles[rng.gen_range(0, self.vehicles.len())];
            let route = route_map.get_mut(&vehicle.number).unwrap();
            let last_stop = route.last().unwrap();
            let last_pos = self.positions.get(last_stop).unwrap();
            let customer_index = rng.gen_range(0, unvisited_customers.len());
            let customer = unvisited_customers.swap_remove(customer_index);
            route.push(customer.number);
        }

        for vehicle in self.vehicles.iter() {
            let mut route = route_map.get(&vehicle.number).unwrap().clone();
            route.push(vehicle.number);
            routes.push(route);
        }

        return routes;
    }

    fn custom_initial(&self) -> Vec<Vec<i32>> {
        let mut routes = Vec::new();
        let mut unvisited_customers: Vec<Customer> = self.customers.iter().cloned().collect();

        let mut route_map: HashMap<i32, Vec<i32>> = HashMap::new();
        for vehicle in self.vehicles.iter() {
            let mut route = Vec::new();
            route.push(vehicle.number);
            route_map.insert(vehicle.number, route);
        }

        let mut rng = rand::thread_rng();

        while !unvisited_customers.is_empty() {
            let vehicle = &self.vehicles[rng.gen_range(0, self.vehicles.len())];
            let route = route_map.get_mut(&vehicle.number).unwrap();
            let last_stop = route.last().unwrap();
            let last_pos = self.positions.get(last_stop).unwrap();
            let closest_customer = self
                .get_closest_customer(&last_pos, &mut unvisited_customers, 1000)
                .unwrap();
            route.push(closest_customer.number);
        }

        for vehicle in self.vehicles.iter() {
            let mut route = route_map.get(&vehicle.number).unwrap().clone();
            route.push(vehicle.number);
            routes.push(route);
        }

        return routes;
    }

    fn get_closest_customer(
        &self,
        point: &Pos,
        un_customers: &mut Vec<Customer>,
        capacity_left: i32,
    ) -> Option<Customer> {
        let mut closest_customer_index: i32 = -1;
        let mut shortest_distance = f64::MAX;
        for i in 0..un_customers.len() {
            let customer = &un_customers[i];
            let distance = point.distance_to(&customer.pos);
            if distance < shortest_distance && capacity_left >= customer.demand {
                shortest_distance = distance;
                closest_customer_index = i as i32;
            }
        }

        let mut closest_customer: Option<Customer> = None;

        if closest_customer_index >= 0 {
            closest_customer = Some(un_customers.swap_remove(closest_customer_index as usize));
        }

        return closest_customer;
    }
}
