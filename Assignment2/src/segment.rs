use std::collections::HashMap;

use crate::matrix::{Matrix, Pos};

use crate::config::CONFIG;

#[derive(Clone)]
pub struct Segment {
    pub number: usize,
    pub positions: Vec<Pos>,
    pub size: usize,
}

impl Segment {
    pub fn from_matrix_pos(
        pos: &Pos,
        segment_matrix: &SegmentMatrix,
        visited: &mut VisitMatrix,
    ) -> Self {
        let mut positions: Vec<Pos> = Vec::new();

        let segment_value = segment_matrix.get_pos(&pos).clone();

        let mut stack: Vec<Pos> = Vec::with_capacity(1000);
        stack.push(pos.clone());
        while !stack.is_empty() {
            let current_pos = stack.pop().expect("Non empty stack");
            if *visited.get_pos(&current_pos) {
                continue;
            }
            visited.set_at_pos(true, &current_pos);
            for new_pos in segment_matrix.get_neighbours(&current_pos).into_iter() {
                // If not visited and in same segment
                if !visited.get_pos(&new_pos) && *segment_matrix.get_pos(&new_pos) == segment_value
                {
                    stack.push(new_pos);
                }
            }
            positions.push(current_pos);
        }
        Segment {
            size: positions.len(),
            positions: positions,
            number: segment_value,
        }
    }

    pub fn get_dominant_neighbour(
        &self,
        segment_matrix: &SegmentMatrix,
        num_segments: usize,
    ) -> usize {
        let mut counts = vec![0; num_segments];

        for pos in self.positions.iter() {
            for new_pos in segment_matrix.get_neighbours(&pos).into_iter() {
                let segment_value = segment_matrix.get_pos(&new_pos).clone();
                if segment_value != self.number {
                    counts[segment_value] += 1;
                }
            }
        }

        let mut largest_index = 0;
        let mut largest_val: i32 = -1;
        for (index, value) in counts.iter().enumerate() {
            if *value as i32 >= largest_val {
                largest_val = *value as i32;
                largest_index = index;
            }
        }

        return largest_index;
    }

    pub fn merge_in(&mut self, other: &Segment) {
        self.positions.extend(other.positions.iter().cloned());
        self.size = self.positions.len();
    }
}

pub struct SegmentContainer {
    pub segments: Vec<Segment>,
}

impl SegmentContainer {
    pub fn new_from_vec(segments: Vec<Segment>) -> Self {
        Self { segments: segments }
    }
    fn get_smallest_index(&self) -> usize {
        let mut smallest: usize = 1_000_000;
        let mut smallest_i: usize = 0;
        for (i, segment) in self.segments.iter().enumerate() {
            if segment.size <= smallest {
                smallest_i = i;
                smallest = segment.size;
            }
        }
        smallest_i
    }

    fn get_segment_index(&self, number: usize) -> usize {
        for (i, segment) in self.segments.iter().enumerate() {
            if segment.number == number {
                return i;
            }
        }
        panic!("Could not find segment with number: {}", number);
    }

    pub fn merge_smallest(
        &mut self,
        segment_matrix: &SegmentMatrix,
        highest_value: usize,
    ) -> Option<Segment> {
        // Merges smallest into neighbour and returns neighbour
        let smallest_index = self.get_smallest_index();
        let smallest = self.segments[smallest_index].clone();
        // println!("Smallest: {}", smallest_index);
        match smallest.size < CONFIG.min_seg_size {
            true => {
                let neighour_number =
                    smallest.get_dominant_neighbour(segment_matrix, highest_value);
                let neighbour_index = self.get_segment_index(neighour_number);
                if smallest_index == neighbour_index {
                    panic!("Not allowed!");
                }
                let neighbour = &mut self.segments[neighbour_index];
                neighbour.merge_in(&smallest);
                println!("Merging {} into {}", smallest_index, neighbour_index);
                let new_segment = neighbour.clone();
                self.segments.remove(smallest_index);
                Some(new_segment)
            }
            false => None,
        }
    }
}

pub type SegmentMatrix = Matrix<usize>;
pub type VisitMatrix = Matrix<bool>;

impl VisitMatrix {
    pub fn new_default(width: usize, height: usize) -> Self {
        Matrix::<bool>::new(false, width, height)
    }
}

impl SegmentMatrix {
    pub fn clean(&mut self) {
        let mut visited = VisitMatrix::new_default(self.width, self.height);

        let mut segment_number = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Pos::new(y, x);
                if !visited.get_pos(&pos) {
                    self.dfs_clean(pos, segment_number, &mut visited);
                    segment_number += 1;
                }
            }
        }
    }

    fn dfs_clean(&mut self, pos: Pos, seg_number: usize, visited: &mut VisitMatrix) {
        let segment_value = self.get_pos(&pos).clone();

        let mut stack: Vec<Pos> = Vec::with_capacity(self.length);
        stack.push(pos);
        while !stack.is_empty() {
            let current_pos = stack.pop().expect("Non empty stack");
            visited.set_at_pos(true, &current_pos);
            self.set_at_pos(seg_number, &current_pos);
            for new_pos in self.get_neighbours(&current_pos).into_iter() {
                // If not visited and in same segment
                if !visited.get_pos(&new_pos) && *self.get_pos(&new_pos) == segment_value {
                    stack.push(new_pos);
                }
            }
        }
    }

    pub fn get_segments(&self) -> SegmentContainer {
        let mut visited = VisitMatrix::new_default(self.width, self.height);

        let mut segments: Vec<Segment> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Pos::new(y, x);
                if !visited.get_pos(&pos) {
                    let new_segment = Segment::from_matrix_pos(&pos, &self, &mut visited);
                    segments.push(new_segment);
                }
            }
        }

        SegmentContainer::new_from_vec(segments)
    }

    pub fn merge(&mut self) {
        let mut segments = self.get_segments();

        let highest_value = segments.segments.len();
        println!("Segments before merge: {}", segments.segments.len());
        loop {
            match segments.merge_smallest(&self, highest_value) {
                Some(new_segment) => {
                    for pos in new_segment.positions.iter() {
                        self.set_at_pos(new_segment.number, pos);
                    }
                }
                None => {
                    break;
                }
            };
        }

        println!("Segments left: {}", segments.segments.len());
    }
}
