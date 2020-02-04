extern crate genetic;

use std::time::Instant;

use genetic::config::CONFIG;
use genetic::problem::Problem;

fn time_method<F: FnMut()>(name: &str, mut f: F) {
    let start_time = Instant::now();
    f();
    println!(
        "Method \"{}\" took {} ms",
        name,
        start_time.elapsed().as_millis()
    );
}

fn main() {
    let problem_path = CONFIG.problem_path.clone();
    let mut problem = Problem::new(problem_path);
    time_method("generate_population", || {
        problem.generate_population();
    });
    time_method("simulate", || {
        for _ in 0..1000 {
            problem.simulate();
        }
    });
}
