use rand::seq::SliceRandom;
use rand::Rng;

use crate::config::CONFIG;

pub fn get_new_position() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() * (CONFIG.x_max - CONFIG.x_min) + CONFIG.x_min
}

pub fn get_new_velocity() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() * (CONFIG.v_max - CONFIG.v_min) + CONFIG.v_min
}

pub fn shuffle_vec<T>(vec: &mut Vec<T>) {
    let mut rng = rand::thread_rng();
    vec.shuffle(&mut rng);
}

pub fn validate_velocity(velocity: f64) -> f64 {
    if velocity < CONFIG.v_min {
        CONFIG.v_min
    } else if velocity > CONFIG.v_max {
        CONFIG.v_max
    } else {
        velocity
    }
}
