use anyhow::Result;

use crate::{
    core::{Float, math::vec3::Vec3, ray::Ray},
    specs::surfaces::{BeamSplitterPathKind, BoundaryKind, Mask},
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

    fn effective_boundary_kind(&self, bs_arm: Option<BeamSplitterPathKind>) -> BoundaryKind {
        match bs_arm {
            Some(BeamSplitterPathKind::Reflecting) => BoundaryKind::Reflecting,
            Some(BeamSplitterPathKind::Transmitting) => BoundaryKind::Refracting,
            // No per-step context available: fall back to today's
            // context-free answer.
            None => BoundaryKind::Refracting,
        }
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

    #[test]
    fn effective_boundary_kind_respects_reflecting_arm() {
        let bs = BeamSplitter::new(5.0);
        assert!(matches!(
            bs.effective_boundary_kind(Some(BeamSplitterPathKind::Reflecting)),
            BoundaryKind::Reflecting
        ));
    }

    #[test]
    fn effective_boundary_kind_respects_transmitting_arm() {
        let bs = BeamSplitter::new(5.0);
        assert!(matches!(
            bs.effective_boundary_kind(Some(BeamSplitterPathKind::Transmitting)),
            BoundaryKind::Refracting
        ));
    }

    #[test]
    fn effective_boundary_kind_falls_back_to_refracting_without_context() {
        let bs = BeamSplitter::new(5.0);
        assert!(matches!(
            bs.effective_boundary_kind(None),
            BoundaryKind::Refracting
        ));
    }

    /// AT: the direct unit-level regression test for the real-ray-tracer
    /// bug — a non-degenerate (oblique) ray through a `Reflecting`-arm
    /// beam splitter must obey the law of reflection, not pass straight
    /// through as Snell's law with matched indices always does.
    #[test]
    fn interact_reflects_when_arm_is_reflecting() {
        let bs = BeamSplitter::new(5.0);
        let dir_in = Vec3::new(0.0, 1.0, 1.0).normalize();
        let norm = bs.norm(Vec3::new(0.0, 0.0, 0.0));
        let mut ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), dir_in);

        bs.interact(
            &mut ray,
            1.0,
            1.0,
            norm,
            Some(BeamSplitterPathKind::Reflecting),
        );

        let cos_theta = dir_in.dot(&norm);
        let expected = dir_in - norm * (2.0 * cos_theta);
        assert_abs_diff_eq!(ray.dir().x(), expected.x(), epsilon = 1e-9);
        assert_abs_diff_eq!(ray.dir().y(), expected.y(), epsilon = 1e-9);
        assert_abs_diff_eq!(ray.dir().z(), expected.z(), epsilon = 1e-9);

        // Must actually have turned, not passed straight through. The
        // normal is along z, so reflection flips the z-component while
        // leaving the tangential (y) component unchanged — check z.
        assert!(
            (ray.dir().z() - dir_in.z()).abs() > 0.1,
            "reflected ray direction must differ from the incident direction \
             for oblique incidence"
        );
    }

    /// Companion case: `Transmitting` arm with matched indices on both
    /// sides passes the ray straight through undeviated (the correct,
    /// expected result for this arm — unlike the `Reflecting` case above,
    /// this one already worked before the fix).
    #[test]
    fn interact_transmits_undeviated_when_arm_is_transmitting() {
        let bs = BeamSplitter::new(5.0);
        let dir_in = Vec3::new(0.0, 1.0, 1.0).normalize();
        let norm = bs.norm(Vec3::new(0.0, 0.0, 0.0));
        let mut ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), dir_in);

        bs.interact(
            &mut ray,
            1.0,
            1.0,
            norm,
            Some(BeamSplitterPathKind::Transmitting),
        );

        assert_abs_diff_eq!(ray.dir().x(), dir_in.x(), epsilon = 1e-9);
        assert_abs_diff_eq!(ray.dir().y(), dir_in.y(), epsilon = 1e-9);
        assert_abs_diff_eq!(ray.dir().z(), dir_in.z(), epsilon = 1e-9);
    }
}
