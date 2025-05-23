/// A 3D vector
use serde::{Deserialize, Serialize};

use crate::core::{EPSILON, Float, PI};

const TOL: Float = (1 as Float) * EPSILON;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(into = "[Float; 3]")]
pub struct Vec3 {
    e: [Float; 3],
}

/// Required to serialize Vec3 directly into an array instead of a JSON Object.
impl From<Vec3> for [Float; 3] {
    fn from(val: Vec3) -> Self {
        val.e
    }
}

impl Vec3 {
    pub fn new(e0: Float, e1: Float, e2: Float) -> Self {
        Self { e: [e0, e1, e2] }
    }

    pub fn x(&self) -> Float {
        self.e[0]
    }

    pub fn y(&self) -> Float {
        self.e[1]
    }

    pub fn z(&self) -> Float {
        self.e[2]
    }

    pub fn set_x(&mut self, x: Float) {
        self.e[0] = x;
    }

    pub fn set_y(&mut self, y: Float) {
        self.e[1] = y;
    }

    pub fn set_z(&mut self, z: Float) {
        self.e[2] = z;
    }

    pub fn k(&self) -> Float {
        self.e[0]
    }

    pub fn l(&self) -> Float {
        self.e[1]
    }

    pub fn m(&self) -> Float {
        self.e[2]
    }

    pub fn length(&self) -> Float {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> Float {
        self.e.iter().map(|e| e * e).sum()
    }

    /// Create a vector with a length of 1.0 in the same direction as the
    /// original vector.
    ///
    /// If the vector has a length of 0.0, the original vector is returned
    /// instead of a Result type. This is to avoid the overhead of unwrapping
    /// the Result type in the calling code.
    pub fn normalize(&self) -> Self {
        let length = self.length();

        if length == 0.0 {
            return *self;
        }

        Self::new(self.e[0] / length, self.e[1] / length, self.e[2] / length)
    }

    pub fn is_unit(&self) -> bool {
        (self.length_squared() - 1.0).abs() / Float::max(1.0, self.length_squared()) < TOL
    }

    pub fn dot(&self, rhs: Self) -> Float {
        self.e[0] * rhs.e[0] + self.e[1] * rhs.e[1] + self.e[2] * rhs.e[2]
    }

    /// Create a square grid of vectors that sample a circle.
    ///
    /// # Arguments
    /// - `radius` - The radius of the circle.
    /// - `z` - The z-coordinate of the circle.
    /// - `spacing` - The spacing of the grid. For example, a spacing of 1.0
    ///   will sample the circle at every pair of integer coordinates, while a
    ///   scale of 0.5 will sample the circle at every pair of half-integer
    ///   coordinates.
    /// - radial_offset_x: Offset the radial position of the vectors by this
    ///   amount in x
    /// - radial_offset_y: Offset the radial position of the vectors by this
    ///   amount in y
    pub fn sq_grid_in_circ(
        radius: Float,
        spacing: Float,
        z: Float,
        radial_offset_x: Float,
        radial_offset_y: Float,
    ) -> Vec<Self> {
        // Upper bound is established by the Gauss Circle Problem.
        let r_over_s = radius / spacing;
        let num_samples = (PI * r_over_s * r_over_s + 9 as Float * r_over_s).ceil() as usize;

        // Bounding box search.
        let mut samples = Vec::with_capacity(num_samples);
        let mut x = -radius;
        while x <= radius {
            let mut y = -radius;
            while y <= radius {
                if x * x + y * y <= radius * radius {
                    samples.push(Self::new(x + radial_offset_x, y + radial_offset_y, z));
                }
                y += spacing;
            }
            x += spacing;
        }

        samples
    }

    /// Create a fan of uniformly spaced vectors with endpoints in a given
    /// z-plane.
    ///
    /// The vectors have endpoints at an angle theta with respect to the x-axis
    /// and extend from distances (-r + radial_offset) to (r +
    /// radial_offset) from the point (0, 0, z).
    ///
    /// # Arguments
    /// - n: Number of vectors to create
    /// - r: Radial span of vector endpoints from [-r, r]
    /// - z: z-coordinate of endpoints
    /// - theta: Angle of vectors with respect to x
    /// - radial_offset_x: Offset the radial position of the vectors by this
    ///   amount in x
    /// - radial_offset_y: Offset the radial position of the vectors by this
    ///   amount in y
    pub fn fan(
        n: usize,
        r: Float,
        z: Float,
        theta: Float,
        radial_offset_x: Float,
        radial_offset_y: Float,
    ) -> Vec<Self> {
        let mut vecs = Vec::with_capacity(n);
        let step = 2.0 * r / (n - 1) as Float;
        for i in 0..n {
            let x = (-r + i as Float * step) * theta.cos() + radial_offset_x;
            let y = (-r + i as Float * step) * theta.sin() + radial_offset_y;
            vecs.push(Vec3::new(x, y, z));
        }
        vecs
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, rhs: &Self) -> bool {
        (self.e[0] - rhs.e[0]) * (self.e[0] - rhs.e[0])
            + (self.e[1] - rhs.e[1]) * (self.e[1] - rhs.e[1])
            + (self.e[2] - rhs.e[2]) * (self.e[2] - rhs.e[2])
            < TOL * TOL
    }
}

impl std::ops::Add<Vec3> for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.e[0] + rhs.e[0],
            self.e[1] + rhs.e[1],
            self.e[2] + rhs.e[2],
        )
    }
}

impl std::ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.e[0] += rhs.e[0];
        self.e[1] += rhs.e[1];
        self.e[2] += rhs.e[2];
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.e[0], -self.e[1], -self.e[2])
    }
}

impl std::ops::Sub<Vec3> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.e[0] - rhs.e[0],
            self.e[1] - rhs.e[1],
            self.e[2] - rhs.e[2],
        )
    }
}

impl std::ops::Mul<Float> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self {
        Self::new(self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_normalize() {
        let v = Vec3::new(1.0, 1.0, 1.0);
        let norm = v.normalize();

        assert_ne!(v.length(), 1.0);
        assert_eq!(norm.length(), 1.0);
    }

    #[test]
    fn test_normalize_zero_length() {
        let v = Vec3::new(0.0, 0.0, 0.0);
        let norm = v.normalize();

        assert_eq!(norm.length(), 0.0);
    }

    #[test]
    fn test_sample_circle_sq_grid_unit_circle() {
        let samples = Vec3::sq_grid_in_circ(1.0, 1.0, 0.0, 0.0, 0.0);
        assert_eq!(samples.len(), 5);
    }

    #[test]
    fn test_sample_circle_sq_grid_radius_2_scale_2() {
        let samples = Vec3::sq_grid_in_circ(2.0, 2.0, 0.0, 0.0, 0.0);
        assert_eq!(samples.len(), 5);
    }

    #[test]
    fn test_sample_circle_sq_grid_radius_2_scale_1() {
        let samples = Vec3::sq_grid_in_circ(2.0, 1.0, 0.0, 0.0, 0.0);
        assert_eq!(samples.len(), 13);
    }
}
