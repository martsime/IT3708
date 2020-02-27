use cogset::{Euclid, Kmeans};
use image::RgbImage;

use crate::segment::SegmentMatrix;

pub fn kmeans(image: &RgbImage, k: usize) -> SegmentMatrix {
    let data: Vec<Euclid<[f64; 3]>> = image
        .pixels()
        .map(|pix| Euclid([pix[0] as f64, pix[1] as f64, pix[2] as f64]))
        .collect();

    let kmeans = Kmeans::new(data.as_slice(), k);

    let (width, height) = image.dimensions();
    let mut segment_matrix = SegmentMatrix::new(0, width as usize, height as usize);
    for (i, (_, indices)) in kmeans.clusters().iter().enumerate() {
        for index in indices.iter() {
            segment_matrix.matrix.set(i, *index);
        }
    }

    segment_matrix.merge_all();
    segment_matrix
}
