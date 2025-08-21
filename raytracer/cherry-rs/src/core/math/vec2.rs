/// A 2D vector.
use std::ops::{Add, Index};

use crate::core::{Float, math::constants::ZERO_TOL};

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: Float,
    pub y: Float,
}

impl Vec2 {
    /// Determines whether two vectors are approximately equal.
    pub fn approx_eq(&self, other: &Self, tol: Float) -> bool {
        (self.x - other.x).abs() < tol && (self.y - other.y).abs() < tol
    }

    /// Computes the dot product of the vector with another.
    pub fn dot(&self, other: &Self) -> Float {
        self.x * other.x + self.y * other.y
    }

    /// Computes the length of the vector.
    pub fn length(&self) -> Float {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Compute the length squared of the vector.
    pub fn length_squared(&self) -> Float {
        self.x * self.x + self.y * self.y
    }

    /// Normalizes the vector in place.
    ///
    /// This method modifies the vector to have a length of 1 if the length is
    /// greater than a small tolerance near zero. Otherwise, it is unchanged.
    pub fn normalize(&mut self) {
        let length = self.length();
        if length > ZERO_TOL {
            self.x /= length;
            self.y /= length;
        }
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Self;

    fn add(self, other: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
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
    fn vec2_addition() {
        let vec1 = Vec2 { x: 1.0, y: 2.0 };
        let vec2 = Vec2 { x: 3.0, y: 4.0 };
        let expected = Vec2 { x: 4.0, y: 6.0 };

        let result = vec1 + vec2;

        assert!(result.approx_eq(&expected, REL_TOL), "Vec2 addition failed");
    }

    #[test]
    fn vec2_normalize() {
        let mut vec = Vec2 { x: 3.0, y: 4.0 };
        vec.normalize();
        assert!((vec.x - 0.6).abs() < REL_TOL);
        assert!((vec.y - 0.8).abs() < REL_TOL);
    }
}
