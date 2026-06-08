use anyhow::Result;

use crate::{
    core::{Float, math::vec3::Vec3, ray::Ray},
    specs::surfaces::{BoundaryKind, Mask},
};

use super::super::{Surface, SurfaceKind, solvers::flat_surface};

/// A flat beam-splitting surface nominally tilted like a mirror.
///
/// The surface is purely geometric. Arm selection (transmitting vs. reflecting)
/// is declared per path in [`PathSpec::beam_splitter_arms`] and exposed
/// per iterator step via [`Step::bs_arm`]. Ray tracing callers use `bs_arm`
/// to determine the appropriate optical behavior for each traversal.
///
/// [`PathSpec::beam_splitter_arms`]: crate::specs::paths::PathSpec
/// [`Step::bs_arm`]: crate::core::sequential_model::Step::bs_arm
#[derive(Debug, Clone)]
pub struct BeamSplitter {
    mask: Mask,
}

impl BeamSplitter {
    pub fn new(semi_diameter: Float) -> Self {
        Self {
            mask: Mask::Circular { semi_diameter },
        }
    }
}

impl Surface for BeamSplitter {
    fn boundary_kind(&self) -> BoundaryKind {
        BoundaryKind::Refracting
    }

    fn intersect(&self, ray: &Ray, _max_iter: usize) -> Result<(Vec3, Vec3)> {
        flat_surface(ray, self, 0)
    }

    fn mask(&self) -> &Mask {
        &self.mask
    }

    fn norm(&self, _pos: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }

    fn sag(&self, _pos: Vec3) -> Float {
        0.0
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::BeamSplitter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn boundary_kind_is_refracting() {
        let bs = BeamSplitter::new(5.0);
        assert!(matches!(bs.boundary_kind(), BoundaryKind::Refracting));
    }

    #[test]
    fn surface_kind_is_beam_splitter() {
        let bs = BeamSplitter::new(5.0);
        assert_eq!(bs.surface_kind(), SurfaceKind::BeamSplitter);
    }

    #[test]
    fn sag_is_zero_everywhere() {
        let bs = BeamSplitter::new(5.0);
        for pos in [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 4.0, 0.0),
            Vec3::new(-2.5, 1.0, 0.0),
        ] {
            assert_abs_diff_eq!(bs.sag(pos), 0.0);
        }
    }

    #[test]
    fn norm_is_z_unit_vector() {
        let bs = BeamSplitter::new(5.0);
        for pos in [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, -2.0, 0.0)] {
            let n = bs.norm(pos);
            assert_abs_diff_eq!(n.x(), 0.0);
            assert_abs_diff_eq!(n.y(), 0.0);
            assert_abs_diff_eq!(n.z(), 1.0);
        }
    }

    #[test]
    fn mask_semi_diameter_is_preserved() {
        let bs = BeamSplitter::new(7.5);
        assert_abs_diff_eq!(bs.mask().semi_diameter(), 7.5);
    }

    #[test]
    fn roc_default_is_infinity() {
        let bs = BeamSplitter::new(5.0);
        assert!(bs.roc(0.0).is_infinite());
    }
}
