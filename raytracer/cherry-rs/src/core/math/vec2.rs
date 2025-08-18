/// A 2D vector.
use std::ops::Index;

use crate::core::{Float, math::constants::ZERO_TOL};

#[derive(Debug)]
pub struct Vec2 {
    pub x: Float,
    pub y: Float,
}

impl Vec2 {
    // Determines whether two vectors are approximately equal.
    pub fn approx_eq(&self, other: &Self, tol: Float) -> bool {
        (self.x - other.x).abs() < tol && (self.y - other.y).abs() < tol
    }

    // Normalizes the vector in place.
    //
    // This method modifies the vector to have a length of 1 if the length is
    // greater than a small tolerance near zero. Otherwise, it is unchanged.
    pub fn normalize(&mut self) {
        let length = (self.x * self.x + self.y * self.y).sqrt();
        if length > ZERO_TOL {
            self.x /= length;
            self.y /= length;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::constants::REL_TOL;

    #[test]
    fn test_normalize() {
        let mut vec = Vec2 { x: 3.0, y: 4.0 };
        vec.normalize();
        assert!((vec.x - 0.6).abs() < REL_TOL);
        assert!((vec.y - 0.8).abs() < REL_TOL);
    }
}
