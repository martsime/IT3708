use rand::Rng;

use crate::config::CONFIG;
use crate::problem::Problem;
use crate::utils;

use rayon::prelude::*;

#[derive(Debug)]
pub struct Pheromone {
    size: usize,
    matrix: Vec<f64>,
}

pub struct Node {
    job: usize,
    operation: usize,
}

#[derive(Clone)]
pub struct Ant {
    pub path: Vec<usize>,
    fitness: Option<usize>,
}

pub struct Colony {
    ants: Vec<Ant>,
    pub pheromones: Pheromone,
    pub nodes: Vec<Node>,
    best_ant: Option<Ant>,
}

pub struct ACO {
    pub colony: Colony,
    pub iteration: usize,
    best_ant: Option<Ant>,
}

impl Pheromone {
    pub fn new(problem: &Problem) -> Self {
        let size = problem.number_of_machines() * problem.number_of_jobs() + 1;
        let matrix: Vec<f64> = vec![0.0; size * size];
        Self { size, matrix }
    }

    fn index(&self, from: usize, to: usize) -> usize {
        from * self.size + to
    }

    pub fn get(&self, from: usize, to: usize) -> &f64 {
        &self.matrix[self.index(to, from)]
    }

    pub fn set(&mut self, from: usize, to: usize, value: f64) {
        let index = self.index(to, from);
        self.matrix[index] = value;
    }

    pub fn update(&mut self, from: usize, to: usize, change: f64) {
        let index = self.index(from, to);
        let old_value = self.matrix[index];
        self.matrix[index] = old_value + change;
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn evaporate(&mut self) {
        let fraction_left = 1.0 - CONFIG.evaporation;
        self.matrix.par_iter_mut().for_each(|value| {
            *value = *value * fraction_left;
        });
    }
}

impl Node {
    pub fn new(job: usize, operation: usize) -> Self {
        Self { job, operation }
    }
}

impl Ant {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            fitness: None,
        }
    }

    pub fn initialize(path: Vec<usize>) -> Self {
        Self {
            path,
            fitness: None,
        }
    }

    pub fn sequence(&self, nodes: &Vec<Node>) -> Vec<usize> {
        let jobs: Vec<usize> = self
            .path
            .iter()
            .filter_map(|node_num| {
                let node = &nodes[*node_num];
                if node.job > 0 {
                    Some(node.job - 1)
                } else {
                    None
                }
            })
            .collect();
        jobs
    }

    pub fn fitness(&self) -> usize {
        if let Some(fitness) = self.fitness {
            fitness
        } else {
            panic!("No fitness calculated");
        }
    }

    pub fn set_fitness(&mut self, fitness: usize) {
        self.fitness = Some(fitness);
    }
}

impl Colony {
    pub fn new(problem: &Problem) -> Self {
        let ants: Vec<Ant> = Vec::with_capacity(CONFIG.colony_size);
        let pheromones = Pheromone::new(problem);
        Self {
            ants,
            pheromones,
            nodes: Self::generate_nodes(problem),
            best_ant: None,
        }
    }

    pub fn reset(&mut self, best_ant: Ant) {
        // Evaporate
        self.pheromones.evaporate();

        // Update pheromenes based on current best
        // let best_ant = self.best_ant();
        let pheromone_strength = CONFIG.pheromone_strength;
        let mut current_node = best_ant.path[0];
        for i in 1..best_ant.path.len() {
            let new_node = best_ant.path[i];
            self.pheromones
                .update(current_node, new_node, pheromone_strength);
            current_node = new_node;
        }

        // Update fields
        self.ants = Vec::with_capacity(CONFIG.colony_size);
        self.best_ant = None;
    }

    fn generate_nodes(problem: &Problem) -> Vec<Node> {
        let mut nodes: Vec<Node> = Vec::new();
        nodes.push(Node::new(0, 0));
        for job_number in 0..problem.number_of_jobs() {
            let job = problem.job(job_number);
            for operation in job.operations.iter() {
                nodes.push(Node::new(operation.job_number, operation.part_number));
            }
        }
        nodes
    }

    fn edges(&self, node: usize) -> Vec<usize> {
        let from_node = &self.nodes[node];

        // If node 0, only first operation for each job is allowed
        let edges: Vec<usize> = if node == 0 {
            self.nodes
                .iter()
                .enumerate()
                .filter_map(|(i, to_node)| {
                    // First operation
                    if to_node.operation == 1 {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            self.nodes
                .iter()
                .enumerate()
                .filter_map(|(i, to_node)| {
                    if to_node.job > 0 {
                        // All other jobs
                        if to_node.job != from_node.job {
                            Some(i)
                        // Next operation in current job
                        } else if to_node.job == from_node.job
                            && to_node.operation - 1 == from_node.operation
                        {
                            Some(i)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        };
        edges
    }

    fn validate_edges(&self, edges: &mut Vec<usize>, visted: &Vec<bool>) {
        // Remove all jobs where some previous operation has
        // not been visisted yet
        edges.retain(|node_number| {
            let current_node = &self.nodes[*node_number];
            if visted[*node_number] {
                return false;
            }
            let mut valid = true;
            for (i, v) in visted.iter().enumerate() {
                if *v {
                    continue;
                } else {
                    let visited_node = &self.nodes[i];
                    if current_node.job == visited_node.job
                        && current_node.operation > visited_node.operation
                    {
                        valid = false;
                        break;
                    }
                }
            }
            valid
        });
    }

    pub fn create_ant(&mut self) -> Ant {
        let num_nodes = self.nodes.len();
        let mut visited: Vec<bool> = vec![false; num_nodes];
        let mut path: Vec<usize> = Vec::with_capacity(num_nodes);

        // Start at node 0
        let mut current_node = 0;
        visited[current_node] = true;
        path.push(current_node);

        while path.len() < num_nodes {
            let mut possible_nodes = self.edges(current_node);
            self.validate_edges(&mut possible_nodes, &visited);

            let pheromones: Vec<f64> = possible_nodes
                .iter()
                .map(|to_node| *self.pheromones.get(current_node, *to_node))
                .collect();

            let total_pheromone: f64 = pheromones.iter().sum();

            let probabilites: Vec<f64> = pheromones
                .iter()
                .map(|pheromone| pheromone / total_pheromone)
                .collect();

            let next_index = utils::sample(probabilites);
            let next_node = possible_nodes[next_index];
            current_node = next_node;
            path.push(next_node);
            visited[next_node] = true;
        }

        Ant::initialize(path)
    }

    pub fn set_best(&mut self) {
        for new_ant in self.ants.iter() {
            if let Some(best_ant) = self.best_ant.as_ref() {
                if new_ant.fitness() < best_ant.fitness() {
                    self.best_ant = Some(new_ant.clone());
                }
            } else {
                self.best_ant = Some(new_ant.clone());
            }
        }
    }

    pub fn best_ant(&self) -> Ant {
        if let Some(best_ant) = self.best_ant.as_ref() {
            best_ant.clone()
        } else {
            panic!("No best ant found");
        }
    }
}

impl ACO {
    pub fn new(problem: &Problem) -> Self {
        let colony = Colony::new(problem);
        Self {
            colony,
            best_ant: None,
            iteration: 0,
        }
    }

    pub fn initialize(&mut self, problem: &Problem) {
        // Init pheromones
        for i in 0..self.colony.nodes.len() {
            let to_nodes = self.colony.edges(i);
            for j in to_nodes {
                self.colony.pheromones.set(i, j, CONFIG.pheromone_init);
            }
        }

        for _ in 0..CONFIG.colony_size {
            let mut new_ant = self.colony.create_ant();
            let fitness = problem.calc_fitness(&new_ant.sequence(&self.colony.nodes));
            new_ant.set_fitness(fitness);
            self.colony.ants.push(new_ant);
        }
        self.colony.set_best();
        self.set_best();
    }

    pub fn local_search(&mut self, problem: &Problem, steps: usize) {
        let mut best = self.best_ant();
        let mut search_iteration: usize = 0;
        let mut new_best = self.best_ant();
        let length = best.path.len();

        for _ in 0..steps {
            search_iteration = search_iteration + 1;
            println!(
                "Local search iteration {}, fitness: {}",
                search_iteration,
                best.fitness()
            );

            for i in 0..length {
                for j in (i + 1)..length {
                    let mut new_ant = best.clone();
                    new_ant.path.swap(i, j);
                    let sequence = new_ant.sequence(&self.colony.nodes);
                    let fitness = problem.calc_fitness(&sequence);
                    new_ant.set_fitness(fitness);
                    if new_ant.fitness() < new_best.fitness() {
                        new_best = new_ant;
                    }
                }
            }

            if new_best.fitness() < best.fitness() {
                best = new_best.clone();
            } else {
                break;
            }
        }

        self.best_ant = Some(best);
    }

    pub fn iterate(&mut self, problem: &Problem) {
        self.colony.reset(self.best_ant());
        self.iteration = self.iteration + 1;
        for _ in 0..CONFIG.colony_size {
            let mut new_ant = self.colony.create_ant();
            let fitness = problem.calc_fitness(&new_ant.sequence(&self.colony.nodes));
            new_ant.set_fitness(fitness);
            self.colony.ants.push(new_ant);
        }
        self.colony.set_best();
        self.set_best();
    }

    pub fn print_iteration(&self) {
        let best_ant = self.best_ant();
        let mut sum: usize = 0;
        for ant in self.colony.ants.iter() {
            sum = sum + ant.fitness();
        }

        let average: f64 = sum as f64 / self.colony.ants.len() as f64;
        println!(
            "Iteration {}, best: {}, average: {:.2}",
            self.iteration,
            best_ant.fitness(),
            average
        )
    }

    pub fn set_best(&mut self) {
        let new_ant = self.colony.best_ant();
        if let Some(best_ant) = self.best_ant.as_ref() {
            if new_ant.fitness() < best_ant.fitness() {
                self.best_ant = Some(new_ant.clone());
            }
        } else {
            self.best_ant = Some(new_ant.clone());
        }
    }

    pub fn best_ant(&self) -> Ant {
        if let Some(best_ant) = self.best_ant.as_ref() {
            best_ant.clone()
        } else {
            panic!("No best ant found");
        }
    }
}
