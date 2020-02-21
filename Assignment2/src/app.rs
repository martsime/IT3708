use gio::prelude::*;
use gtk::prelude::*;

use gdk_pixbuf::{Colorspace, Pixbuf};

use std::thread;
use std::time::Duration;

use rand::Rng;

pub struct App {
    gui: gtk::Application,
}

struct Worker {
    image: image::RgbImage,
}

impl Worker {
    pub fn new() -> Self {
        let image: image::RgbImage = match image::open("training/86016/Test image.jpg") {
            Ok(image) => image.into_rgb(),
            Err(_) => panic!("Unable to load image!"),
        };
        println!("Loaded image!");

        println!("Image size: {:?}", image.dimensions());

        for pixel in image.pixels() {
            println!("Pixel: {:?}", pixel);
        }

        Worker { image: image }
    }

    pub fn get_image(&mut self) -> image::RgbImage {
        let mut rng = rand::thread_rng();
        let (width, height) = self.image.dimensions();

        for _ in 0..1000 {
            let x = rng.gen_range(0, width);
            let y = rng.gen_range(0, height);

            self.image.put_pixel(x, y, image::Rgb([0, 0, 0]));
        }

        self.image.clone()
    }
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

            let gtk_image = match pixelb {
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
            grid_layout.attach(&gtk_image, 0, 2, 2, 1);

            window.add(&grid_layout);

            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            thread::spawn(move || {
                let mut worker = Worker::new();
                loop {
                    thread::sleep(Duration::from_millis(10));
                    let image = worker.get_image();
                    tx.send(image).expect("Failed to send");
                }
            });

            rx.attach(None, move |image| {
                let (width, height) = image.dimensions();
                let mut flattened = image.into_flat_samples();
                let raw_pixels: &mut [u8] = flattened.as_mut_slice();

                let pixbuf = Pixbuf::new_from_mut_slice(
                    raw_pixels,
                    Colorspace::Rgb,
                    false,
                    8,
                    width as i32,
                    height as i32,
                    width as i32 * 3,
                );

                gtk_image.set_from_pixbuf(Some(&pixbuf));

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
