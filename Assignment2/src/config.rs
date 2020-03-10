use envconfig::Envconfig;
use lazy_static::*;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "GUI_WIDTH", default = "1200")]
    pub gui_width: i32,

    #[envconfig(from = "GUI_HEIGHT", default = "1000")]
    pub gui_height: i32,

    #[envconfig(from = "GUI_BORDER", default = "10")]
    pub gui_border: i32,

    #[envconfig(from = "ORIGINAL_IMAGE_SIZE", default = "500")]
    pub original_image_size: i32,

    #[envconfig(from = "IMAGE_SIZE", default = "300")]
    pub image_size: i32,

    #[envconfig(from = "PLOT_SIZE", default = "500")]
    pub plot_size: i32,

    #[envconfig(from = "IMAGE_PATH", default = "training/147091/Test image.jpg")]
    pub image_path: String,

    #[envconfig(from = "OUT_PATH", default = "evaluator/my_out")]
    pub out_path: String,

    #[envconfig(from = "MIN_SEG_SIZE", default = "50")]
    pub min_seg_size: usize,

    #[envconfig(from = "MAX_SEGMENTS", default = "1000")]
    pub max_segments: usize,

    #[envconfig(from = "THREADS", default = "12")]
    pub threads: usize,

    #[envconfig(from = "POPULATION_SIZE", default = "30")]
    pub population_size: usize,

    #[envconfig(from = "KMEANS", default = "30")]
    pub kmeans: usize,

    #[envconfig(from = "CROSSOVER_SEG_MAX", default = "10")]
    pub crossover_seg_max: usize,

    #[envconfig(from = "CROSSOVER_RATE", default = "1.0")]
    pub crossover_rate: f64,

    #[envconfig(from = "MUTATION_RATE", default = "0.5")]
    pub mutation_rate: f64,
}

impl Config {
    pub fn plot_size(&self) -> (i32, i32) {
        (self.plot_size, self.plot_size * 2 / 3)
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init().unwrap();
}
