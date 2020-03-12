use std::thread;

use image::RgbImage;
use rayon::prelude::*;

use gio::prelude::*;
use gtk::prelude::*;

use glib::Sender;

use crate::config::CONFIG;
use crate::gui::Gui;
use crate::segment::SegmentMatrix;
use crate::simulation::{Fronts, Simulation};

pub struct App {
    app: gtk::Application,
}

struct Worker {
    image: image::RgbImage,
    simulation: Simulation,
    image_channel: Sender<Fronts>,
}

impl Worker {
    pub fn new(image_channel: Sender<Fronts>) -> Worker {
        let image: image::RgbImage = match image::open(&CONFIG.image_path()) {
            Ok(image) => image.into_rgb(),
            Err(_) => panic!("Unable to load image!"),
        };
        Worker {
            image: image,
            simulation: Simulation::new(),
            image_channel: image_channel,
        }
    }

    pub fn run(&mut self) {
        let segment_matrices: Vec<SegmentMatrix> = (0..CONFIG.kmeans)
            .into_par_iter()
            .map(|i| crate::kmeans::kmeans(&self.image, i + 2))
            .collect();

        self.simulation.add_initial(segment_matrices);
        self.simulation.evaluate(&self.image);
        println!("Evaluated!");
        let mut fronts = self.simulation.population.get_fronts();
        self.simulation.population.print_fronts(&fronts);
        self.image_channel
            .send(fronts)
            .expect("Failed to send images");
        for i in 0..CONFIG.generations {
            self.simulation.evolve(&self.image);
            fronts = self.simulation.population.get_fronts();
            self.simulation.population.print_fronts(&fronts);
            self.image_channel
                .send(fronts)
                .expect("Failed to send images");
        }

        let fronts = self.simulation.population.get_fronts();

        for individual in fronts.layers[0].iter() {
            let segment_matrix = &individual.segment_matrix;
            let segments = segment_matrix.get_segments();
            let image = segment_matrix.into_green_border_image(&self.image);
            let number = segments.len();
            let image_path = format!("{}/green-image-{}.jpg", CONFIG.out_path(), number);
            println!("Image format: {}", image_path);
            if number <= CONFIG.max_segments {
                image::save_buffer_with_format(
                    image_path,
                    &image,
                    image.width(),
                    image.height(),
                    image::ColorType::Rgb8,
                    image::ImageFormat::Jpeg,
                )
                .expect("Unable to save image");
            }
            let image = segment_matrix.into_border_image();
            let image_path = format!("{}/border-image-{}.jpg", CONFIG.out_path(), number);
            println!("Image format: {}", image_path);
            if number <= CONFIG.max_segments {
                image::save_buffer_with_format(
                    image_path,
                    &image,
                    image.width(),
                    image.height(),
                    image::ColorType::Rgb8,
                    image::ImageFormat::Jpeg,
                )
                .expect("Unable to save image");
            }
        }
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

            r_image_channel.attach(None, move |fronts| {
                let mut i = 0;
                for layer in fronts.layers.iter() {
                    for individual in layer.iter() {
                        gui.update_image(i, &individual.segment_matrix);
                        i += 1;
                    }
                }
                gui.update_plots(&fronts);
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
