/// A 3D vector
use std::ops;

use serde::{Deserialize, Serialize};

static TOL: f32 = 1e-3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(into = "[f32; 3]")]
pub struct Vec3 {
    e: [f32; 3],
}

/// Required to serialize Vec3 directly into an array instead of a JSON Object.
impl Into<[f32; 3]> for Vec3 {
    fn into(self) -> [f32; 3] {
        self.e
    }
}

impl Vec3 {
    pub fn new(e0: f32, e1: f32, e2: f32) -> Self {
        Self { e: [e0, e1, e2] }
    }

    pub fn x(&self) -> f32 {
        self.e[0]
    }

    pub fn y(&self) -> f32 {
        self.e[1]
    }

    pub fn z(&self) -> f32 {
        self.e[2]
    }

    pub fn set_x(&mut self, x: f32) {
        self.e[0] = x;
    }

    pub fn set_y(&mut self, y: f32) {
        self.e[1] = y;
    }

    pub fn set_z(&mut self, z: f32) {
        self.e[2] = z;
    }

    pub fn k(&self) -> f32 {
        self.e[0]
    }

    pub fn l(&self) -> f32 {
        self.e[1]
    }

    pub fn m(&self) -> f32 {
        self.e[2]
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.e.iter().map(|e| e * e).sum()
    }

    pub fn normalize(&self) -> Self {
        let length = self.length();
        Self::new(self.e[0] / length, self.e[1] / length, self.e[2] / length)
    }

    pub fn is_unit(&self) -> bool {
        (self.length_squared() - 1.0).abs() < TOL
    }

    pub fn dot(&self, rhs: Self) -> f32 {
        self.e[0] * rhs.e[0] + self.e[1] * rhs.e[1] + self.e[2] * rhs.e[2]
    }

    /// Create a fan of uniformly spaced vectors with endpoints in a given z-plane.
    ///
    /// The vectors have endpoints at an angle theta with respect to the x-axis and extend from
    /// distances -r to r from the point (0, 0, z).
    ///
    /// # Arguments
    /// - n: Number of vectors to create
    /// - r: Radial span of vector endpoints from [-r, r]
    /// - theta: Angle of vectors with respect to x
    /// - z: z-coordinate of endpoints
    pub fn fan(n: usize, r: f32, theta: f32, z: f32) -> Vec<Self> {
        // TODO: Include endpoints!
        let mut vecs = Vec::with_capacity(n);
        for i in 0..n {
            let x = r * (2.0 * i as f32 / n as f32 - 1.0) * theta.cos();
            let y = r * (2.0 * i as f32 / n as f32 - 1.0) * theta.sin();
            vecs.push(Vec3::new(x, y, z));
        }
        vecs
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, rhs: &Self) -> bool {
        (self.e[0] - rhs.e[0]).abs() < TOL
            && (self.e[1] - rhs.e[1]).abs() < TOL
            && (self.e[2] - rhs.e[2]).abs() < TOL
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

impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs)
    }
}
