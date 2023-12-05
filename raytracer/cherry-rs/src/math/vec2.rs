#[derive(Debug, Clone)]
pub struct Vec2 {
    e: [f32; 2],
}

impl Vec2 {
    pub fn new(e0: f32, e1: f32) -> Self {
        Self { e: [e0, e1] }
    }

    pub fn y(&self) -> f32 {
        // Note the index; Vec2s are used for paraxial ray tracing, where the first value is the
        // distance of the ray from the optic axis.
        self.e[0]
    }

    pub fn theta(&self) -> f32 {
        self.e[1]
    }
}
