use envconfig::Envconfig;
use lazy_static::*;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "DATA_NUMBER", default = "2")]
    pub data_number: usize,
}

impl Config {
    pub fn problem_path(&self) -> String {
        format!("data/{}.txt", self.data_number)
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init().unwrap();
}
