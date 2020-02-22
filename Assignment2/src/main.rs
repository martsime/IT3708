extern crate gio;
extern crate gtk;

mod app;
mod kmeans;

use app::App;

fn main() {
    App::new().build().run();
}
