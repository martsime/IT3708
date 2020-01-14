use std::collections::HashSet;
use std::f64;
use std::fmt;
use std::i32;

use crate::problem::{Capacities, Distances};
use crate::solution::Solution;

use rand::prelude::*;
use rand::{self, Rng};

#[derive(Clone, Eq, Hash, PartialEq)]
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

impl fmt::Debug for Gene {
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

    pub fn from_vec(vec: Vec<Gene>) -> Chromosome {
        Chromosome { genes: vec }
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
        let chromosome_length = new_chromosome.genes.len();
        let mut rng = rand::thread_rng();
        let index_one = rng.gen_range(0, chromosome_length);
        let index_two = rng.gen_range(0, chromosome_length);
        new_chromosome.genes.swap(index_one, index_two);
        // println!("Swapping index {} with {}", index_one, index_two);
        new_chromosome
    }

    pub fn order_one_crossover(&self, other: &Chromosome) -> Chromosome {
        let mut new_chromosome = self.clone();

        let chromosome_length = new_chromosome.genes.len();
        let mut rng = rand::thread_rng();
        let index_one = rng.gen_range(0, chromosome_length);
        let index_two = rng.gen_range(index_one, chromosome_length);

        // Set of all the genes in the crossover sequence
        let set: HashSet<&Gene> = self.genes[index_one..index_two].iter().collect();

        let mut insert_index = index_two;

        for i in 0..chromosome_length {
            // Wrap index around
            let new_index = (index_two + i) % chromosome_length;
            let new_gene = &other.genes[new_index];
            if !set.contains(&new_gene) {
                new_chromosome.genes[insert_index] = new_gene.clone();
                insert_index = (insert_index + 1) % chromosome_length;
            }
        }

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
        let weight_multiple = 5;

        let mut weights = Vec::new();
        let mut indices = Vec::new();

        for i in 0..self.scores.len() {
            let score = self.scores[i].1;
            if score < 10000.0 {
                let weight = (max_weight - i) * weight_multiple;
                weights.push(weight);
                indices.push(self.scores[i].0);
            } else {
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
    pub population: Population,
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
        for i in selection.iter().cloned() {
            let selected = &self.population.chromosomes[i];
            let mut rng = rand::thread_rng();
            let roll: f64 = rng.gen();

            if roll > 0.9 {
                let other_selected_index = selection[rng.gen_range(0, selection.len())];
                let other_selected = &self.population.chromosomes[other_selected_index];
                new_population.add(selected.order_one_crossover(other_selected));
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

    pub fn add_solution(&mut self, routes: Vec<Vec<i32>>) {
        let solution = Solution { routes };
        let chromosome = solution.encode();
        self.population.chromosomes.push(chromosome);
    }

    pub fn evaluate(&mut self, distances: &Distances, capacities: &Capacities) {
        self.population.evaluate(distances, capacities);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gene_eq() {
        let gene_one = Gene::Depot(1);
        let gene_two = Gene::Depot(1);
        let gene_three = Gene::Customer(1);

        assert_eq!(gene_one, gene_two);
        assert_ne!(gene_one, gene_three);
    }

    #[test]
    fn test_order_one_crossover() {
        let num_customers: usize = 10;
        let num_depots: usize = 2;
        let num_vechicles: usize = 3;

        let num_genes = num_customers + num_depots * num_vechicles;
        let mut genes: Vec<Gene> = Vec::with_capacity(num_genes);
        for c in 1..=num_customers {
            genes.push(Gene::Customer(c as i32));
        }

        for d in 1..=num_depots {
            for _ in 1..=num_vechicles {
                genes.push(Gene::Depot((d + num_customers) as i32));
            }
        }

        let mut rng = rand::thread_rng();
        let mut indices: Vec<usize> = (0..num_genes).collect();
        let mut chromosome_one = Chromosome::new();
        let mut chromosome_two = Chromosome::new();

        indices.shuffle(&mut rng);
        for i in indices.iter().cloned() {
            chromosome_one.genes.push(genes[i].clone());
        }

        indices.shuffle(&mut rng);
        for i in indices.iter().cloned() {
            chromosome_two.genes.push(genes[i].clone());
        }

        let new_chromosome = chromosome_one.order_one_crossover(&chromosome_two);

        let mut valid: bool = true;
        for gene in genes {
            if !new_chromosome.genes.contains(&gene) {
                valid = false;
            }
        }

        assert_eq!(true, valid);
    }
}
