use pyo3::prelude::*;
use std::io::{self, BufRead};
use std::fs::File;
use std::path::Path;

#[pyclass(module = "genetic")]
struct GeneticProgram {
    problem: Problem,
}


struct Pos {
    pub x: i32,
    pub y: i32,
}

struct Customer {
    pub number: i32,
    pub pos: Pos,
    service_time: Option<i32>,
    demand: i32,
}

struct Depot {
    max_duration: Option<i32>,
    capacity: i32,
    pub number: i32,
    pub pos: Pos,
}

struct Problem {
    path: String,
    max_vehicles: i32, // Maximum number of vehicles available for each depot
    num_customers: i32, // Total number of customers
    num_depots: i32, // Number of depots
    customers: Vec<Customer>,
    depots: Vec<Depot>,
}

impl Problem {
    pub fn new(path: String) -> Problem {
        Problem::load_and_parse(path)
    }

    fn print_path(&self) {
        println!("Path: {}", self.path)
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
        let depot_pos_lines = &lines[depot_pos_start_index..(depot_pos_start_index + num_depots as usize)];

        let depots = Problem::parse_depots(depot_info_lines, depot_pos_lines, num_depots);

        let customer_start_index = (1 + num_depots) as usize;
        let customer_lines = &lines[customer_start_index..(customer_start_index + num_customers as usize)];

        let customers = Problem::parse_customers(customer_lines, num_customers);

        Problem {
            path,
            max_vehicles,
            num_customers,
            num_depots,
            depots,
            customers,
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
        };
        return customers;
    }

    fn parse_depots(info_lines: &[Vec<i32>], pos_lines: &[Vec<i32>], num_depots: i32) -> Vec<Depot> {
        let mut depots: Vec<Depot> = Vec::with_capacity(num_depots as usize);

        for i in 0..num_depots {
            let info_line = info_lines[i as usize].clone();
            let max_duration = match info_line[0] {
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
                max_duration,
                capacity,
                number,
                pos,
            };
            depots.push(depot);
        };
        return depots;
    }

    fn load(path: &String) -> Vec<Vec<i32>> {
        let path = Path::new(&path);
        let file = File::open(path).unwrap();
        let reader = io::BufReader::new(file);
        
        let lines: Vec<Vec<i32>> = reader.lines()
            .map(|line| {
                line.unwrap().split_whitespace().map(|num| {
                    num.parse().unwrap()
                }).collect()
            }).collect();
        lines
    }

    pub fn generate_objects(&self) -> Vec<(i32, i32, i32)> {
        let mut objects: Vec<(i32, i32, i32)> = Vec::with_capacity((self.num_customers + self.num_depots) as usize);
        for customer in self.customers.iter() {
            objects.push((customer.number, customer.pos.x, customer.pos.y));
        };
        for depot in self.depots.iter() {
            objects.push((depot.number, depot.pos.x, depot.pos.y));
        };

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
}

#[pymethods]
impl GeneticProgram {
    #[new]
    fn new(obj: &PyRawObject, path: String) {
        let problem = Problem::new(path);
        problem.print_path();
        obj.init(
            GeneticProgram {
                problem,
            }
        );
    }

    fn get_boundaries(&self) -> PyResult<(i32, i32, i32, i32)> {
        Ok(self.problem.get_boundaries())
    }
}

#[pymodule]
fn genetic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GeneticProgram>()?;
    Ok(())
}
