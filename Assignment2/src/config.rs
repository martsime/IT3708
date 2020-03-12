use envconfig::Envconfig;
use lazy_static::*;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "IMAGE_NUMBER", default = "216066")]
    image_number: String,

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

    #[envconfig(from = "IMAGE_FOLDER", default = "training")]
    image_folder: String,

    #[envconfig(from = "OUT_FOLDER", default = "evaluator/my_out")]
    out_path: String,

    #[envconfig(from = "MIN_SEG_SIZE", default = "200")]
    pub min_seg_size: usize,

    #[envconfig(from = "MAX_SEGMENTS", default = "10000")]
    pub max_segments: usize,

    #[envconfig(from = "THREADS", default = "12")]
    pub threads: usize,

    #[envconfig(from = "POPULATION_SIZE", default = "20")]
    pub population_size: usize,

    #[envconfig(from = "GENERATIONS", default = "10")]
    pub generations: usize,

    #[envconfig(from = "KMEANS", default = "10")]
    pub kmeans: usize,

    #[envconfig(from = "CROSSOVER_SEG_MAX", default = "2")]
    pub crossover_seg_max: usize,

    #[envconfig(from = "CROSSOVER_RATE", default = "1.0")]
    pub crossover_rate: f64,

    #[envconfig(from = "MUTATION_RATE", default = "0.5")]
    pub mutation_rate: f64,

    #[envconfig(from = "MUTATIONS_MAX", default = "20")]
    pub mutations_max: usize,

    #[envconfig(from = "WEIGHTED", default = "false")]
    pub weighted: bool,

    #[envconfig(from = "EV_WEIGHT", default = "0.1")]
    pub ev_weight: f64,

    #[envconfig(from = "CON_WEIGTH", default = "0.1")]
    pub con_weight: f64,

    #[envconfig(from = "OD_WEIGHT", default = "0.1")]
    pub od_weight: f64,

    #[envconfig(from = "TOURNAMENT_K", default = "2")]
    pub tournament_k: usize,
}

impl Config {
    pub fn plot_size(&self) -> (i32, i32) {
        (self.plot_size, self.plot_size * 2 / 3)
    }

    pub fn out_path(&self) -> String {
        format!("{}/{}", self.out_path, self.image_number)
    }

    pub fn image_path(&self) -> String {
        format!("{}/{}/Test image.jpg", self.image_folder, self.image_number)
    }

    pub fn total_weight(&self) -> f64 {
        self.ev_weight + self.con_weight + self.od_weight
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init().unwrap();
}
