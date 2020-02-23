#[macro_use]
extern crate envconfig_derive;

extern crate gio;
extern crate gtk;

mod app;
mod config;
mod gui;
mod kmeans;
mod matrix;
mod segment;
mod simulation;
mod utils;

use app::App;

fn main() {
    App::new().build().run();
}
