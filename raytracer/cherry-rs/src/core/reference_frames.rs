/// Reference frame logic for building sequential optical systems.
use crate::core::{Float, math::vec3::Vec3};

/// A right, up, forward (RUF) reference frame.
///
/// A RUF reference frame forms a right-handed coordinate system where:
/// - The forward vector points in the direction of the optical axis.
/// - The right and up vectors are orthogonal to the forward vector and to each
///   other.
#[derive(Debug, Clone, Copy)]
pub struct RUFReferenceFrame {
    right: Vec3,
    up: Vec3,
    forward: Vec3,
}

/// Handles 3D positioning of surfaces in a sequential optical system.
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
    /// The position of the cursor. This changes with each surface added to
    /// the system, and it is always in the global coordinate system.
    pos: Vec3,

    /// The local reference frame of the cursor. This changes after interaction
    /// with a reflecting surface type.
    frame: RUFReferenceFrame,
}

impl RUFReferenceFrame {
    /// Creates a new right, up, forward (FRU) reference frame.
    ///
    /// By convention, the right vector points along the x-axis, the
    /// up vector points along the y-axis, and the forward vector points along
    /// the z-axis of the global reference frame,. The optical axis is
    /// aligned with the forward vector.
    pub fn new() -> Self {
        Self {
            right: Vec3::new(1.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            forward: Vec3::new(0.0, 0.0, 1.0),
        }
    }
}

impl Cursor {
    /// Create a new cursor at the given axial position in the global coordinate
    /// system.
    pub(crate) fn new(z: Float) -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, z),
            frame: RUFReferenceFrame::new(),
        }
    }

    /// Advance the cursor by a given distance along the z-direction.
    pub fn advance(&mut self, distance: Float) {
        // Edge case for advancing from negative infinity to 0.
        if (self.pos.z() == Float::NEG_INFINITY) && (distance == Float::INFINITY) {
            self.pos.set_z(0.0);
            return;
        }
        self.pos += self.frame.forward * distance;
    }

    /// Invert the direction of the cursor.
    pub fn invert(&mut self) {
        self.frame.forward = -self.frame.forward;

        // !todo!("Ensure right-handedness is maintained");
    }

    pub(super) fn pos(&self) -> Vec3 {
        self.pos
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
}
