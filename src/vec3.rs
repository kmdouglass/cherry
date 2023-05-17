#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Vec3 {
    e: [f32; 3],
}

impl Vec3 {
    pub fn new(e0: f32, e1: f32, e2: f32) -> Self {
        Self { e: [e0, e1, e2] }
    }

    fn x(&self) -> f32 {
        self.e[0]
    }

    fn y(&self) -> f32 {
        self.e[1]
    }

    fn z(&self) -> f32 {
        self.e[2]
    }

    fn k(&self) -> f32 {
        self.e[0]
    }

    fn l(&self) -> f32 {
        self.e[1]
    }

    fn m(&self) -> f32 {
        self.e[2]
    }

    fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    fn length_squared(&self) -> f32 {
        self.e.iter().map(|e| e * e).sum()
    }
}
