use rand::Rng;
use rayon::prelude::*;
use std::cmp::Ordering;

use crate::config::CONFIG;
use crate::problem::Problem;
use crate::utils;

#[derive(Debug, Clone)]
pub struct ParticleItem {
    position: f64,
    velocity: f64,
    job: usize,
}

#[derive(Debug, Clone)]
pub struct Particle {
    items: Vec<ParticleItem>,
    sequence: Option<Vec<usize>>,
    pub number: usize,
    pub fitness: Option<usize>,
    pub local_best: Option<Box<Self>>,
}

#[derive(Debug, Clone)]
pub struct Swarm {
    pub particles: Vec<Particle>,
    pub global_best: Option<Particle>,
}

#[derive(Debug, Clone)]
pub struct PSO {
    pub swarm: Swarm,
    pub iteration: usize,
}

impl PartialOrd for ParticleItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position.partial_cmp(&other.position)
    }
}

impl PartialEq for ParticleItem {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Particle {
    pub fn new(jobs: &Vec<usize>, number: usize) -> Self {
        let items: Vec<ParticleItem> = jobs
            .iter()
            .map(|job| ParticleItem {
                position: utils::get_new_position(),
                velocity: utils::get_new_velocity(),
                job: *job,
            })
            .collect();
        Self {
            items,
            number,
            sequence: None,
            fitness: None,
            local_best: None,
        }
    }

    pub fn get_sequence(&self) -> &Vec<usize> {
        if let Some(sequence) = self.sequence.as_ref() {
            sequence
        } else {
            panic!("Sequence is not calculated");
        }
    }

    pub fn generate_sequence(&mut self) {
        let mut cloned_items = self.items.clone();
        cloned_items.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let sequence: Vec<usize> = cloned_items
            .iter()
            .map(|particle_item| particle_item.job)
            .collect();
        self.sequence = Some(sequence);
    }

    pub fn get_fitness(&self) -> usize {
        if let Some(fitness) = self.fitness {
            fitness
        } else {
            panic!("Fitness is not calculated");
        }
    }

    pub fn update_local_best(&mut self) {
        if let Some(local_best) = self.local_best.as_ref() {
            if self.get_fitness() < local_best.get_fitness() {
                self.local_best = Some(Box::new(self.clone()));
            }
        } else {
            self.local_best = Some(Box::new(self.clone()));
        }
    }

    fn get_local_best(&self) -> Particle {
        if let Some(local_best) = self.local_best.as_ref() {
            local_best.as_ref().clone()
        } else {
            panic!("No local best!");
        }
    }

    pub fn update(&mut self, global_best: &Particle, inertia: f64) {
        let local_best = self.get_local_best();

        let mut rng = rand::thread_rng();
        for i in 0..self.items.len() {
            let mut item = &mut self.items[i];
            let local_item = &local_best.items[i];
            let global_item = &global_best.items[i];

            let part_one: f64 = inertia * item.velocity;
            let part_two: f64 =
                CONFIG.c_1 * rng.gen::<f64>() * (local_item.position - item.position);
            let part_three: f64 =
                CONFIG.c_2 * rng.gen::<f64>() * (global_item.position - item.position);

            let new_velocity = part_one + part_two + part_three;
            let new_velocity = utils::validate_velocity(new_velocity);
            item.velocity = new_velocity;
            item.position = item.position + item.velocity;
        }
    }
}

impl Swarm {
    pub fn new(problem: &Problem) -> Self {
        let mut jobs = problem.get_job_operations();
        let mut particles: Vec<Particle> = Vec::new();
        for i in 0..CONFIG.swarm_size {
            utils::shuffle_vec(&mut jobs);
            particles.push(Particle::new(&jobs, i));
        }
        Self {
            particles,
            global_best: None,
        }
    }

    pub fn neighborhood(particle: &Particle) -> Self {
        let mut particles: Vec<Particle> = Vec::new();
        particles.push(particle.clone());
        for i in 0..particle.items.len() {
            for j in i..particle.items.len() {
                if i == j {
                    continue;
                }
                let mut new_particle = particle.clone();
                let tmp = new_particle.items[i].clone();

                // Swap positions
                new_particle.items[i].position = new_particle.items[j].position;
                new_particle.items[j].position = tmp.position;
                new_particle.items[i].velocity = new_particle.items[j].velocity;
                new_particle.items[j].velocity = tmp.velocity;
                particles.push(new_particle);
            }
        }
        Self {
            particles,
            global_best: None,
        }
    }

    pub fn generate_sequences(&mut self) {
        self.particles.par_iter_mut().for_each(|particle| {
            particle.generate_sequence();
        });
    }

    pub fn evaluate(&mut self, problem: &Problem) {
        self.particles.par_iter_mut().for_each(|particle| {
            let sequence = particle.get_sequence();
            let fitness = problem.calc_fitness(&sequence);
            particle.fitness = Some(fitness);
        });
    }

    pub fn find_best(&mut self) {
        // Update global best
        let mut global_best: Option<&Particle> = None;
        for particle in self.particles.iter() {
            let fitness = particle.get_fitness();

            if let Some(global) = global_best.as_ref() {
                if fitness < global.get_fitness() {
                    global_best = Some(particle);
                }
            } else {
                global_best = Some(particle);
            }
        }
        if let Some(new_best) = global_best {
            if let Some(old_best) = self.global_best.as_ref() {
                if new_best.get_fitness() < old_best.get_fitness() {
                    self.global_best = Some(new_best.clone())
                }
            } else {
                self.global_best = Some(new_best.clone());
            }
        }

        // Update local best
        self.particles.par_iter_mut().for_each(|particle| {
            particle.update_local_best();
        });
    }

    pub fn get_global_best(&self) -> Particle {
        if let Some(global_best) = self.global_best.as_ref() {
            global_best.clone()
        } else {
            panic!("No global best!");
        }
    }

    pub fn update(&mut self, inertia: f64) {
        let global_best = self.get_global_best();
        self.particles.par_iter_mut().for_each(|particle| {
            particle.update(&global_best, inertia);
        });
    }
}

impl PSO {
    pub fn new(problem: &Problem) -> Self {
        Self {
            swarm: Swarm::new(problem),
            iteration: 0,
        }
    }

    pub fn initialize(&mut self, problem: &Problem) {
        self.swarm.generate_sequences();
        self.swarm.evaluate(problem);
        self.swarm.find_best();
    }

    pub fn iterate(&mut self, problem: &Problem) {
        self.iteration = self.iteration + 1;
        let inertia = CONFIG.get_inertia(self.iteration);
        self.swarm.update(inertia);
        self.swarm.generate_sequences();
        self.swarm.evaluate(problem);
        self.swarm.find_best();
    }

    pub fn local_search(&mut self, problem: &Problem, steps: usize) {
        let mut best = self.swarm.get_global_best();
        let mut search_iteration: usize = 0;
        let mut new_best = self.swarm.get_global_best();
        for _ in 0..steps {
            search_iteration = search_iteration + 1;
            println!(
                "Local search iteration {}, fitness: {}",
                search_iteration,
                best.get_fitness()
            );
            let mut new_swarm = Swarm::neighborhood(&new_best);
            new_swarm.generate_sequences();
            new_swarm.evaluate(problem);
            new_swarm.find_best();
            new_best = new_swarm.get_global_best();
            if new_best.get_fitness() < best.get_fitness() {
                best = new_best.clone();
            } else {
                break;
            }
        }
        best.update_local_best();
        self.swarm.particles[best.number] = best.clone();
        self.swarm.find_best();
    }

    pub fn print_iteration(&self) {
        let global_best = self.swarm.get_global_best();
        let mut average_local: usize = 0;
        for particle in self.swarm.particles.iter() {
            average_local = average_local + particle.get_local_best().get_fitness();
        }
        let average: f64 = average_local as f64 / CONFIG.swarm_size as f64;
        println!(
            "Iteration {}, best: {}, average: {:.2}",
            self.iteration,
            global_best.get_fitness(),
            average,
        );
        /*

        for (i, particle) in self.swarm.particles.iter().enumerate() {
            let mut positions: String = String::from("[ ");
            let mut velocities: String = String::from("[ ");
            println!("Particle {}: fitness: {}", i, particle.get_fitness());
            for item in particle.items.iter() {
                positions.push_str(&format!("{:.2} ", item.position));
                velocities.push_str(&format!("{:.2} ", item.velocity));
            }
            println!("Position: {}]", positions);
            println!("Velocity: {}]", velocities);
        }
        */
    }
}
