use std::collections::HashMap;
use std::f64;
use std::hash::{Hash, Hasher};
use std::i32;

use rand::seq::SliceRandom;
use rand::{self, Rng};

use rayon::prelude::*;

use crate::heuristic;
use crate::parser;
use crate::simulation::{Encode, Simulation};
use crate::solution::{OptimalSolution, Solution};
use crate::utils::Pos;
use crate::CONFIG;

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
    pub distances: HashMap<(i32, i32), f64>,
    pub capacities: HashMap<i32, i32>,
    pub positions: HashMap<i32, Pos>,
}

pub struct Problem {
    path: String,
    pub max_vehicles: i32, // Maximum number of vehicles available for each depot
    pub num_customers: i32, // Total number of customers
    pub num_depots: i32,   // Number of depots
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
            path,
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

    pub fn calculate_positions(&self) -> HashMap<i32, Pos> {
        let mut map = HashMap::new();
        for customer in self.customers.iter() {
            map.insert(customer.number, customer.pos.clone());
        }
        for depot in self.depots.iter() {
            map.insert(depot.number, depot.pos.clone());
        }

        return map;
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
        let model = self.model.as_ref().unwrap();
        self.simulation.population.chromosomes = (0..CONFIG.population_size)
            .into_par_iter()
            .map(|_| {
                let route = heuristic::savings_init(&model, &self);
                // println!("Generated: {}", i);
                Solution::new(route).encode()
            })
            .collect();

        self.simulation.evaluate(model);
    }

    pub fn create_model(&mut self) {
        self.model = Some(Model {
            distances: self.calculate_distances(),
            capacities: self.calculate_capacities(),
            positions: self.calculate_positions(),
        });
    }

    pub fn simulate(&mut self) -> Solution {
        let model = self.model.as_ref().unwrap();
        let mut solution: Solution = self.simulation.get_best_solution();
        for _ in 0..CONFIG.draw_rate {
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
            let last_pos = &self
                .model
                .as_ref()
                .unwrap()
                .positions
                .get(last_stop)
                .unwrap();
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
            let last_pos = &self
                .model
                .as_ref()
                .unwrap()
                .positions
                .get(last_stop)
                .unwrap();
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

    fn depot_initial(&self) -> Vec<Vec<i32>> {
        let mut routes = Vec::new();

        let mut depot_map = self.map_customers_to_depot();

        let mut rng = rand::thread_rng();
        for (_, value) in depot_map.iter_mut() {
            value.shuffle(&mut rng);
        }

        let mut route_map: HashMap<i32, Vec<i32>> = HashMap::new();
        for vehicle in self.vehicles.iter() {
            let mut route = Vec::new();
            route.push(vehicle.number);
            route_map.insert(vehicle.number, route);
        }

        fn customers_left(depot_map: &HashMap<&Depot, Vec<Customer>>) -> bool {
            for (_, value) in depot_map.iter() {
                if value.len() > 0 {
                    return true;
                }
            }
            return false;
        }

        while customers_left(&depot_map) {
            let vehicle = &self.vehicles[rng.gen_range(0, self.vehicles.len())];

            let depot = &vehicle.get_depot(&self.depots);
            let mut unvisited_customers = depot_map.get_mut(depot).unwrap();
            let route = route_map.get_mut(&vehicle.number).unwrap();
            let last_stop = route.last().unwrap();
            let last_pos = &self
                .model
                .as_ref()
                .unwrap()
                .positions
                .get(last_stop)
                .unwrap();
            let closest_customer =
                self.get_closest_customer(&last_pos, &mut unvisited_customers, 100000);
            match closest_customer {
                Some(c) => route.push(c.number),
                None => {}
            }
        }

        for vehicle in self.vehicles.iter() {
            let mut route = route_map.get(&vehicle.number).unwrap().clone();
            route.push(vehicle.number);
            routes.push(route);
        }

        for (key, value) in depot_map.iter() {
            if value.len() > 0 {
                println!("ERROR: Unserved customers at depot: {}", key.number);
            }
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
