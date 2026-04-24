use crate::{
    core::{Float, math::vec3::Vec3},
    specs::surfaces::{BoundaryType, Mask},
};

use super::{Surface, SurfaceKind};

/// A physical iris — a flat surface that clips rays by a circular aperture.
#[derive(Debug, Clone)]
pub struct Iris {
    mask: Mask,
}

impl Iris {
    pub fn new(semi_diameter: Float) -> Self {
        Self {
            mask: Mask::Circular { semi_diameter },
        }
    }
}

impl Surface for Iris {
    fn sag(&self, _pos: Vec3) -> Float {
        0.0
    }

    fn norm(&self, _pos: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }

    fn mask(&self) -> &Mask {
        &self.mask
    }

    fn boundary_type(&self) -> BoundaryType {
        BoundaryType::NoOp
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Iris
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn sag_and_norm_are_always_flat() {
        let iris = Iris::new(5.0);
        for pos in [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 4.0, 0.0),
            Vec3::new(-1.0, 2.5, 0.0),
        ] {
            assert_abs_diff_eq!(iris.sag(pos), 0.0);
            let norm = iris.norm(pos);
            assert_abs_diff_eq!(norm.x(), 0.0);
            assert_abs_diff_eq!(norm.y(), 0.0);
            assert_abs_diff_eq!(norm.z(), 1.0);
        }
    }

    #[test]
    fn boundary_type_is_noop() {
        let iris = Iris::new(5.0);
        assert!(matches!(iris.boundary_type(), BoundaryType::NoOp));
    }

    #[test]
    fn roc_default_is_infinity() {
        let iris = Iris::new(5.0);
        assert!(iris.roc(0.0).is_infinite());
    }

    #[test]
    fn mask_blocks_ray_outside_aperture() {
        let iris = Iris::new(5.0);
        assert!(!iris.mask().outside_clear_aperture(Vec3::new(4.9, 0.0, 0.0)));
        assert!(iris.mask().outside_clear_aperture(Vec3::new(5.1, 0.0, 0.0)));
    }

    #[test]
    fn mask_preserves_semi_diameter() {
        let iris = Iris::new(7.5);
        assert_abs_diff_eq!(iris.mask().semi_diameter(), 7.5);
    }
}
