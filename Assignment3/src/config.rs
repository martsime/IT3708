use envconfig::Envconfig;
use lazy_static::*;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "DATA_NUMBER", default = "7")]
    pub data_number: usize,

    #[envconfig(from = "IMAGE_WIDTH", default = "1200")]
    pub image_width: i32,

    #[envconfig(from = "IMAGE_HEIGHT", default = "1200")]
    pub image_height: i32,

    #[envconfig(from = "PADDING", default = "100")]
    pub padding: i32,

    #[envconfig(from = "IMAGE_PATH", default = "")]
    pub image_path: String,

    // Partical Swarm Optimization (PSO) settings
    #[envconfig(from = "ITERATIONS", default = "0")]
    pub iterations: usize,

    #[envconfig(from = "LS_N", default = "1000000")] // Local search every n iterations
    pub ls_n: usize,

    #[envconfig(from = "LS_STEPS", default = "1000")]
    pub ls_steps: usize,

    #[envconfig(from = "SWARM_SIZE", default = "10")]
    pub swarm_size: usize,

    #[envconfig(from = "X_MIN", default = "0.0")]
    pub x_min: f64,

    #[envconfig(from = "X_MAX", default = "2.0")]
    pub x_max: f64,

    #[envconfig(from = "V_MIN", default = "-2.0")]
    pub v_min: f64,

    #[envconfig(from = "V_MAX", default = "2.0")]
    pub v_max: f64,

    #[envconfig(from = "C_1", default = "2.0")]
    pub c_1: f64,

    #[envconfig(from = "C_2", default = "2.0")]
    pub c_2: f64,

    #[envconfig(from = "W_START", default = "0.9")]
    pub w_start: f64,

    #[envconfig(from = "W_MIN", default = "0.4")]
    pub w_min: f64,
}

impl Config {
    pub fn problem_path(&self) -> String {
        format!("data/{}.txt", self.data_number)
    }

    pub fn image_path(&self) -> String {
        format!("image-{}.png", self.data_number)
    }

    pub fn get_inertia(&self, iteration: usize) -> f64 {
        self.w_start - (self.w_start - self.w_min) * (iteration as f64 / self.iterations as f64)
    }

    pub fn get_image_size(&self) -> (i32, i32) {
        (self.image_width, self.image_height)
    }

    pub fn get_padding(&self) -> (i32, i32) {
        (self.padding, self.padding)
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init().unwrap();
}
