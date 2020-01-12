use std::fmt;

use std::f64;
use std::i32;

use crate::problem::{Capacities, Distances};
use crate::solution::Solution;

use rand::prelude::*;
use rand::{self, Rng};

#[derive(Clone)]
pub enum Gene {
    Customer(i32),
    Depot(i32),
}

impl Gene {
    pub fn value(&self) -> i32 {
        match self {
            Gene::Customer(val) => i32::clone(val),
            Gene::Depot(val) => i32::clone(val),
        }
    }
}

impl fmt::Display for Gene {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Gene::Customer(val) => write!(f, "{}", val),
            Gene::Depot(val) => write!(f, "{}", val),
        }
    }
}

#[derive(Clone)]
pub struct Chromosome {
    pub genes: Vec<Gene>,
}

impl Chromosome {
    pub fn new() -> Chromosome {
        Chromosome { genes: Vec::new() }
    }

    fn get_first_depot_index(&self) -> Option<usize> {
        let mut index: Option<usize> = None;

        let total_genes = self.genes.len();

        // Start index is from the first depot
        for i in 0..total_genes {
            match self.genes[i] {
                Gene::Depot(_) => {
                    index = Some(i);
                    break;
                }
                _ => (),
            }
        }
        index
    }

    pub fn get_single_mutation(&self) -> Chromosome {
        let mut new_chromosome = self.clone();
        let new_chromosome_length = new_chromosome.genes.len();
        let mut rng = rand::thread_rng();
        let index_one = rng.gen_range(0, new_chromosome_length);
        let index_two = rng.gen_range(0, new_chromosome_length);
        new_chromosome.genes.swap(index_one, index_two);
        // println!("Swapping index {} with {}", index_one, index_two);
        new_chromosome
    }

    pub fn crossover_mutation(&self) -> Chromosome {
        let mut new_chromosome = self.clone();

        let new_chromosome_length = new_chromosome.genes.len();
        let mut rng = rand::thread_rng();
        let index_one = rng.gen_range(0, new_chromosome_length);
        let index_two = rng.gen_range(index_one, new_chromosome_length);

        let slice = &mut new_chromosome.genes[index_one..index_two];

        slice.shuffle(&mut rng);

        //println!("Index: ({}, {})", index_one, index_two);
        //println!("Old: {}", self);
        //println!("New: {}", new_chromosome);
        new_chromosome
    }

    pub fn evaluate(&self, distances: &Distances, capacities: &Capacities) -> f64 {
        let total_genes = self.genes.len();
        let start_index = self.get_first_depot_index().unwrap();

        let mut score: f64 = 0.0;
        let mut index = start_index;
        let mut current_node = self.genes[index].value();
        let mut depot_node = current_node;
        let mut distance_key: (i32, i32);

        let mut capacity_left = capacities.get(&depot_node).unwrap();

        loop {
            index = (index + 1) % total_genes;
            match self.genes[index] {
                Gene::Depot(node) => {
                    // Back to last depot
                    distance_key = (current_node, depot_node);
                    current_node = node;
                    depot_node = node;
                    capacity_left = capacities.get(&depot_node).unwrap();
                }
                Gene::Customer(node) => {
                    distance_key = (current_node, node);
                    current_node = node;
                    capacity_left -= capacities.get(&current_node).unwrap();
                }
            }
            let distance: f64 = match distances.get(&distance_key) {
                Some(val) => val,
                None => {
                    panic!("Unable to find distance: {:?}", distance_key);
                }
            };
            score += distance;

            if capacity_left < 0 {
                score += 10000.0;
            }

            if index == start_index {
                break;
            }
        }
        score
    }
}

impl fmt::Display for Chromosome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chromosome: [ ")?;
        for gene in self.genes.iter() {
            write!(f, "{} ", gene)?;
        }
        write!(f, "]")
    }
}

trait Decode {
    fn decode(&self) -> Solution;
}

pub trait Encode {
    fn encode(&self) -> Chromosome;
}

pub struct Population {
    pub chromosomes: Vec<Chromosome>,
    pub scores: Vec<(usize, f64)>,
}

impl Population {
    pub fn new() -> Population {
        Population {
            chromosomes: Vec::new(),
            scores: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, distances: &Distances, capacities: &Capacities) {
        let mut scores: Vec<(usize, f64)> = Vec::new();
        for i in 0..self.chromosomes.len() {
            let chromosome = &self.chromosomes[i];
            let score = chromosome.evaluate(distances, capacities);
            scores.push((i, score));
        }

        scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        self.scores = scores;
    }

    pub fn selection(&self) -> Vec<usize> {
        let max_weight = self.scores.len();
        let weight_multiple = 1;

        let mut weights = Vec::new();
        let mut indices = Vec::new();

        //println!("{:?}", self.scores);
        for i in 0..self.scores.len() {
            let score = self.scores[i].1;
            if score < 10000.0 {
                let weight = (max_weight - i) * weight_multiple;
                weights.push(weight);
                indices.push(self.scores[i].0);
            } else {
                //println!("Index {} skipped, score: {}", i, score);
            }
        }

        let dist = rand::distributions::WeightedIndex::new(&weights).unwrap();
        let mut rng = rand::thread_rng();

        let mut selected = Vec::with_capacity(self.scores.len());
        for _ in 0..self.scores.len() {
            let select = dist.sample(&mut rng);
            selected.push(indices[select]);
        }

        selected
    }

    pub fn add(&mut self, chromosome: Chromosome) {
        self.chromosomes.push(chromosome);
    }
}

impl Decode for Chromosome {
    fn decode(&self) -> Solution {
        let mut routes = Vec::new();

        let start_index = self.get_first_depot_index().unwrap();
        let total_genes = self.genes.len();

        let mut depot: i32 = self.genes[start_index].value();

        let mut index = start_index;
        loop {
            let mut route = Vec::new();
            route.push(depot);
            loop {
                index = (index + 1) % total_genes;
                match self.genes[index] {
                    Gene::Depot(val) => {
                        route.push(depot);
                        depot = val;
                        break;
                    }
                    Gene::Customer(val) => {
                        route.push(val);
                    }
                }
            }

            routes.push(route);

            if index == start_index {
                break;
            }
        }
        Solution { routes }
    }
}

pub struct Simulation {
    population: Population,
    generation: i32,
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            population: Population::new(),
            generation: 1,
        }
    }
    pub fn run(&mut self, distances: &Distances, capacities: &Capacities) {
        let mut new_population = Population::new();
        let selection = self.population.selection();
        for i in selection {
            let selected = &self.population.chromosomes[i];
            let mut rng = rand::thread_rng();
            let roll: f64 = rng.gen();

            if roll > 0.95 {
                new_population.add(selected.crossover_mutation());
            } else if roll > 0.6 {
                new_population.add(selected.get_single_mutation());
            } else {
                new_population.add(selected.clone());
            }
        }
        self.population = new_population;
        self.population.evaluate(distances, capacities);
        self.generation += 1;
    }

    pub fn get_best_solution(&self) -> Solution {
        let chromosome = &self.population.chromosomes[self.population.scores[0].0];
        let solution = chromosome.decode();
        solution
    }

    pub fn create_population(
        &mut self,
        size: i32,
        initial_routes: Vec<Vec<i32>>,
        distances: &Distances,
        capacities: &Capacities,
    ) {
        let solution = Solution {
            routes: initial_routes,
        };
        let chromosome = solution.encode();
        for _ in 0..size {
            self.population.chromosomes.push(chromosome.clone());
        }
        self.population.evaluate(distances, capacities);
    }
}
