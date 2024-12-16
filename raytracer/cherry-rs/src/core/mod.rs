/// Shared data types for modeling ray tracing systems.
pub(super) mod math;
pub(crate) mod refractive_index;
pub(crate) mod sequential_model;

pub(crate) use math::array::argmin;
use math::vec3::Vec3;

pub(crate) type Float = f64;

pub(crate) const EPSILON: Float = Float::EPSILON;
pub(crate) const PI: Float = std::f64::consts::PI;

/// The cursor navigates through the optical system surface by surface, keeping
/// track of its position as it changes.
#[derive(Debug)]
pub(crate) struct Cursor {
    /// The position of the cursor.
    pos: Vec3,

    /// The direction of the cursor expressed as a unit vector.
    dir: Vec3,
}

impl Cursor {
    /// Create a new cursor at the given axial position in the global coordinate
    /// system.
    pub(crate) fn new(z: Float) -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, z),
            dir: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    /// Advance the cursor by a given distance along the z-direction.
    pub fn advance(&mut self, distance: Float) {
        // Edge case for advancing from negative infinity to 0.
        if (self.pos.z() == Float::NEG_INFINITY) && (distance == Float::INFINITY) {
            self.pos.set_z(0.0);
            return;
        }
        self.pos += self.dir * distance;
    }

    /// Invert the direction of the cursor.
    pub fn invert(&mut self) {
        self.dir = -self.dir;
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
