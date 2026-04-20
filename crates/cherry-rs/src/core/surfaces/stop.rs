use crate::{
    core::{Float, math::vec3::Vec3},
    specs::surfaces::BoundaryType,
};

use super::{Surface, SurfaceKind};

/// An aperture stop — a flat surface that limits the beam.
#[derive(Debug, Clone)]
pub struct Stop {
    pub semi_diameter: Float,
}

impl Stop {
    pub fn new(semi_diameter: Float) -> Self {
        Self { semi_diameter }
    }
}

impl Surface for Stop {
    fn sag(&self, _pos: Vec3) -> Float {
        0.0
    }

    fn norm(&self, _pos: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }

    fn semi_diameter(&self) -> Float {
        self.semi_diameter
    }

    fn boundary_type(&self) -> BoundaryType {
        BoundaryType::NoOp
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Stop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn sag_and_norm_are_always_flat() {
        let stop = Stop::new(5.0);
        for pos in [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 4.0, 0.0),
            Vec3::new(-1.0, 2.5, 0.0),
        ] {
            assert_abs_diff_eq!(stop.sag(pos), 0.0);
            let norm = stop.norm(pos);
            assert_abs_diff_eq!(norm.x(), 0.0);
            assert_abs_diff_eq!(norm.y(), 0.0);
            assert_abs_diff_eq!(norm.z(), 1.0);
        }
    }

    #[test]
    fn semi_diameter_round_trips() {
        let stop = Stop::new(7.5);
        assert_abs_diff_eq!(stop.semi_diameter(), 7.5);
    }

    #[test]
    fn boundary_type_is_noop() {
        let stop = Stop::new(5.0);
        assert!(matches!(stop.boundary_type(), BoundaryType::NoOp));
    }

    #[test]
    fn roc_default_is_infinity() {
        let stop = Stop::new(5.0);
        assert!(stop.roc(0.0).is_infinite());
    }

    #[test]
    fn outside_clear_aperture_default_impl() {
        let stop = Stop::new(5.0);
        assert!(!stop.outside_clear_aperture(Vec3::new(4.9, 0.0, 0.0)));
        assert!(stop.outside_clear_aperture(Vec3::new(5.1, 0.0, 0.0)));
    }
}
