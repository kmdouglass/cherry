/// A 2D vector.
use std::ops::Index;
 
use crate::core::Float;

#[derive(Debug)]
pub struct Vec2 {
    pub x: Float,
    pub y: Float,
}

impl Index<usize> for Vec2 {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of bounds for Vec2"),
        }
    }
}
