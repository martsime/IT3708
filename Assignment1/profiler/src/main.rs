extern crate genetic;

use genetic::config::CONFIG;
use genetic::problem::Problem;

fn main() {
    let problem_path = CONFIG.problem_path.clone();
    let mut problem = Problem::new(problem_path);
    problem.generate_population();
}
