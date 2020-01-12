mod problem;
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
    fn new(obj: &PyRawObject, path: String) {
        let problem = Problem::new(path);
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
}

#[pymodule]
fn genetic(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GeneticProgram>()?;
    Ok(())
}
