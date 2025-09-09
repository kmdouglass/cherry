/// A line in 3D.
use crate::core::math::vec3::Vec3;

#[derive(Debug)]
pub struct Line {
    point: Vec3,
    direction: Vec3,
}

impl Line {
    pub fn new(point: Vec3, direction: Vec3) -> Self {
        Self { point, direction }
    }
}
