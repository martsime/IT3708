#[macro_use]
extern crate envconfig_derive;

extern crate gio;
extern crate gtk;

mod app;
mod config;
mod kmeans;

use app::App;

fn main() {
    App::new().build().run();
}
