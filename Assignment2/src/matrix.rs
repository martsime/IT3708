#[derive(Clone, Debug)]
pub struct Pos {
    pub y: usize,
    pub x: usize,
}

impl Pos {
    pub fn new(y: usize, x: usize) -> Self {
        Pos { y: y, x: x }
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
        self.get(pos.y * self.width + pos.x)
    }

    pub fn set(&mut self, value: T, index: usize) {
        if index < self.length {
            self.data[index] = value;
        }
    }

    pub fn set_at_pos(&mut self, value: T, pos: &Pos) {
        self.set(value, pos.y * self.width + pos.x);
    }

    pub fn get_neighbours(&self, pos: &Pos) -> Vec<Pos> {
        let mut neighbours = Vec::with_capacity(4);
        // North
        if pos.y >= 1 {
            neighbours.push(Pos::new(pos.y - 1, pos.x));
        }
        // South
        if pos.y <= self.height - 2 {
            neighbours.push(Pos::new(pos.y + 1, pos.x));
        }
        // West
        if pos.x >= 1 {
            neighbours.push(Pos::new(pos.y, pos.x - 1));
        }
        // South
        if pos.x <= self.width - 2 {
            neighbours.push(Pos::new(pos.y, pos.x + 1));
        }

        neighbours
    }
}
