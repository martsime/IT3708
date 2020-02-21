extern crate gio;
extern crate gtk;

mod app;

use app::App;

fn main() {
    App::new().build().run();
}
