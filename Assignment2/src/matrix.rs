use lazy_static::*;

#[derive(Clone, Debug)]
pub struct Pos {
    pub y: i32,
    pub x: i32,
}

impl Pos {
    pub fn new(y: i32, x: i32) -> Self {
        Pos { y: y, x: x }
    }

    pub fn new_usize(y: usize, x: usize) -> Pos {
        Pos {
            y: y as i32,
            x: x as i32,
        }
    }

    pub fn add(&self, other: &Pos) -> Pos {
        Pos::new(self.y + other.y, self.x + other.x)
    }
}

lazy_static! {
    pub static ref FOUR_DIRECTIONS: Vec<Pos> = vec![
        Pos::new(-1, 0), // N
        Pos::new(0, 1),  // E
        Pos::new(1, 0),  // S
        Pos::new(0, -1), // W
    ];
    pub static ref EIGHT_DIRECTIONS: Vec<Pos> = vec![
        Pos::new(-1, 0), // N
        Pos::new(-1, 1), // NE
        Pos::new(0, 1),  // E
        Pos::new(1, 1),  // SE
        Pos::new(1, 0),  // S
        Pos::new(1, -1), // SW
        Pos::new(0, -1), // W
        Pos::new(-1, -1),// NW
    ];
}

pub struct Matrix<T> {
    data: Vec<T>,
    pub width: usize,
    pub height: usize,
    pub length: usize,
}

impl<T> Matrix<T>
where
    T: Clone,
{
    pub fn new(init: T, width: usize, height: usize) -> Self {
        let length = width * height;
        Matrix {
            data: vec![init; length],
            width: width,
            height: height,
            length: length,
        }
    }

    pub fn get(&self, index: usize) -> &T {
        &self.data[index]
    }

    pub fn get_pos(&self, pos: &Pos) -> &T {
        self.get(pos.y as usize * self.width + pos.x as usize)
    }

    pub fn set(&mut self, value: T, index: usize) {
        if index < self.length {
            self.data[index] = value;
        }
    }

    pub fn set_at_pos(&mut self, value: T, pos: &Pos) {
        self.set(value, pos.y as usize * self.width + pos.x as usize);
    }

    fn get_valid_pos(&self, pos: &Pos, delta_pos: &Vec<Pos>) -> Vec<Pos> {
        delta_pos
            .iter()
            .filter_map(|direction| {
                let new_pos = pos.add(direction);
                if self.validate_pos(&new_pos) {
                    Some(new_pos)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_sides(&self, pos: &Pos) -> Vec<Pos> {
        self.get_valid_pos(pos, &FOUR_DIRECTIONS)
    }

    pub fn get_neighbours(&self, pos: &Pos) -> Vec<Pos> {
        self.get_valid_pos(pos, &EIGHT_DIRECTIONS)
    }

    pub fn validate_pos(&self, pos: &Pos) -> bool {
        pos.y >= 0 && pos.y < self.height as i32 && pos.x >= 0 && pos.x < self.width as i32
    }
}
