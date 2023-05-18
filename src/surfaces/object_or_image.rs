use crate::vec3::Vec3;

/// Represents the object or image plane in an optical system.
pub struct ObjectOrImagePlane {
    // Position of the center of the object plane.
    pos: Vec3,

    // Euler angles of the normal to the object plane.
    dir: Vec3,

    diam: f32,
}

impl ObjectOrImagePlane {
    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        Self {
            pos,
            dir,
            diam: f32::INFINITY,
        }
    }

    pub fn sag_norm(&self, _: Vec3) -> (f32, Vec3) {
        // Compute surface sag
        let sag = 0.0;

        // Compute surface normal
        let norm = Vec3::new(0.0, 0.0, 1.0);

        (sag, norm)
    }
}
