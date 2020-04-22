#[macro_use]
extern crate envconfig_derive;

mod ant;
mod config;
mod image;
mod parser;
mod particle;
mod problem;
mod utils;

use ant::ACO;
use config::{Method, CONFIG};
use image::draw_image;
use particle::PSO;
use problem::Problem;

fn main() {
    println!("File at: {}", CONFIG.problem_path());
    let problem = Problem::from_file(&CONFIG.problem_path());

    match CONFIG.method {
        Method::PSO => {
            let mut pso = PSO::new(&problem);
            pso.initialize(&problem);
            pso.print_iteration();

            for i in 1..(CONFIG.swarm_iterations + 1) {
                pso.iterate(&problem);
                if i % CONFIG.ls_n == 0 && i < CONFIG.swarm_iterations {
                    pso.local_search(&problem, CONFIG.ls_steps);
                }
                pso.print_iteration();
            }
            pso.local_search(&problem, 1000);
            let global_best = pso.swarm.get_global_best();
            let fitness = global_best.get_fitness();
            let sequence = global_best.get_sequence();
            draw_image(sequence, fitness, &problem);
        }
        Method::ACO => {
            let mut aco = ACO::new(&problem);
            aco.initialize(&problem);
            for _ in 1..(CONFIG.ant_iterations + 1) {
                aco.iterate(&problem);
                aco.print_iteration();
            }
            aco.local_search(&problem, 1000);
            let best = aco.best_ant();
            let fitness = best.fitness();
            let sequence = best.sequence(&aco.colony.nodes);

            draw_image(&sequence, fitness, &problem);
        }
    }
}
