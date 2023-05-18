use crate::vec3::Vec3;

/// A refracting flat circular surface.
pub struct RefractingCircularFlat {
    // Position of the center of the lens relative to the global reference frame
    pos: Vec3,

    // Euler angles of the optics axis through the lens relative to the global reference frame
    dir: Vec3,

    // Diameter of the lens
    diam: f32,

    // Refractive index
    n: f32,
}

impl RefractingCircularFlat {
    pub fn new(pos: Vec3, dir: Vec3, diam: f32, n: f32) -> Self {
        Self { pos, dir, diam, n }
    }

    pub fn sag_norm(&self, _: Vec3) -> (f32, Vec3) {
        // Compute surface sag
        let sag = 0.0;

        // Compute surface normal
        let norm = Vec3::new(0.0, 0.0, 1.0);

        (sag, norm)
    }
}
