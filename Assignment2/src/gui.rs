use std::f64::consts::PI;

use image::RgbImage;

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
use gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::prelude::*;

use crate::config::CONFIG;
use crate::segment::SegmentMatrix;
use crate::simulation::Fronts;

mod plot {
    use super::*;
    pub fn generate_image() -> gtk::Image {
        let (width, height) = CONFIG.plot_size();
        let surface =
            ImageSurface::create(Format::Rgb24, width, height).expect("Unable to create surface");
        let cr = Context::new(&surface);
        draw_background(&cr);
        // draw_labels(&cr);

        cr.set_line_width(1.0);
        cr.set_source_rgb(1.0, 0.0, 0.0);
        cr.rectangle(100.0, 100.0, 100.0, 100.0);
        cr.stroke();
        gtk::Image::new_from_surface(Some(&surface))
    }

    pub fn update_image(image: &gtk::Image) {
        let (width, height) = CONFIG.plot_size();
        let surface =
            ImageSurface::create(Format::Rgb24, width, height).expect("Unable to create surface");
        let cr = Context::new(&surface);
        draw_background(&cr);
        draw_labels(&cr);
        image.set_from_surface(Some(&surface));
    }

    fn draw_background(cr: &Context) {
        let (width, height) = CONFIG.plot_size();
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        cr.fill();
    }

    fn draw_labels(cr: &Context) {
        let (width, height) = CONFIG.plot_size();
        cr.select_font_face("Cairo", FontSlant::Normal, FontWeight::Normal);
        cr.set_font_size(16.0);
        let label_padding: f64 = 20.0;
        // y-axis label
        let text = "Hello world";
        let te = cr.text_extents(&text);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.move_to(
            label_padding + te.height / 2.0,
            height as f64 / 2.0 + te.width / 2.0,
        );
        cr.rotate(-PI / 2.0);
        cr.show_text(text);
        cr.rotate(PI / 2.0);
        cr.stroke();

        // x-axis label
        let text = "Hello world";
        let te = cr.text_extents(&text);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.move_to(
            width as f64 / 2.0 - te.width / 2.0,
            height as f64 - label_padding / 2.0 - te.height / 2.0,
        );
        cr.show_text(text);
        cr.stroke();
    }
}

pub struct Gui {
    image: image::RgbImage,
    original_image: gtk::Image,
    images: Vec<gtk::Image>,
    labels: Vec<gtk::Label>,
    plots: Vec<gtk::Image>,
    container: gtk::Box,
}

impl Gui {
    pub fn new() -> Gui {
        let image: image::RgbImage = match image::open(&CONFIG.image_path) {
            Ok(image) => image.into_rgb(),
            Err(_) => panic!("Unable to load image!"),
        };
        let original_image = Gui::generate_original_image();
        let images = Gui::generate_images();
        let labels = Gui::generate_labels();
        let plots = Gui::generate_plots();
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        Gui {
            image: image,
            original_image: original_image,
            images: images,
            labels: labels,
            plots: plots,
            container,
        }
    }

    pub fn add_to_window(&self, window: &gtk::ApplicationWindow) {
        window.add(&self.container);
    }

    fn generate_original_image() -> gtk::Image {
        let pixbuf = gdk_pixbuf::Pixbuf::new_from_file_at_size(
            &CONFIG.image_path,
            CONFIG.original_image_size,
            CONFIG.original_image_size,
        )
        .expect("Failed to load image");
        gtk::Image::new_from_pixbuf(Some(&pixbuf))
    }

    fn generate_images() -> Vec<gtk::Image> {
        let pixbuf = &gdk_pixbuf::Pixbuf::new_from_file_at_size(
            &CONFIG.image_path,
            CONFIG.image_size,
            CONFIG.image_size,
        )
        .expect("Failed to load image");
        let mut images: Vec<gtk::Image> = Vec::new();
        for _ in 0..CONFIG.population_size {
            let image = gtk::Image::new_from_pixbuf(Some(pixbuf));
            images.push(image);
        }
        images
    }

    fn generate_plots() -> Vec<gtk::Image> {
        let mut plots: Vec<gtk::Image> = Vec::new();
        plots.push(plot::generate_image());
        plots.push(plot::generate_image());
        plots.push(plot::generate_image());
        plots
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
        // Left side
        let left_scroll = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
        left_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Always);
        left_scroll.set_min_content_width(CONFIG.original_image_size);
        left_scroll.set_vexpand(true);

        let left_flowbox = gtk::FlowBox::new();
        left_flowbox.set_orientation(gtk::Orientation::Horizontal);
        left_flowbox.set_selection_mode(gtk::SelectionMode::None);
        left_flowbox.set_max_children_per_line(1);

        left_flowbox.add(&self.original_image);
        for plot in self.plots.iter() {
            left_flowbox.add(plot);
        }
        left_scroll.add(&left_flowbox);

        // Right side
        let right_scroll = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
        right_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Always);
        right_scroll.set_hexpand(true);
        right_scroll.set_vexpand(true);

        let right_flowbox = gtk::FlowBox::new();
        right_flowbox.set_orientation(gtk::Orientation::Horizontal);
        right_flowbox.set_selection_mode(gtk::SelectionMode::None);

        for i in 0..CONFIG.population_size {
            let gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            gtk_box.add(&self.labels[i]);
            gtk_box.add(&self.images[i]);
            right_flowbox.add(&gtk_box);
        }

        right_scroll.add(&right_flowbox);

        self.container.add(&left_scroll);
        self.container.add(&right_scroll);
    }

    pub fn update_image(&self, number: usize, segment_matrix: &SegmentMatrix) {
        let gtk_image = &self.images[number];
        let image = segment_matrix.into_centroid_image(&self.image);

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

        let current_pixbuf = gtk_image.get_pixbuf().expect("Failed to get pixbuf");
        let image_width = current_pixbuf.get_width();
        let image_height = current_pixbuf.get_height();

        let scaled_pixbuf = pixbuf
            .scale_simple(image_width, image_height, gdk_pixbuf::InterpType::Bilinear)
            .expect("Failed to scale");

        gtk_image.set_from_pixbuf(Some(&scaled_pixbuf));
    }

    pub fn update_plots(&self, fronts: &Fronts) {
        println!("Updating plots!");
        for image in self.plots.iter() {
            plot::update_image(image);
        }
    }
}
