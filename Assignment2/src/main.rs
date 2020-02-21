extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1000, 1000);

    let image = gtk::Image::new_from_file("training/86016/Test image.jpg");
    image.set_size_request(400, 400);

    let image2 = gtk::Image::new_from_file("training/86016/Test image.jpg");

    let text_buffer = gtk::EntryBuffer::new(Some("10"));
    let text_input = gtk::Entry::new_with_buffer(&text_buffer);
    text_input.set_size_request(175, 20);
    let space = gtk::Label::new(None);
    space.set_size_request(200, 10);

    let grid_layout = gtk::Grid::new();
    let button = gtk::Button::new_with_label("Load");
    button.set_size_request(25, 20);
    let text_buffer_clone = text_buffer.clone();
    button.connect_clicked(move |_| {
        println!("Hello world: {}", text_buffer_clone.get_text());
    });
    grid_layout.attach(&text_input, 0, 0, 1, 1);
    grid_layout.attach(&button, 1, 0, 1, 1);
    grid_layout.attach(&space, 0, 1, 2, 1);
    grid_layout.attach(&image, 0, 2, 2, 1);

    window.add(&grid_layout);

    println!("{:?}", window.get_size());

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.martsime.IT3708.assignment2"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&[]);
}
