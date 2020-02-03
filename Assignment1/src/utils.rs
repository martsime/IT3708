#[derive(Debug, Eq, PartialEq)]
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
}
