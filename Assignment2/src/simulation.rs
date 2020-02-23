use image::{Rgb, RgbImage};

use crate::config::CONFIG;
use crate::matrix::Pos;
use crate::segment::SegmentMatrix;
use crate::utils;

pub struct Fitness {
    pub edge_value: f64,
    pub connectivity: f64,
    pub overall_deviation: f64,
}

pub struct Individual {
    fitness: Option<Fitness>,
    segment_matrix: SegmentMatrix,
}

pub trait ImageFitness {
    fn calc_connectivity(&self) -> f64;
    fn calc_overall_deviation(&self, image: &RgbImage) -> f64;
    fn calc_edge_value(&self, image: &RgbImage) -> f64;
}

pub struct Population {
    individuals: Vec<Individual>,
}

pub struct Simulation {
    iteration: usize,
    pub population: Population,
}

impl ImageFitness for Individual {
    fn calc_connectivity(&self) -> f64 {
        let mut fitness: f64 = 0.0;
        for y in 0..self.segment_matrix.height {
            for x in 0..self.segment_matrix.width {
                let pos = Pos::new_usize(y, x);
                let segment_number = self.segment_matrix.get_pos(&pos);
                for neighbour_pos in self.segment_matrix.get_neighbours(&pos).iter() {
                    let neighour_number = self.segment_matrix.get_pos(neighbour_pos);
                    if segment_number != neighour_number {
                        fitness += 1.0 / 8.0;
                    }
                }
            }
        }
        fitness
    }

    fn calc_overall_deviation(&self, image: &RgbImage) -> f64 {
        let mut total_fitness: f64 = 0.0;
        for segment in self.segment_matrix.get_segments().iter() {
            let mut segment_fitness: f64 = 0.0;
            let centroid_pixel: Rgb<u8> = segment.get_pixel_centroid(image);
            for pos in segment.positions.iter() {
                let pixel: &Rgb<u8> = image.get_pixel(pos.x as u32, pos.y as u32);
                segment_fitness += utils::pixel_distance(&centroid_pixel, pixel);
            }
            total_fitness += segment_fitness;
        }
        total_fitness
    }

    fn calc_edge_value(&self, image: &RgbImage) -> f64 {
        let mut fitness: f64 = 0.0;
        for y in 0..self.segment_matrix.height {
            for x in 0..self.segment_matrix.width {
                let pos = Pos::new_usize(y, x);
                let segment_number = self.segment_matrix.get_pos(&pos);
                let pixel: &Rgb<u8> = image.get_pixel(pos.x as u32, pos.y as u32);
                for neighbour_pos in self.segment_matrix.get_neighbours(&pos).iter() {
                    let neighour_number = self.segment_matrix.get_pos(neighbour_pos);
                    if segment_number != neighour_number {
                        let neighbour_pixel =
                            image.get_pixel(neighbour_pos.x as u32, neighbour_pos.y as u32);
                        fitness += utils::pixel_distance(pixel, neighbour_pixel);
                    }
                }
            }
        }
        fitness
    }
}

impl Individual {
    pub fn new(segment_matrix: SegmentMatrix) -> Individual {
        Individual {
            fitness: None,
            segment_matrix: segment_matrix,
        }
    }

    pub fn evaluate(&mut self, image: &RgbImage) {
        let fitness = Fitness {
            connectivity: self.calc_connectivity(),
            overall_deviation: self.calc_overall_deviation(image),
            edge_value: self.calc_edge_value(image),
        };
        self.fitness = Some(fitness);
    }

    pub fn get_fitness(&self) -> &Fitness {
        match &self.fitness {
            Some(fitness) => fitness,
            None => {
                panic!("No fitness calculated yet!");
            }
        }
    }
}

impl Population {
    pub fn new() -> Population {
        Population {
            individuals: Vec::new(),
        }
    }

    pub fn add(&mut self, individual: Individual) {
        self.individuals.push(individual);
    }

    pub fn evaluate(&mut self, image: &RgbImage) {
        for (i, individual) in self.individuals.iter_mut().enumerate() {
            individual.evaluate(image);
            let fitness = individual.get_fitness();
            println!(
                "Individual: {} Fitness: {:.2}, {:.2}, {:.2}",
                i, fitness.edge_value, fitness.connectivity, fitness.overall_deviation
            );
        }
    }
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            population: Population::new(),
            iteration: 0,
        }
    }
    pub fn add_initial(&mut self, segment_matrices: Vec<SegmentMatrix>) {
        let num_segments = segment_matrices.len();
        for i in 0..CONFIG.population_size {
            let segment_matrix = segment_matrices[i % num_segments].clone();
            self.population.add(Individual::new(segment_matrix));
        }
    }
}
