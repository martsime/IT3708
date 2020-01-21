#[cfg(test)]
#[macro_use]
extern crate approx;

mod parser;
mod problem;
mod simulation;
mod solution;
mod utils;

use std::collections::HashMap;

use problem::Problem;
use pyo3::prelude::*;

#[pyclass(module = "genetic")]
struct GeneticProgram {
    problem: Problem,
}

#[pymethods]
impl GeneticProgram {
    #[new]
    fn new(obj: &PyRawObject, problem_path: String, optimal_solution_path: String) {
        let mut problem = Problem::new(problem_path);
        problem.load_optimal_solution(optimal_solution_path);
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
