use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use image::RgbImage;
use rayon::prelude::*;

use gio::prelude::*;
use gtk::prelude::*;

use glib::Sender;

use crate::config::CONFIG;
use crate::gui::Gui;
use crate::segment::SegmentMatrix;
use crate::simulation::Simulation;

pub struct App {
    app: gtk::Application,
}

struct Worker {
    image: image::RgbImage,
    simulation: Simulation,
    channel: Sender<Vec<RgbImage>>,
}

impl Worker {
    pub fn new(channel: Sender<Vec<RgbImage>>) -> Worker {
        let image: image::RgbImage = match image::open(&CONFIG.image_path) {
            Ok(image) => image.into_rgb(),
            Err(_) => panic!("Unable to load image!"),
        };
        Worker {
            image: image,
            simulation: Simulation::new(),
            channel: channel,
        }
    }

    pub fn run(&mut self) {
        let segment_matrices: Vec<SegmentMatrix> = (0..CONFIG.kmeans)
            .into_par_iter()
            .map(|i| crate::kmeans::kmeans(&self.image, i + 2))
            .collect();

        let images: Vec<RgbImage> = segment_matrices
            .iter()
            .map(|segment_matrix| segment_matrix.into_centroid_image(&self.image))
            .collect();

        println!("Images generated");
        self.simulation.add_initial(segment_matrices);
        self.simulation.population.evaluate(&self.image);
        println!("Evaluated!");

        self.channel.send(images).expect("Failed to send images");
    }
}

impl App {
    pub fn new() -> App {
        let application = gtk::Application::new(
            Some("com.github.martsime.IT3708.assignment2"),
            Default::default(),
        )
        .expect("Initialization failed...");
        App { app: application }
    }

    pub fn build(self) -> Self {
        self.app.connect_activate(|app| {
            let window = gtk::ApplicationWindow::new(app);

            window.set_title("");
            window.set_border_width(10);
            window.set_position(gtk::WindowPosition::Center);
            window.set_default_size(1000, 1000);

            let grid = gtk::Grid::new();
            let gui = Gui::new();
            gui.build();
            gui.add_to_window(&window);

            let (t_image_channel, r_image_channel) =
                glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            thread::spawn(move || {
                let mut worker = Worker::new(t_image_channel);
                worker.run();
            });

            r_image_channel.attach(None, move |images| {
                for (i, image) in images.into_iter().enumerate() {
                    gui.update_image(image, i);
                }
                glib::Continue(true)
            });

            window.show_all();
        });
        self
    }

    pub fn run(self) -> Self {
        self.app.run(&[]);
        self
    }
}
