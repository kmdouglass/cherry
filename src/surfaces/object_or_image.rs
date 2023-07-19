use crate::math::mat3::Mat3;
use crate::math::vec3::Vec3;

/// Represents the object or image plane in an optical system.
#[derive(Debug)]
pub struct ObjectOrImagePlane {
    // Position of the center of the object plane
    pub pos: Vec3,

    // Euler angles of the normal to the object plane
    pub dir: Vec3,

    // Rotation matrix from the global reference frame to the surface reference frame
    pub rot_mat: Mat3,

    // Diameter of the object plane
    pub diam: f32,

    // Refractive index
    pub(crate) n: f32,
}

impl ObjectOrImagePlane {
    pub fn new(pos: Vec3, dir: Vec3, diam: f32, n: f32) -> Self {
        let rot_mat = Mat3::from_euler_angles(dir.x(), dir.y(), dir.z());
        Self {
            pos,
            dir,
            rot_mat,
            diam: diam,
            n,
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
