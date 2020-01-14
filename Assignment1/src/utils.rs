#[derive(Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Clone for Pos {
    fn clone(&self) -> Self {
        Pos {
            x: self.x,
            y: self.y,
        }
    }
}

impl Pos {
    pub fn distance_to(&self, other: &Pos) -> f64 {
        (((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f64).sqrt()
    }

    pub fn new(x: i32, y: i32) -> Pos {
        Pos { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let pos1 = Pos::new(0, 0);
        let pos2 = Pos::new(10, 10);
        let distance = pos1.distance_to(&pos2);
        assert_relative_eq!(14.142135623730951f64, distance, epsilon = 0.00001f64);
    }
}
