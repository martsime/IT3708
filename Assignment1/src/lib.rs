#[macro_use]
extern crate envconfig_derive;

extern crate genetic;

use envconfig::Envconfig;

use std::collections::HashMap;

use genetic::config::Config;
use genetic::problem::Problem;
use pyo3::prelude::*;

#[pyclass(module = "pygenetic")]
struct GeneticProgram {
    problem: Problem,
    config: Config,
}

#[pymethods]
impl GeneticProgram {
    #[new]
    fn new(obj: &PyRawObject) {
        let config: Config = Config::init().unwrap();
        let mut problem = Problem::new(&config);
        if config.load_solution {
            let optimal_solution_path = config.optimal_solution_path.clone();
            problem.load_optimal_solution(optimal_solution_path);
        }
        obj.init(GeneticProgram { problem, config });
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
        self.problem.generate_population(&self.config);
    }

    fn get_solution_fitness(&self) -> PyResult<f64> {
        Ok(self.problem.get_solution().score())
    }

    fn simulate(&mut self) -> PyResult<Vec<Vec<i32>>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let solution = py.allow_threads(|| self.problem.simulate(&self.config));
        Ok(solution.routes)
    }

    fn update_config(&mut self) {
        println!("Crossover: {}", self.config.single_swap_mut_rate);
    }
}

#[pymodule]
fn pygenetic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GeneticProgram>()?;
    Ok(())
}
