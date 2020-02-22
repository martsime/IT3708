use image::{Rgb, RgbImage};
use rand::Rng;

use cogset::{Euclid, Kmeans};

use crate::matrix::Pos;
use crate::segment::SegmentMatrix;

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
    let mut matrix = SegmentMatrix::new(0, width as usize, height as usize);
    for (i, (_, indices)) in kmeans.clusters().iter().enumerate() {
        for index in indices.iter() {
            matrix.set(i, *index);
        }
    }

    matrix.clean();
    matrix.merge();
    matrix.clean();

    let mut segments = matrix.get_segments();

    let mut colors: Vec<Rgb<u8>> = Vec::new();
    for segment in segments.segments.iter() {
        let mut r: f64 = 0.0;
        let mut g: f64 = 0.0;
        let mut b: f64 = 0.0;
        for pos in segment.positions.iter() {
            let rgb = image.get_pixel(pos.x as u32, pos.y as u32);
            r += rgb[0] as f64;
            g += rgb[1] as f64;
            b += rgb[2] as f64;
        }
        let pixels = segment.size as f64;
        colors.push(Rgb([
            (r / pixels).round() as u8,
            (g / pixels).round() as u8,
            (b / pixels).round() as u8,
        ]));
    }

    let mut new_image = RgbImage::new(width, height);
    for y in 0..matrix.height {
        for x in 0..matrix.width {
            let value = matrix.get_pos(&Pos::new(y, x));
            let color = colors[*value];
            new_image.put_pixel(x as u32, y as u32, color);
        }
    }

    new_image
}
