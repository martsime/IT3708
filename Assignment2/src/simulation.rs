use std::collections::HashSet;

use image::{Rgb, RgbImage};
use rand::Rng;

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
    pub segment_matrix: SegmentMatrix,
}

pub trait ImageFitness {
    fn calc_connectivity(&self) -> f64;
    fn calc_overall_deviation(&self, image: &RgbImage) -> f64;
    fn calc_edge_value(&self, image: &RgbImage) -> f64;
}

pub struct Population {
    pub individuals: Vec<Individual>,
}

pub struct Simulation {
    iteration: usize,
    pub population: Population,
}

impl Fitness {
    pub fn new(edge_value: f64, connectivity: f64, overall_deviation: f64) -> Fitness {
        Fitness {
            edge_value,
            connectivity,
            overall_deviation,
        }
    }
    pub fn dominates(&self, other: &Fitness) -> bool {
        let v1 = self.get_values();
        let v2 = other.get_values();
        let mut dominate = true;
        for i in 0..v1.len() {
            if v1[i] >= v2[i] {
                dominate = false;
                break;
            }
        }
        dominate
    }

    fn get_values(&self) -> [f64; 3] {
        [self.edge_value, self.connectivity, self.overall_deviation]
    }
}

impl ImageFitness for Individual {
    fn calc_connectivity(&self) -> f64 {
        let matrix = &self.segment_matrix.matrix;
        let mut fitness: f64 = 0.0;
        for y in 0..matrix.height {
            for x in 0..matrix.width {
                let pos = Pos::new_usize(y, x);
                let segment_number = matrix.get_pos(&pos);
                for neighbour_pos in matrix.get_neighbours(&pos).iter() {
                    let neighour_number = matrix.get_pos(neighbour_pos);
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
        let matrix = &self.segment_matrix.matrix;
        let mut fitness: f64 = 0.0;
        for y in 0..matrix.height {
            for x in 0..matrix.width {
                let pos = Pos::new_usize(y, x);
                let segment_number = matrix.get_pos(&pos);
                let pixel: &Rgb<u8> = image.get_pixel(pos.x as u32, pos.y as u32);
                for neighbour_pos in matrix.get_neighbours(&pos).iter() {
                    let neighour_number = matrix.get_pos(neighbour_pos);
                    if segment_number != neighour_number {
                        let neighbour_pixel =
                            image.get_pixel(neighbour_pos.x as u32, neighbour_pos.y as u32);
                        fitness += utils::pixel_distance(pixel, neighbour_pixel);
                    }
                }
            }
        }
        fitness * -1.0
    }
}

impl Clone for Individual {
    fn clone(&self) -> Individual {
        Individual::new(self.segment_matrix.clone())
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

    pub fn crossover(&self, other: &Individual) -> Individual {
        let mut new_individual = self.clone();
        let other_segments = other.segment_matrix.get_segments();

        let num_seg = std::cmp::min(CONFIG.crossover_seg_max, other_segments.len());
        let indices: HashSet<usize> = if num_seg == other_segments.len() {
            (0..other_segments.len()).into_iter().collect()
        } else {
            let mut rng = rand::thread_rng();
            let mut set = HashSet::new();
            while set.len() < num_seg {
                set.insert(rng.gen_range(0, other_segments.len()));
            }
            set
        };

        for i in indices.iter() {
            let segment = other_segments.get(*i);
            for pos in segment.positions.iter() {
                new_individual
                    .segment_matrix
                    .matrix
                    .set_at_pos(segment.number, pos);
            }
        }
        new_individual.segment_matrix.clean();
        new_individual
    }

    pub fn mutate(&mut self) {
        let segments = self.segment_matrix.get_segments();
        let mut rng = rand::thread_rng();
        let segment = segments.get(rng.gen_range(0, segments.len()));
        let neighbour_index = segment.get_dominant_neighbour(&self.segment_matrix, segments.len());
        let other_segment = segments.get(neighbour_index);
        self.segment_matrix.merge(&segment, &other_segment);
    }

    pub fn dominates(&self, other: &Individual) -> bool {
        self.get_fitness().dominates(other.get_fitness())
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

    pub fn get_fronts(&self) -> Vec<Vec<&Individual>> {
        // panic!("");
        let pop_size = self.individuals.len();
        let mut choosed: Vec<bool> = vec![false; self.individuals.len()];
        let mut fronts: Vec<Vec<&Individual>> = Vec::new();
        let mut count = 0;
        while count < pop_size {
            let mut front_indices: Vec<usize> = Vec::new();
            for i in 0..pop_size {
                if choosed[i] {
                    continue;
                }
                let individual = &self.individuals[i];
                let mut dominated = false;
                for j in 0..pop_size {
                    if i == j || choosed[j] {
                        continue;
                    }
                    let other = &self.individuals[j];
                    if other.dominates(individual) {
                        dominated = true;
                        break;
                    }
                }
                if !dominated {
                    front_indices.push(i);
                }
            }
            let front = front_indices
                .into_iter()
                .map(|index| {
                    count += 1;
                    choosed[index] = true;
                    &self.individuals[index]
                })
                .collect();
            fronts.push(front);
        }

        for (i, f) in fronts.iter().enumerate() {
            println!("Front: {}", i);
            for ind in f.iter() {
                let fitness = ind.get_fitness();
                println!(
                    "Fitness: {:.2}, {:.2}, {:.2}",
                    fitness.edge_value, fitness.connectivity, fitness.overall_deviation
                );
            }
            println!("");
        }
        fronts
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fitness_domination() {
        let f1 = Fitness::new(10.0, 10.0, 10.0);
        let f2 = Fitness::new(11.0, 10.0, 9.0);
        let f3 = Fitness::new(9.5, 9.5, 9.5);

        // f1 and f2 no domination
        assert_eq!(f1.dominates(&f2), false);
        assert_eq!(f2.dominates(&f1), false);

        // f3 dominates f1
        assert_eq!(f1.dominates(&f3), false);
        assert_eq!(f3.dominates(&f1), true);

        // f2 and f3 no domination
        assert_eq!(f2.dominates(&f3), false);
        assert_eq!(f3.dominates(&f2), false);
    }
}
