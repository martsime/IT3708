use cogset::{Euclid, Kmeans};
use image::RgbImage;

use crate::segment::SegmentMatrix;

pub fn kmeans(image: &RgbImage, k: usize) -> SegmentMatrix {
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
    let mut matrix = SegmentMatrix::new(0, width as usize, height as usize);
    for (i, (_, indices)) in kmeans.clusters().iter().enumerate() {
        for index in indices.iter() {
            matrix.set(i, *index);
        }
    }

    matrix.merge();
    matrix
}
