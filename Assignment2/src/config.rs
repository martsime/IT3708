use envconfig::Envconfig;
use lazy_static::*;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "IMAGE_SIZE", default = "300")]
    pub image_size: i32,

    #[envconfig(from = "IMAGE_PATH", default = "training/147091/Test image.jpg")]
    pub image_path: String,

    #[envconfig(from = "IMAGE_ROWS", default = "4")]
    pub image_rows: usize,

    #[envconfig(from = "IMAGE_COLS", default = "4")]
    pub image_cols: usize,

    #[envconfig(from = "MIN_SEG_SIZE", default = "50")]
    pub min_seg_size: usize,

    #[envconfig(from = "THREADS", default = "12")]
    pub threads: usize,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init().unwrap();
}
