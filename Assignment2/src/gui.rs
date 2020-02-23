use image::RgbImage;

use gdk_pixbuf::{Colorspace, Pixbuf};
use gio::prelude::*;
use gtk::prelude::*;

use crate::config::CONFIG;

pub struct Gui {
    images: Vec<gtk::Image>,
    labels: Vec<gtk::Label>,
    grid: gtk::Grid,
}

impl Gui {
    pub fn new() -> Gui {
        let images = Gui::generate_images();
        let grid = gtk::Grid::new();
        let labels = Gui::generate_labels();
        Gui {
            images: images,
            grid: grid,
            labels: labels,
        }
    }

    pub fn add_to_window(&self, window: &gtk::ApplicationWindow) {
        window.add(&self.grid);
    }

    fn generate_images() -> Vec<gtk::Image> {
        let mut images: Vec<gtk::Image> = Vec::new();
        for _ in 0..CONFIG.population_size {
            let pixbuf = gdk_pixbuf::Pixbuf::new_from_file_at_size(
                &CONFIG.image_path,
                CONFIG.image_size,
                CONFIG.image_size,
            )
            .expect("Failed to load image");
            let image = gtk::Image::new_from_pixbuf(Some(&pixbuf));
            images.push(image);
        }
        images
    }

    fn generate_labels() -> Vec<gtk::Label> {
        let mut labels: Vec<gtk::Label> = Vec::new();
        for i in 0..CONFIG.population_size {
            let text = format!("Label: {}", i);
            let label = gtk::Label::new(Some(&text));
            labels.push(label);
        }
        labels
    }

    pub fn build(&self) {
        let num_cols = (CONFIG.population_size as f64).sqrt().round() as usize;
        for i in 0..CONFIG.population_size {
            let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            gtk_box.add(&self.labels[i]);
            gtk_box.add(&self.images[i]);

            let row = (i / num_cols) as i32;
            let col = (i % num_cols) as i32;
            self.grid.attach(&gtk_box, col, row, 1, 1);
        }
    }

    pub fn update_image(&self, image: RgbImage, number: usize) {
        let gtk_image = &self.images[number];

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

        // Calculate display size from config and keep ratio
        let (display_width, display_height) = if width > height {
            let scale_ratio = CONFIG.image_size as f64 / width as f64;
            (CONFIG.image_size, (height as f64 * scale_ratio) as i32)
        } else {
            let scale_ratio = CONFIG.image_size as f64 / height as f64;
            ((width as f64 * scale_ratio) as i32, CONFIG.image_size)
        };

        let scaled_pixbuf = pixbuf
            .scale_simple(
                display_width,
                display_height,
                gdk_pixbuf::InterpType::Bilinear,
            )
            .expect("Failed to scale");

        gtk_image.set_from_pixbuf(Some(&scaled_pixbuf));
    }
}
