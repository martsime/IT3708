use image::{Rgb, RgbImage};

use crate::config::CONFIG;
use crate::matrix::{Matrix, Pos};

pub type SegmentMatrix = Matrix<usize>;
pub type VisitMatrix = Matrix<bool>;

#[derive(Clone)]
pub struct Segment {
    pub number: usize,
    pub positions: Vec<Pos>,
    pub size: usize,
}

pub struct SegmentContainer {
    segments: Vec<Segment>,
}

impl Segment {
    /// From a given Pos, use a depth first search to create a segment
    pub fn from_matrix_pos(
        pos: &Pos,
        segment_matrix: &SegmentMatrix,
        visited: &mut VisitMatrix,
    ) -> Segment {
        let segment_value = segment_matrix.get_pos(&pos).clone();
        let mut positions: Vec<Pos> = Vec::new();
        let mut stack: Vec<Pos> = Vec::new();
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

    /// Check all neighbour pixels of the segment and find the neighbour segment
    /// with most connections to the current segment
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

        if let Some((largest_index, _value)) =
            counts.iter().enumerate().max_by(|(_, a), (_, b)| a.cmp(&b))
        {
            largest_index
        } else {
            panic!("Could not find largest neighbour");
        }
    }

    /// Merge the other segment into self
    pub fn merge_in(&mut self, other: &Segment) {
        self.positions.extend(other.positions.iter().cloned());
        self.size = self.positions.len();
    }

    pub fn get_pixel_centroid(&self, image: &RgbImage) -> Rgb<u8> {
        // Calcute average color for each segment
        let mut r: f64 = 0.0;
        let mut g: f64 = 0.0;
        let mut b: f64 = 0.0;
        for pos in self.positions.iter() {
            let rgb = image.get_pixel(pos.x as u32, pos.y as u32);
            r += rgb[0] as f64;
            g += rgb[1] as f64;
            b += rgb[2] as f64;
        }
        let pixels = self.size as f64;
        let centroid_pixel = Rgb([
            (r / pixels).round() as u8,
            (g / pixels).round() as u8,
            (b / pixels).round() as u8,
        ]);
        centroid_pixel
    }
}

impl SegmentContainer {
    pub fn new_from_vec(segments: Vec<Segment>) -> SegmentContainer {
        SegmentContainer { segments: segments }
    }

    pub fn iter(&self) -> std::slice::Iter<Segment> {
        self.segments.iter()
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    fn get_smallest_index(&self) -> usize {
        if let Some((index, _segment)) = self
            .segments
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.size.cmp(&b.size))
        {
            index
        } else {
            panic!("Could not find smallest");
        }
    }

    fn get_segment_index(&self, number: usize) -> usize {
        if let Some((index, _segment)) = self
            .segments
            .iter()
            .enumerate()
            .find(|(_, segment)| segment.number == number)
        {
            index
        } else {
            panic!("Could not find segment with number: {}", number);
        }
    }

    /// Find the smallest segment and merge it with its
    /// most connecting neighbour segment
    pub fn merge_smallest(
        &mut self,
        segment_matrix: &SegmentMatrix,
        highest_value: usize,
    ) -> Option<Segment> {
        // Merges smallest into neighbour and returns neighbour
        let smallest_index = self.get_smallest_index();
        let smallest = self.segments[smallest_index].clone();
        if smallest.size < CONFIG.min_seg_size {
            let neighour_number = smallest.get_dominant_neighbour(segment_matrix, highest_value);
            let neighbour_index = self.get_segment_index(neighour_number);
            let neighbour = &mut self.segments[neighbour_index];
            neighbour.merge_in(&smallest);
            let new_segment = neighbour.clone();
            self.segments.remove(smallest_index);
            Some(new_segment)
        } else {
            None
        }
    }
}

impl VisitMatrix {
    pub fn new_default(width: usize, height: usize) -> Self {
        Matrix::<bool>::new(false, width, height)
    }
}

impl SegmentMatrix {
    /// Does a depth first search to find all N segments in the matrix
    /// and update the matrix values so numbers are in 0..N
    pub fn clean(&mut self) {
        let mut visited = VisitMatrix::new_default(self.width, self.height);

        let mut segment_number = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Pos::new(y, x);

                // If pos not visited yet, create a segment from it
                if !visited.get_pos(&pos) {
                    let segment_value = self.get_pos(&pos).clone();
                    let mut stack: Vec<Pos> = Vec::with_capacity(self.length);
                    stack.push(pos);
                    while !stack.is_empty() {
                        let current_pos = stack.pop().expect("Empty stack");
                        visited.set_at_pos(true, &current_pos);
                        self.set_at_pos(segment_number, &current_pos);
                        for new_pos in self.get_neighbours(&current_pos).into_iter() {
                            // If not visited and in same segment
                            if !visited.get_pos(&new_pos)
                                && *self.get_pos(&new_pos) == segment_value
                            {
                                stack.push(new_pos);
                            }
                        }
                    }
                    segment_number += 1;
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
        self.clean();
        let mut segments = self.get_segments();

        let highest_value = segments.len();
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
        self.clean();
    }

    pub fn into_centroid_image(&self, image: &RgbImage) -> RgbImage {
        let segments = self.get_segments();

        // Calcute average color for each segment
        let colors: Vec<Rgb<u8>> = segments
            .iter()
            .map(|segment| segment.get_pixel_centroid(&image))
            .collect();

        let mut new_image = RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let value = self.get_pos(&Pos::new(y, x));
                let color = colors[*value];
                new_image.put_pixel(x as u32, y as u32, color);
            }
        }
        new_image
    }
}
