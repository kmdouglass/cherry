/// A 3D vector
use std::ops;

static TOL: f32 = 1e-3;

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    e: [f32; 3],
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
