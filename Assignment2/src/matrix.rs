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

    pub fn get_neighbours(&self, pos: &Pos) -> Vec<Pos> {
        let mut neighbours = Vec::with_capacity(4);
        // North
        if pos.y >= 1 {
            neighbours.push(Pos::new(pos.y - 1, pos.x));
        }
        // South
        if pos.y as usize <= self.height - 2 {
            neighbours.push(Pos::new(pos.y + 1, pos.x));
        }
        // West
        if pos.x >= 1 {
            neighbours.push(Pos::new(pos.y, pos.x - 1));
        }
        // South
        if pos.x as usize <= self.width - 2 {
            neighbours.push(Pos::new(pos.y, pos.x + 1));
        }

        neighbours
    }

    pub fn validate_pos(&self, pos: &Pos) -> bool {
        pos.y >= 0 && pos.y < self.height as i32 && pos.x >= 0 && pos.x < self.width as i32
    }
}
