use std::cmp::max;
use std::fs::File;

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
use rand::Rng;

use crate::config::CONFIG;
use crate::problem::Problem;

fn generate_colors(n: usize) -> Vec<(f64, f64, f64)> {
    let mut rng = rand::thread_rng();
    (0..n)
        .into_iter()
        .map(|_| (rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>()))
        .collect()
}

pub fn draw_image(sequence: &Vec<usize>, fitness: usize, problem: &Problem) {
    let (image_width, image_height) = CONFIG.get_image_size();
    let (horizontal_padding, vertical_padding) = CONFIG.get_padding();
    let width = image_width - 2 * horizontal_padding;
    let height = image_height - 2 * vertical_padding;
    let surface = ImageSurface::create(Format::Rgb24, image_width, image_height)
        .expect("Cannot create image surface");
    let cr = Context::new(&surface);

    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.rectangle(0.0, 0.0, image_width as f64, image_height as f64);
    cr.fill();

    let num_machines: usize = problem.number_of_machines();

    let num_jobs: usize = problem.number_of_jobs();
    let colors = generate_colors(num_jobs);

    let machine_height = height as f64 / num_machines as f64;
    let machine_padding = 1.0;

    let mut machine_y: Vec<f64> = Vec::new();
    for i in 0..num_machines {
        machine_y.push(machine_height * (i as f64) + vertical_padding as f64 + machine_padding);
    }

    let time_x = width as f64 / fitness as f64;

    let mut machine_times = vec![0; num_machines];
    let mut job_times = vec![0; num_jobs];
    let mut job_operation_numbers = vec![1; num_jobs];
    for job_number in sequence {
        let job = problem.job(*job_number);
        let operation_number = job_operation_numbers[job.number - 1];
        let operation = &job.operations[operation_number - 1];
        // Update next operation for job
        job_operation_numbers[job.number - 1] = operation_number + 1;
        let machine = operation.machine;

        // Start time must be after time and when job and machine is ready
        let start_time = max(machine_times[machine], job_times[job.number - 1]);
        // Update when machine and job is ready for a new operation
        let end_time = start_time + operation.time;
        machine_times[machine] = end_time;
        job_times[job.number - 1] = end_time;

        let job_width = operation.time as f64 * time_x;
        let job_height = machine_height - 2.0 * machine_padding;
        let pos_y = machine_y[machine];
        let pos_x = start_time as f64 * time_x + horizontal_padding as f64;

        let (r, g, b) = colors[job.number - 1];
        cr.set_source_rgb(r, g, b);
        cr.rectangle(pos_x, pos_y, job_width, job_height);
        cr.fill();
    }

    // Display legends
    let rec_size: f64 = 20.0;
    let rec_padding: f64 = 2.0;
    let pos_y = vertical_padding as f64 + height as f64 + rec_size as f64 + rec_padding * 2.0;
    cr.select_font_face("Cairo", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(16.0);
    for job_number in 0..num_jobs {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let label_y = pos_y - rec_padding;
        let pos_x = horizontal_padding as f64 + (job_number) as f64 * (rec_size + rec_padding);
        cr.move_to(pos_x, label_y);
        cr.show_text(&format!("{}", job_number + 1));
        let (r, g, b) = colors[job_number];
        cr.set_source_rgb(r, g, b);
        cr.rectangle(pos_x, pos_y, rec_size, rec_size);
        cr.fill();
    }

    // Draw machine labels
    cr.set_source_rgb(0.0, 0.0, 0.0);
    let pos_x = horizontal_padding as f64 - 50.0;

    for machine_num in 0..num_machines {
        let pos_y = machine_y[machine_num] + machine_height / 2.0;
        cr.move_to(pos_x, pos_y);
        cr.show_text(&format!("M {}", machine_num));
    }

    let image_path = CONFIG.image_path(fitness);
    let mut file = File::create(&image_path).expect("Could not create image file");
    match surface.write_to_png(&mut file) {
        Ok(_) => {
            println!("Image: {} created", image_path);
        }
        Err(_) => {
            println!("Error writing image to path");
        }
    }
}
