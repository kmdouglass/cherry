/// Reference frame logic for building sequential optical systems.
use crate::core::{
    Float,
    math::{mat3::Mat3, vec3::Vec3},
};

/// A reference frame for 3D positioning of surfaces in a sequential optical
/// system.
///
/// The Cursor models my way of thinking about how 3D optical systems are built.
/// We start from a source and move sequentially from one surface to the next.
/// The cursor keeps track of the position and orientation of the system in 3D
/// after a surface is added to the system.
///
/// An alternative to the Cursor would be having the user specify the 3D
/// coordinates of each surface, but this logically detaches surface placement
/// from its position in the sequence of surfaces, which I find less intuitive.
#[derive(Debug)]
pub struct Cursor {
    /// The origin of the cursor reference frame. This changes with each surface
    /// added to the system, and it is always in the global coordinate
    /// system.
    pos: Vec3,

    /// Unit vector representing the right direction of the cursor reference
    /// frame.
    right: Vec3,

    /// Unit vector representing the up direction of the cursor reference frame.
    up: Vec3,

    /// Unit vector representing the forward direction of the cursor reference
    forward: Vec3,
}

impl Cursor {
    /// Create a new cursor at the given axial position in the global coordinate
    /// system.
    pub(crate) fn new(z: Float) -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, z),
            right: Vec3::new(1.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            forward: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    /// Advance the cursor by a given distance along the z-direction.
    pub fn advance(&mut self, distance: Float) {
        // Edge case for advancing from negative infinity to 0.
        if (self.pos.z() == Float::NEG_INFINITY) && (distance == Float::INFINITY) {
            self.pos.set_z(0.0);
            return;
        }
        self.pos += self.forward * distance;
    }

    /// Invert the direction of the cursor.
    pub fn invert(&mut self) {
        self.forward = -self.forward;

        // !todo!("Ensure right-handedness is maintained");
    }

    pub(super) fn pos(&self) -> Vec3 {
        self.pos
    }

    /// Returns a rotation matrix that transforms vectors from the global
    /// coordinate system to the local reference frame of the cursor.
    pub fn rotation_matrix(&self) -> Mat3 {
        Mat3::new(
            self.right.x(),
            self.up.x(),
            self.forward.x(),
            self.right.y(),
            self.up.y(),
            self.forward.y(),
            self.right.z(),
            self.up.z(),
            self.forward.z(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cursor_advance() {
        let mut cursor = Cursor::new(0.0);
        cursor.advance(1.0);
        assert_eq!(cursor.pos(), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_cursor_invert() {
        let mut cursor = Cursor::new(0.0);
        cursor.invert();
        cursor.advance(1.0);
        assert_eq!(cursor.pos(), Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_cursor_start_from_neg_infinity() {
        let mut cursor = Cursor::new(Float::NEG_INFINITY);
        cursor.advance(Float::INFINITY);
        assert_eq!(cursor.pos(), Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_cursor_rotation_matrix() {
        let cursor = Cursor::new(0.0);
        let expected = Mat3::identity();

        let rotation_matrix = cursor.rotation_matrix();

        assert!(
            rotation_matrix.approx_eq(&expected, 1e-6),
            "Expected rotation matrix to be identity, got: {:?}",
            rotation_matrix
        );
    }
}
