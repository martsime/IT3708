use std::cmp;
use std::f64;
use std::fmt;

use rand::{self, Rng};

use crate::problem::{Depot, Model, Problem};
use crate::utils::Pos;
use crate::CONFIG;

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
    pub fn evaluate(&mut self, model: &Model, depot: &Depot) {
        // Cost already calculated
        if let Some(_) = self.cost {
            return;
        }
        let mut score: f64 = 0.0;
        let mut capacity_left: i32 = depot.capacity;
        let mut current_pos: &Pos = &depot.pos;
        for customer_number in self.customers.iter() {
            let customer_pos = match model.positions.get(customer_number) {
                Some(pos) => pos,
                None => {
                    panic!(
                        "ERROR! Could not find position for node: {}",
                        customer_number
                    );
                }
            };
            let customer_demand = match model.capacities.get(customer_number) {
                Some(demand) => demand,
                None => {
                    panic!("ERROR! Could not find demand for node: {}", customer_number);
                }
            };
            score += current_pos.distance_to(customer_pos);
            capacity_left -= customer_demand;
            if capacity_left < 0 {
                score += CONFIG.infeasibility_penalty as f64;
            }

            current_pos = customer_pos;
        }
        // Add distane back to depot
        score += depot.pos.distance_to(current_pos);
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
        .map(|customer| Route {
            cost: None,
            customers: vec![*customer],
        })
        .collect()
}

fn evaluate_routes(routes: &mut Vec<Route>, model: &Model, depot: &Depot) {
    for route in routes.iter_mut() {
        route.evaluate(model, depot);
    }
}

fn calculate_savings(routes: &Vec<Route>, model: &Model, depot: &Depot) -> Vec<Vec<f64>> {
    let num_routes = routes.len();
    let mut savings: Vec<Vec<f64>> = vec![vec![-100000.0; num_routes]; num_routes];

    for (i1, r1) in routes.iter().enumerate() {
        for (i2, r2) in routes.iter().enumerate() {
            if r1 == r2 {
                continue;
            }
            let mut merged_route = merge_routes(r1, r2);
            merged_route.evaluate(model, depot);
            let saving = r1.get_cost() + r2.get_cost() - merged_route.get_cost();
            savings[i1][i2] = saving;
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

fn sort_savings(savings: &Vec<Vec<f64>>) -> Vec<((usize, usize), f64)> {
    let mut savings: Vec<((usize, usize), f64)> = savings
        .iter()
        .enumerate()
        .flat_map(|(i, row)| {
            let r: Vec<((usize, usize), f64)> = row
                .iter()
                .cloned()
                .enumerate()
                .map(|(j, saving)| ((i, j), saving))
                .collect();
            r
        })
        .collect();

    // Filter out all where indices are the same
    savings.retain(|((i, j), _)| i != j);

    // Reverse (largest first)
    savings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    savings
}

fn select_routes_to_merge(sorted_savings: &Vec<((usize, usize), f64)>) -> (usize, usize) {
    let cws_bias = cmp::min(CONFIG.cws_bias, sorted_savings.len());

    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0, cws_bias);
    if index >= sorted_savings.len() {
        panic!("This is not allowed!");
    }
    let ((i, j), _) = sorted_savings[index];
    (i, j)
}

fn positive_saving_left(sorted_savings: &Vec<((usize, usize), f64)>) -> bool {
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
        let mut total_count = 0;
        for r in routes.iter() {
            for _ in r.customers.iter() {
                total_count += 1;
            }
        }
        evaluate_routes(&mut routes, &model, &depot);
        loop {
            let mut new_count = 0;
            for r in routes.iter() {
                for _ in r.customers.iter() {
                    new_count += 1;
                }
            }
            if new_count != total_count {
                // panic!("New {} != Old {}", new_count, total_count);
            }
            let savings_matrix = calculate_savings(&routes, model, depot);
            let sorted_savings = sort_savings(&savings_matrix);

            // Continue merging until we have enough vehicles and there is no saving
            let enough_vehicles = routes.len() <= problem.max_vehicles as usize;
            if !positive_saving_left(&sorted_savings) && enough_vehicles {
                break;
            }

            let (i, j) = select_routes_to_merge(&sorted_savings);
            let route_one = &routes[i];
            let route_two = &routes[j];
            let mut new_route = merge_routes(route_one, route_two);
            new_route.evaluate(model, depot);
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
