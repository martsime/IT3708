use image::Rgb;

pub fn pixel_distance(a: &Rgb<u8>, b: &Rgb<u8>) -> f64 {
    ((a[0] as f64 - b[0] as f64).powi(2)
        + (a[1] as f64 - b[1] as f64).powi(2)
        + (a[2] as f64 - b[2] as f64).powi(2))
    .sqrt()
}
