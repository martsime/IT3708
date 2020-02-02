#[cfg(test)]
#[macro_use]
extern crate approx;

#[macro_use]
extern crate envconfig_derive;

mod config;
mod heuristic;
mod parser;
mod problem;
mod simulation;
mod solution;
mod utils;

use std::collections::HashMap;

use config::CONFIG;
use problem::Problem;
use pyo3::prelude::*;

#[pyclass(module = "genetic")]
struct GeneticProgram {
    problem: Problem,
}

#[pymethods]
impl GeneticProgram {
    #[new]
    fn new(obj: &PyRawObject) {
        let problem_path = CONFIG.problem_path.clone();
        let mut problem = Problem::new(problem_path);
        if CONFIG.load_solution {
            let solution_path = CONFIG.solution_path.clone();
            problem.load_optimal_solution(solution_path);
        }
        obj.init(GeneticProgram { problem });
    }

    fn get_customers(&self) -> PyResult<HashMap<i32, (i32, i32)>> {
        Ok(self.problem.get_customers())
    }

    fn get_depots(&self) -> PyResult<HashMap<i32, (i32, i32)>> {
        Ok(self.problem.get_depots())
    }

    fn get_boundaries(&self) -> PyResult<(i32, i32, i32, i32)> {
        Ok(self.problem.get_boundaries())
    }

    fn generate_population(&mut self) {
        self.problem.generate_population();
    }

    fn simulate(&mut self) -> PyResult<Vec<Vec<i32>>> {
        let solution = self.problem.simulate();
        Ok(solution.routes)
    }
}

#[pymodule]
fn genetic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GeneticProgram>()?;
    Ok(())
}
