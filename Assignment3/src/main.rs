#[macro_use]
extern crate envconfig_derive;

mod config;
mod image;
mod parser;
mod particle;
mod problem;
mod utils;

use config::CONFIG;
use image::draw_image;
use particle::PSO;
use problem::Problem;

fn main() {
    println!("File at: {}", CONFIG.problem_path());
    let problem = Problem::from_file(&CONFIG.problem_path());
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
    draw_image(&global_best, &problem);
}
