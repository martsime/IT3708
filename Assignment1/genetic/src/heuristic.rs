use std::cmp;
use std::f64;
use std::fmt;

use lazysort::SortedBy;
use rand::{self, Rng};

use crate::config::CONFIG;
use crate::problem::{Model, Problem, Vehicle};

struct Savings {
    pub dim: usize,
    pub vec: Vec<f64>,
}

impl Savings {
    pub fn new(dim: usize) -> Savings {
        Savings {
            dim: dim,
            vec: vec![-100_000.0; dim * dim],
        }
    }

    pub fn change(&mut self, i: usize, j: usize, new_value: f64) {
        self.vec[i * self.dim + j] = new_value;
    }

    pub fn get_indices(&self, number: usize) -> (usize, usize) {
        let row = number / self.dim;
        let col = number % self.dim;
        (row, col)
    }
}

#[derive(PartialEq)]
struct Route {
    pub cost: Option<f64>,
    pub customers: Vec<i32>,
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cost = match self.cost {
            Some(cost) => format!("{}", cost),
            None => String::from("None"),
        };
        write!(f, "Cost: {} [ ", cost)?;
        for customer in self.customers.iter() {
            write!(f, "{} ", customer)?;
        }
        write!(f, "]")
    }
}

impl Route {
    pub fn evaluate(&mut self, model: &Model, vehicle: &Vehicle) {
        // Cost already calculated
        if let Some(_) = self.cost {
            return;
        }
        let mut score: f64 = 0.0;
        let mut capacity_left: i32 = vehicle.capacity;
        let start_node = vehicle.number as usize;
        let mut current_node = start_node;
        for customer_number in self.customers.iter() {
            score += model.get_distance(current_node, *customer_number as usize);
            capacity_left -= model.get_demand(*customer_number as usize);
            if capacity_left < 0 {
                score += CONFIG.infeasibility_penalty as f64;
            }

            current_node = *customer_number as usize;
        }
        // Add distane back to depot
        score += model.get_distance(current_node, start_node);
        self.cost = Some(score);
    }

    pub fn get_cost(&self) -> f64 {
        match self.cost {
            Some(cost) => cost,
            None => {
                panic!("Cost is not calculated for route!");
            }
        }
    }
}

fn single_customers_routes(customers: Vec<i32>) -> Vec<Route> {
    customers
        .iter()
        .map(|customer| {
            let mut customers: Vec<i32> = Vec::with_capacity(customers.len());
            customers.push(*customer);

            Route {
                cost: None,
                customers: customers,
            }
        })
        .collect()
}

fn evaluate_routes(routes: &mut Vec<Route>, model: &Model, vehicle: &Vehicle) {
    for route in routes.iter_mut() {
        route.evaluate(model, vehicle);
    }
}

fn calculate_savings(routes: &Vec<Route>, model: &Model, vehicle: &Vehicle) -> Savings {
    let num_routes = routes.len();
    let mut savings = Savings::new(num_routes);

    for (i1, r1) in routes.iter().enumerate() {
        for (i2, r2) in routes.iter().enumerate() {
            if r1 == r2 {
                continue;
            }
            let mut merged_route = merge_routes(r1, r2);
            merged_route.evaluate(model, vehicle);
            let saving = r1.get_cost() + r2.get_cost() - merged_route.get_cost();
            savings.change(i1, i2, saving);
        }
    }

    savings
}

fn merge_routes(r1: &Route, r2: &Route) -> Route {
    let merged: Vec<i32> = r1
        .customers
        .iter()
        .chain(r2.customers.iter())
        .cloned()
        .collect();

    Route {
        cost: None,
        customers: merged,
    }
}

fn remove_routes(routes: &mut Vec<Route>, index_one: usize, index_two: usize) {
    // Must remove the element with the highest index first
    let min_index = cmp::min(index_one, index_two);
    let max_index = cmp::max(index_one, index_two);

    if min_index == max_index {
        panic!("Cannot remove two routes with same index: {}", min_index);
    }

    routes.remove(max_index);
    routes.remove(min_index);
}

fn sort_savings(savings: &Savings) -> Vec<(usize, f64)> {
    let sorted_savings: Vec<(usize, f64)> = savings
        .vec
        .iter()
        .cloned()
        .enumerate()
        .filter(|(i, _)| {
            let (x, y) = savings.get_indices(*i);
            x != y
        })
        .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
        .take(CONFIG.cws_bias)
        .collect();

    sorted_savings
}

fn select_routes_to_merge(sorted_savings: &Vec<(usize, f64)>) -> usize {
    let cws_bias = cmp::min(CONFIG.cws_bias, sorted_savings.len());

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0, cws_bias);
    if index >= sorted_savings.len() {
        panic!("This is not allowed!");
    }
    let (i, _) = sorted_savings[index];
    i
}

fn positive_saving_left(sorted_savings: &Vec<(usize, f64)>) -> bool {
    match sorted_savings.first() {
        Some(saving) => {
            let (_, saved_cost) = saving;
            if *saved_cost > 0.0 {
                true
            } else {
                false
            }
        }
        None => false,
    }
}

pub fn savings_init(model: &Model, problem: &Problem) -> Vec<Vec<i32>> {
    let mut initial_solution = Vec::new();
    let depot_map = problem.map_customers_to_depot();
    for (depot, customers) in depot_map.iter() {
        let customers = customers.iter().map(|c| c.number).collect();
        let mut routes = single_customers_routes(customers);
        let vehicle = problem.get_vehicle_for_depot(depot);
        evaluate_routes(&mut routes, &model, &vehicle);
        loop {
            let savings_matrix = calculate_savings(&routes, model, vehicle);
            let sorted_savings = sort_savings(&savings_matrix);

            // Continue merging until we have enough vehicles and there is no saving
            let enough_vehicles = routes.len() <= problem.max_vehicles as usize;
            if !positive_saving_left(&sorted_savings) && enough_vehicles {
                break;
            }

            let i = select_routes_to_merge(&sorted_savings);
            let (i, j) = savings_matrix.get_indices(i);
            let route_one = &routes[i];
            let route_two = &routes[j];
            let mut new_route = merge_routes(route_one, route_two);
            new_route.evaluate(model, vehicle);
            remove_routes(&mut routes, i, j);
            routes.push(new_route);
        }

        let vehicles: Vec<i32> = problem
            .vehicles
            .iter()
            .filter(|v| v.depot == depot.number)
            .map(|v| v.number)
            .collect();

        for (i, v) in vehicles.iter().enumerate() {
            let mut route: Vec<i32> = Vec::new();
            route.push(*v);
            if i < routes.len() {
                route.extend(&routes[i].customers);
            }
            route.push(*v);
            initial_solution.push(route);
        }
    }

    initial_solution
}
