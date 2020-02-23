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
        Worker { image: image }
    }

    pub fn get_image_with_kmeans(&mut self, k: usize) -> image::RgbImage {
        let segment_matrix = crate::kmeans::kmeans(&self.image, k);
        segment_matrix.into_centroid_image(&self.image)
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

            let grid = gtk::Grid::new();

            let mut images: Vec<gtk::Image> = Vec::new();
            let num_images = CONFIG.image_cols * CONFIG.image_rows;

            for i in 0..CONFIG.image_rows * CONFIG.image_cols {
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

                event_box.connect_button_press_event(move |_image, _event| {
                    println!("Pressed image {}", i);
                    gtk::Inhibit(false)
                });

                images.push(gtk_image.clone());

                let label = gtk::Label::new(Some(&format!("Child: {}", i + 2)));

                let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
                gtk_box.add(&label);
                gtk_box.add(&event_box);

                let row = (i / CONFIG.image_cols) as i32;
                let col = (i % CONFIG.image_cols) as i32;
                grid.attach(&gtk_box, col, row, 1, 1);
            }

            window.add(&grid);

            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            for i in 0..num_images {
                let mut worker = Worker::new();
                let thread_tx = tx.clone();
                thread::spawn(move || {
                    let image = worker.get_image_with_kmeans(i + 2);
                    thread_tx.send((i, image)).expect("Failed to send");
                });
            }

            rx.attach(None, move |(i, image)| {
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
                let gtk_image = &images[i];
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

                glib::Continue(true)
            });

            window.show_all();
        });
        self
    }

    pub fn run(self) -> Self {
        self.gui.run(&[]);
        self
    }
}
