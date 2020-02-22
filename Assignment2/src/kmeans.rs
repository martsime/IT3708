use image::{Rgb, RgbImage};
use rand::Rng;

use cogset::{Euclid, Kmeans};

pub fn kmeans(image: &RgbImage, k: usize) -> RgbImage {
    let data: Vec<Euclid<[f64; 3]>> = image
        .pixels()
        .map(|pixel| {
            let r = pixel[0] as f64;
            let g = pixel[1] as f64;
            let b = pixel[2] as f64;
            let euclid = Euclid([r, g, b]);
            euclid
        })
        .collect();

    let kmeans = Kmeans::new(data.as_slice(), k);

    let (width, height) = image.dimensions();
    let mut new_image = RgbImage::new(width, height);
    for (i, (cluster, indices)) in kmeans.clusters().iter().enumerate() {
        let r = cluster.0[0] as u8;
        let g = cluster.0[1] as u8;
        let b = cluster.0[2] as u8;
        println!("Cluster: {}", i);
        let cluster_color = match i {
            0 => image::Rgb([255, 0, 0]),
            1 => image::Rgb([0, 255, 0]),
            2 => image::Rgb([0, 0, 255]),
            3 => image::Rgb([255, 255, 255]),
            4 => image::Rgb([0, 0, 0]),
            _ => image::Rgb([r, g, b]),
        };
        for index in indices.iter() {
            let x = index % width as usize;
            let y = index / width as usize;
            new_image.put_pixel(x as u32, y as u32, cluster_color);
        }
    }

    new_image
}
