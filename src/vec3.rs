#[derive(Debug, Clone, Copy, PartialEq)]
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
}
