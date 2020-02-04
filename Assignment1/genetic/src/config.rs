use envconfig::Envconfig;
use lazy_static::*;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "PROBLEM_PATH", default = "")]
    pub problem_path: String,

    #[envconfig(from = "OPTIMAL_SOLUTION_PATH", default = "")]
    pub optimal_solution_path: String,

    #[envconfig(from = "SOLUTION_PATH", default = "")]
    pub solution_path: String,

    #[envconfig(from = "LOAD_SOLUTION", default = "false")]
    pub load_solution: bool,

    #[envconfig(from = "SHOW_SOLUTION", default = "false")]
    pub show_solution: bool,

    #[envconfig(from = "SHOW_OPTIMAL_SOLUTION", default = "false")]
    pub show_optimal_solution: bool,

    #[envconfig(from = "POPULATION_SIZE", default = "50")]
    pub population_size: usize,

    #[envconfig(from = "POPULATION_GEN_STEP", default = "50")]
    pub population_gen_step: usize,

    #[envconfig(from = "GENERATIONS", default = "1000")]
    pub generations: usize,

    #[envconfig(from = "DRAW_RATE", default = "1")]
    pub draw_rate: i32,

    #[envconfig(from = "VERBOSE", default = "false")]
    pub verbose: bool,

    #[envconfig(from = "ELITE_COUNT", default = "2")]
    pub elite_count: usize,

    #[envconfig(from = "SINGLE_SWAP_MUT_RATE", default = "0.05")]
    pub single_swap_mut_rate: f64,

    #[envconfig(from = "SINGLE_SWAP_MUT_MAX", default = "2")]
    pub single_swap_mut_max: usize,

    #[envconfig(from = "VEHICLE_REMOVE_MUT_RATE", default = "0.05")]
    pub vehicle_remove_mut_rate: f64,

    #[envconfig(from = "VEHICLE_REMOVE_MUT_MAX", default = "1")]
    pub vehicle_remove_mut_max: usize,

    #[envconfig(from = "CROSSOVER_RATE", default = "1.0")]
    pub crossover_rate: f64,

    #[envconfig(from = "PARENT_SELECTION_K", default = "5")]
    pub parent_selection_k: usize,

    #[envconfig(from = "INFEASIBILITY_PENALTY", default = "1000")]
    pub infeasibility_penalty: i32,

    #[envconfig(from = "CWS_BIAS", default = "10")]
    pub cws_bias: usize,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init().unwrap();
}
