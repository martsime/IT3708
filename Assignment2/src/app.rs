use gio::prelude::*;
use gtk::prelude::*;

use gdk_pixbuf::Pixbuf;

use std::thread;
use std::time::Duration;

pub struct App {
    gui: gtk::Application,
}

impl App {
    pub fn new() -> App {
        let application = gtk::Application::new(
            Some("com.github.martsime.IT3708.assignment2"),
            Default::default(),
        )
        .expect("Initialization failed...");
        App { gui: application }
    }

    pub fn build(self) -> Self {
        self.gui.connect_activate(|app| {
            let window = gtk::ApplicationWindow::new(app);

            window.set_title("");
            window.set_border_width(10);
            window.set_position(gtk::WindowPosition::Center);
            window.set_default_size(1000, 1000);

            let pixelb = Pixbuf::new_from_file_at_size("training/86016/Test image.jpg", 400, 400);

            let image = match pixelb {
                Ok(buf) => gtk::Image::new_from_pixbuf(Some(&buf)),
                Err(_) => {
                    panic!("Failed to load pixelbuffer");
                }
            };

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

            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            thread::spawn(move || {
                let mut i = 0;
                loop {
                    thread::sleep(Duration::from_millis(1000));
                    i += 1;
                    tx.send(i).expect("Could not send on channel!");
                }
            });

            rx.attach(None, move |value| {
                println!("Value: {}", value);
                glib::Continue(true)
            });

            println!("{:?}", window.get_size());

            window.show_all();
        });
        self
    }

    pub fn run(self) -> Self {
        self.gui.run(&[]);
        self
    }
}
