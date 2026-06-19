/// A thin lens surface type for 3D ray tracing.
use crate::core::{
    Float,
    math::vec3::Vec3,
    surfaces::{BoundaryKind, Mask, Surface, SurfaceKind, solvers::flat_surface},
};

/// A thin lens for 3D ray tracing.
///
/// The focal length of the thin lens is specified as that when the refractive
/// index of both sides of the lens is in air (1.0).
#[derive(Debug)]
pub struct ThinLens {
    pub focal_length: Float,
    mask: Mask,
}

impl ThinLens {
    pub fn new(semi_diameter: Float, focal_length: Float) -> Self {
        Self {
            focal_length,
            mask: Mask::Circular { semi_diameter },
        }
    }
}

impl Surface for ThinLens {
    fn boundary_kind(&self) -> BoundaryKind {
        BoundaryKind::Refracting
    }

    fn interact(&self, ray: &mut crate::Ray, n_0: Float, n_1: Float, _norm: crate::Vec3) {
        let power = self.power(0.0, n_0, n_1);
        let ux_0 = ray.dir().l() / ray.dir().n();
        let uy_0 = ray.dir().m() / ray.dir().n();

        // Paraxial ray transer equations for a thin lens
        let ux_1 = (n_0 * ux_0 - power * ray.pos().x()) / n_1;
        let uy_1 = (n_0 * uy_0 - power * ray.pos().y()) / n_1;

        // l_1 = ux_1 * dir_cos_n_1, m_1 = uy_1 * dir_cos_n_1, and
        // l_1^2 + m_1^2 + dir_cos_n_1^2 = 1, so:
        // dir_cos_n_1^2 * (ux_1^2 + uy_1^2 + 1) = 1
        let dir_cos_n_1 = (1.0 + ux_1 * ux_1 + uy_1 * uy_1).powf(-0.5);
        let dir_cos_l_1 = ux_1 * dir_cos_n_1;
        let dir_cos_m_1 = uy_1 * dir_cos_n_1;

        ray.set_dir(Vec3::new(dir_cos_l_1, dir_cos_m_1, dir_cos_n_1).normalize());
    }

    fn intersect(
        &self,
        ray: &crate::core::ray::Ray,
        max_iter: usize,
    ) -> anyhow::Result<(Vec3, Vec3)> {
        flat_surface(ray, self, max_iter)
    }

    fn mask(&self) -> &Mask {
        &self.mask
    }

    fn power(&self, _azimuth_rad: Float, _n_0: Float, _n_1: Float) -> Float {
        1.0 / self.focal_length
    }

    fn norm(&self, _pos: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }

    fn sag(&self, _pos: crate::Vec3) -> Float {
        0.0
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::ThinLens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ray::Ray;
    use approx::assert_abs_diff_eq;

    /// Tests a ray incident at 10 degrees at a height of 10 mm on a thin lens
    /// with a focal length of 100 mm.
    #[test]
    fn test_ray_interact() {
        let focal_length = 100.0;
        let semi_diameter = 25.0;

        let lens = ThinLens::new(semi_diameter, focal_length);
        let mut ray = Ray::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(
                0.0,
                (80.0_f64).to_radians().cos(),
                (10.0_f64).to_radians().cos(),
            )
            .normalize(),
        );

        let norm = lens.norm(ray.pos());
        assert_abs_diff_eq!(norm.x(), 0.0, epsilon = 1e-6);
        assert_abs_diff_eq!(norm.y(), 0.0, epsilon = 1e-6);
        assert_abs_diff_eq!(norm.z(), 1.0, epsilon = 1e-6);

        lens.interact(&mut ray, 1.0, 1.0, norm);
        let dir = ray.dir();
        assert_abs_diff_eq!(dir.x(), 0.0, epsilon = 1e-6);
        assert_abs_diff_eq!(dir.y(), 0.07610561, epsilon = 1e-6);
        assert_abs_diff_eq!(dir.z(), 0.99709976, epsilon = 1e-6);
    }

    /// A thin lens's power is `1/focal_length` regardless of the surrounding
    /// media — unlike a curved refracting surface, it isn't derived from
    /// `roc()` and `n_0`/`n_1`.
    #[test]
    fn test_power_is_media_independent() {
        let lens = ThinLens::new(25.0, 100.0);
        let expected = 1.0 / 100.0;
        assert_abs_diff_eq!(lens.power(0.0, 1.0, 1.0), expected, epsilon = 1e-12);
        assert_abs_diff_eq!(lens.power(0.0, 1.0, 1.5), expected, epsilon = 1e-12);
        assert_abs_diff_eq!(lens.power(1.23, 1.0, 1.0), expected, epsilon = 1e-12);
    }
}
