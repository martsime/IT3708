use std::collections::HashSet;
use std::f64;
use std::fmt;
use std::i32;

use crate::problem::{Capacities, Distances};
use crate::solution::Solution;

use rand::{self, Rng};
use rayon::prelude::*;

const ELITISM: usize = 2;
const MUTATION_RATE: f64 = 0.03;
const CROSSOVER_RATE: f64 = 0.5;

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

    pub fn single_mutation(&self) -> Chromosome {
        let mut new_chromosome = self.clone();
        let chromosome_length = new_chromosome.genes.len();
        let mut rng = rand::thread_rng();
        let index_one = rng.gen_range(0, chromosome_length);
        let index_two = rng.gen_range(0, chromosome_length);
        new_chromosome.genes.swap(index_one, index_two);
        new_chromosome
    }

    pub fn order_one_crossover(&self, other: &Chromosome) -> (Chromosome, Chromosome) {
        let mut child_one = self.clone();
        let mut child_two = other.clone();

        let chromosome_length = self.genes.len();
        let mut rng = rand::thread_rng();
        let index_one = rng.gen_range(0, chromosome_length);
        let index_two = rng.gen_range(index_one, chromosome_length);

        // Set of all the genes in the crossover sequence
        let set_one: HashSet<&Gene> = self.genes[index_one..index_two].iter().collect();
        let set_two: HashSet<&Gene> = other.genes[index_one..index_two].iter().collect();

        let mut insert_index_one = index_two;
        let mut insert_index_two = index_two;

        for i in 0..chromosome_length {
            // Wrap index around
            let new_index = (index_two + i) % chromosome_length;
            let new_gene_one = &other.genes[new_index];
            if !set_one.contains(&new_gene_one) {
                child_one.genes[insert_index_one] = new_gene_one.clone();
                insert_index_one = (insert_index_one + 1) % chromosome_length;
            }

            let new_gene_two = &self.genes[new_index];
            if !set_two.contains(&new_gene_two) {
                child_two.genes[insert_index_two] = new_gene_two.clone();
                insert_index_two = (insert_index_two + 1) % chromosome_length;
            }
        }

        (child_one, child_two)
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
                score += 1000.0;
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
        let mut scores: Vec<(usize, f64)> = (0..self.chromosomes.len())
            .into_par_iter()
            .map(|i| {
                let chromosome = &self.chromosomes[i];
                let score = chromosome.evaluate(distances, capacities);
                (i, score)
            })
            .collect();

        scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        self.scores = scores;
    }

    pub fn evolve(&self) -> Population {
        let mut new_chromosomes: Vec<Chromosome> = Vec::with_capacity(self.chromosomes.len());

        for i in 0..ELITISM {
            let elite_chromosome = &self.chromosomes[self.scores[i].0];
            new_chromosomes.push(elite_chromosome.clone());
        }

        // let weights: Vec<f64> = self.scores.iter().map(|tup| 1.0 / tup.1).collect();
        // let weights: Vec<usize> = (0..self.scores.len()).rev().collect();
        // let selection: Vec<usize> = self.scores.iter().map(|tup| tup.0).collect();

        // let dist = rand::distributions::WeightedIndex::new(&weights).unwrap();

        let num_scores = self.scores.len();

        let iterations = (self.chromosomes.len() - ELITISM) / 2;
        new_chromosomes.par_extend((0..iterations).into_par_iter().flat_map(|_| {
            let mut rng = rand::thread_rng();
            let one = rng.gen_range(0, num_scores);
            let two = rng.gen_range(0, num_scores);
            let three = rng.gen_range(0, num_scores);
            let four = rng.gen_range(0, num_scores);

            let parent_one: &Chromosome;
            let parent_two: &Chromosome;

            if self.scores[one].1 < self.scores[two].1 {
                parent_one = &self.chromosomes[self.scores[one].0];
            } else {
                parent_one = &self.chromosomes[self.scores[two].0];
            }

            if self.scores[three].1 < self.scores[four].1 {
                parent_two = &self.chromosomes[self.scores[three].0];
            } else {
                parent_two = &self.chromosomes[self.scores[four].0];
            }

            let crossover: f64 = rng.gen();
            let (mut child_one, mut child_two);
            if crossover < CROSSOVER_RATE {
                let (a, b) = parent_one.order_one_crossover(parent_two);
                child_one = a;
                child_two = b;
            } else {
                child_one = parent_one.clone();
                child_two = parent_two.clone();
            }
            let mutate_one: f64 = rng.gen();
            let mutate_two: f64 = rng.gen();

            let number_of_mutations = rng.gen_range(1, MUTATIONS_MAX);

            if mutate_one < MUTATION_RATE {
                for _ in 0..number_of_mutations {
                    child_one = child_one.single_mutation();
                }
            }

            if mutate_two < MUTATION_RATE {
                for _ in 0..number_of_mutations {
                    child_two = child_two.single_mutation();
                }
            }
            vec![child_one, child_two]
        }));

        let mut new_population = Population::new();
        new_population.chromosomes = new_chromosomes;
        new_population
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
        Solution::new(routes)
    }
}

pub struct Simulation {
    pub population: Population,
    pub generation: i32,
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            population: Population::new(),
            generation: 1,
        }
    }
    pub fn run(&mut self, distances: &Distances, capacities: &Capacities) {
        let new_population = self.population.evolve();
        self.population = new_population;
        self.population.evaluate(distances, capacities);

        self.generation += 1;
    }

    pub fn get_best_solution(&self) -> Solution {
        let (index, score) = self.population.scores[0];
        let chromosome = &self.population.chromosomes[index];
        let mut solution = chromosome.decode();
        solution.score = Some(score);
        solution
    }

    pub fn add_solution(&mut self, routes: Vec<Vec<i32>>) {
        let solution = Solution::new(routes);
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
}
