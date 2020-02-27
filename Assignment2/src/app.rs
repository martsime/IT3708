use std::thread;

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

        self.simulation.add_initial(segment_matrices);
        self.simulation.population.evaluate(&self.image);
        println!("Evaluated!");
        let images: Vec<RgbImage> = self
            .simulation
            .population
            .individuals
            .iter()
            .map(|individual| individual.segment_matrix.into_centroid_image(&self.image))
            .collect();

        self.simulation.population.get_fronts();
        println!("Images generated");
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
            window.set_border_width(CONFIG.gui_border as u32);
            window.set_position(gtk::WindowPosition::Center);
            window.set_default_size(
                CONFIG.gui_width + CONFIG.gui_border * 2,
                CONFIG.gui_height + CONFIG.gui_border * 2,
            );

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
