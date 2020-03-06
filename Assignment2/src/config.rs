use envconfig::Envconfig;
use lazy_static::*;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "GUI_WIDTH", default = "2000")]
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
    plot_size: i32,

    #[envconfig(from = "IMAGE_PATH", default = "training/176035/Test image.jpg")]
    pub image_path: String,

    #[envconfig(from = "MIN_SEG_SIZE", default = "100")]
    pub min_seg_size: usize,

    #[envconfig(from = "THREADS", default = "12")]
    pub threads: usize,

    #[envconfig(from = "POPULATION_SIZE", default = "36")]
    pub population_size: usize,

    #[envconfig(from = "KMEANS", default = "36")]
    pub kmeans: usize,

    #[envconfig(from = "CROSSOVER_SEG_MAX", default = "10")]
    pub crossover_seg_max: usize,
}

impl Config {
    pub fn plot_size(&self) -> (i32, i32) {
        (self.plot_size, self.plot_size * 2 / 3)
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init().unwrap();
}
