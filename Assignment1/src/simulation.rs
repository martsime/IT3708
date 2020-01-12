use std::fmt;

use std::collections::HashMap;
use std::i32;

use crate::utils::Pos;

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

    pub fn generate(&mut self) {
        for i in 0..5 {
            self.genes.push(Gene::Customer(i));
        }
        self.genes.push(Gene::Depot(15));

        for i in 5..10 {
            self.genes.push(Gene::Customer(i));
        }

        self.genes.push(Gene::Depot(15));
        for i in 10..15 {
            self.genes.push(Gene::Customer(i));
        }

        self.genes.push(Gene::Depot(15));
        self.genes.push(Gene::Depot(15));
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
                _ => {}
            }
        }
        index
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

pub struct Solution {
    pub routes: Vec<Vec<i32>>,
}

trait Decode {
    fn decode(&self) -> Solution;
}

trait Encode {
    fn encode(&self) -> Chromosome;
}

pub struct Population {
    pub chromosomes: Vec<Chromosome>,
    pub scores: Vec<f32>,
}

impl Population {
    pub fn new() -> Population {
        Population {
            chromosomes: Vec::new(),
            scores: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, positions: &HashMap<i32, Pos>) {
        let mut new_scores: Vec<f32> = Vec::with_capacity(self.chromosomes.len());
        for chromosome in self.chromosomes.iter() {
            let total_genes = chromosome.genes.len();
            let start_index = chromosome.get_first_depot_index().unwrap();

            let mut score: f32 = 0.0;
            let mut index = start_index;
            let mut current_node = chromosome.genes[index].value();

            let mut current_pos = positions.get(&current_node).unwrap();

            let mut depot_pos = current_pos;
            loop {
                index = (index + 1) % total_genes;
                match chromosome.genes[index] {
                    Gene::Depot(val) => {
                        // Back to last depot
                        score += current_pos.distance_to(depot_pos);
                        current_node = val;
                        current_pos = positions.get(&val).unwrap();
                        depot_pos = current_pos;
                    }
                    Gene::Customer(val) => {
                        let new_node = val;
                        let new_pos = positions.get(&current_node).unwrap();
                        score += current_pos.distance_to(new_pos);
                        current_node = new_node;
                        current_pos = new_pos;
                    }
                }

                if index == start_index {
                    break;
                }
            }
            new_scores.push(score);
        }
        self.scores = new_scores;
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
    nodes: HashMap<i32, Pos>,
}

impl Simulation {
    pub fn new() -> Simulation {
        let nodes: HashMap<i32, Pos> = HashMap::new();
        Simulation {
            population: Population::new(),
            generation: 1,
            nodes,
        }
    }

    pub fn set_positions(&mut self, positions: HashMap<i32, Pos>) {
        self.nodes = positions;
    }

    pub fn run(&mut self) {
        self.population.evaluate(&self.nodes);
        println!("Scores: {:?}", self.population.scores);
    }

    pub fn get_best_solution(&self) -> Solution {
        let chromosome = self.population.chromosomes.first().unwrap();
        let solution = chromosome.decode();
        solution
    }

    pub fn reset_population(&mut self) {
        self.population = Population::new();
    }

    pub fn create_population(&mut self, size: i32, initial_routes: Vec<Vec<i32>>) {
        let solution = Solution {
            routes: initial_routes,
        };
        let chromosome = solution.encode();
        for _ in 0..size {
            self.population.chromosomes.push(chromosome.clone());
        }
    }
}

impl Encode for Solution {
    fn encode(&self) -> Chromosome {
        println!("Routes: {:?}", self.routes);
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
        Chromosome { genes }
    }
}
