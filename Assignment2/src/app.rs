use gio::prelude::*;
use gtk::prelude::*;

use gdk_pixbuf::{Colorspace, Pixbuf};

use std::thread;
use std::time::Duration;

use crate::config::CONFIG;

pub struct App {
    gui: gtk::Application,
}

struct Worker {
    image: image::RgbImage,
}

impl Worker {
    pub fn new() -> Self {
        let image: image::RgbImage = match image::open(&CONFIG.image_path) {
            Ok(image) => image.into_rgb(),
            Err(_) => panic!("Unable to load image!"),
        };
        println!("Loaded image!");

        println!("Image size: {:?}", image.dimensions());

        Worker { image: image }
    }

    pub fn get_image(&mut self) -> image::RgbImage {
        let mut rng = rand::thread_rng();
        let (width, height) = self.image.dimensions();

        /*
        for _ in 0..1000 {
            let x = rng.gen_range(0, width);
            let y = rng.gen_range(0, height);

            self.image.put_pixel(x, y, image::Rgb([0, 0, 0]));
        }
        */

        crate::kmeans::kmeans(&self.image, 20)
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

            let flowbox = gtk::FlowBox::new();
            flowbox.set_selection_mode(gtk::SelectionMode::None);
            let grid = gtk::Grid::new();

            let mut images: Vec<gtk::Image> = Vec::new();

            let cols = 5;
            for i in 0..25 {
                let pixelb = Pixbuf::new_from_file_at_size(
                    &CONFIG.image_path,
                    CONFIG.image_size,
                    CONFIG.image_size,
                );
                let gtk_image = match pixelb {
                    Ok(buf) => gtk::Image::new_from_pixbuf(Some(&buf)),
                    Err(_) => {
                        panic!("Failed to load pixelbuffer");
                    }
                };

                let event_box = gtk::EventBox::new();
                event_box.add(&gtk_image);

                event_box.connect_button_press_event(|_image, _event| {
                    println!("Pressed image");
                    gtk::Inhibit(false)
                });

                images.push(gtk_image.clone());

                let label = gtk::Label::new(Some(&format!("Child: {}", i + 1)));

                let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
                gtk_box.add(&label);
                gtk_box.add(&event_box);

                let row = i / cols;
                let col = i % cols;
                println!("Attach at: ({}, {})", row, col);
                grid.attach(&gtk_box, col, row, 1, 1);

                println!("Row spacing: {}", flowbox.get_row_spacing());
                println!("Size: {:?}", gtk_box.get_size_request());
            }

            window.add(&grid);

            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            thread::spawn(move || {
                let mut worker = Worker::new();
                loop {
                    thread::sleep(Duration::from_millis(10));
                    let image = worker.get_image();
                    tx.send(image).expect("Failed to send");
                }
            });

            let mut num = 0;
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
                println!("Length: {}", images.len());
                let gtk_image = &images[num];
                let old_pixel_buf = gtk_image.get_pixbuf().unwrap();
                let (display_width, display_height) =
                    (old_pixel_buf.get_width(), old_pixel_buf.get_height());
                let scaled_pixbuf = pixbuf
                    .scale_simple(
                        display_width,
                        display_height,
                        gdk_pixbuf::InterpType::Bilinear,
                    )
                    .expect("Failed to scale");

                gtk_image.set_from_pixbuf(Some(&scaled_pixbuf));
                num = (num + 1) % 25;

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
