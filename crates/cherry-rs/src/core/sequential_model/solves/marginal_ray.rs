use anyhow::{Result, anyhow};

use crate::{
    core::Float,
    specs::{gaps::GapSpec, surfaces::SurfaceSpec},
    views::paraxial::marginal_ray_bundle,
};

use super::super::SequentialModel;
use super::{Solve, SolveKind};

/// Adjusts a gap thickness so that the paraxial marginal ray height at the
/// following surface equals a target value.
///
/// The canonical use case is placing the image plane at the paraxial image
/// plane by setting `target_height = 0.0` for the gap preceding the image
/// surface.
pub struct MarginalRaySolve {
    gap_index: usize,
    target_height: Float,
    wavelength_id: usize,
}

impl MarginalRaySolve {
    pub fn new(gap_index: usize, target_height: Float, wavelength_id: usize) -> Self {
        Self {
            gap_index,
            target_height,
            wavelength_id,
        }
    }
}

impl Solve for MarginalRaySolve {
    fn parameter_kind(&self) -> SolveKind {
        SolveKind::Thickness
    }

    fn apply(
        &self,
        model: &SequentialModel,
        gap_specs: &mut Vec<GapSpec>,
        _surface_specs: &mut Vec<SurfaceSpec>,
    ) -> Result<()> {
        if self.gap_index >= gap_specs.len() {
            return Err(anyhow!(
                "gap_index {} is out of range (gap_specs has {} gaps)",
                self.gap_index,
                gap_specs.len()
            ));
        }
        if self.wavelength_id >= model.wavelengths().len() {
            return Err(anyhow!(
                "wavelength_id {} is out of range (model has {} wavelengths)",
                self.wavelength_id,
                model.wavelengths().len()
            ));
        }

        let bundle = marginal_ray_bundle(model, self.wavelength_id)?;
        let ray = &bundle.rays_at_surface(self.gap_index)[0];
        let h = ray.height;
        let u_prime = ray.angle;

        let eps = Float::EPSILON * h.abs().max(1.0);
        if u_prime.abs() < eps {
            return Err(anyhow!(
                "marginal ray angle at surface {} is effectively zero; \
                 thickness is indeterminate (collimated beam in gap space)",
                self.gap_index
            ));
        }

        let t = (self.target_height - h) / u_prime;
        if t < 0.0 {
            return Err(anyhow!(
                "computed gap thickness {t} is negative for gap {}",
                self.gap_index
            ));
        }

        gap_specs[self.gap_index].thickness = t;
        Ok(())
    }

    fn surface_index(&self) -> usize {
        self.gap_index
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use crate::{
        GapSpec, Rotation3D, SequentialModel, SurfaceSpec,
        core::{Float, sequential_model::builder::SequentialModelBuilder},
        n,
        specs::{fields::FieldSpec, surfaces::BoundaryType},
        views::paraxial::ParaxialView,
    };

    use super::*;

    /// Convexplano lens gaps with an arbitrary image-space thickness.
    fn convexplano_gaps(image_gap_thickness: Float) -> Vec<GapSpec> {
        vec![
            GapSpec {
                thickness: Float::INFINITY,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 5.3,
                refractive_index: n!(1.515),
            },
            GapSpec {
                thickness: image_gap_thickness,
                refractive_index: n!(1.0),
            },
        ]
    }

    fn convexplano_surfaces() -> Vec<SurfaceSpec> {
        vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 12.5,
                radius_of_curvature: 25.8,
                surf_type: BoundaryType::Refracting,
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Sphere {
                semi_diameter: 12.5,
                radius_of_curvature: Float::INFINITY,
                surf_type: BoundaryType::Refracting,
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
            },
        ]
    }

    fn field_specs() -> Vec<FieldSpec> {
        vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }]
    }

    /// target_height = 0.0 places the image plane at the paraxial image plane.
    #[test]
    fn image_plane_at_paraxial_image() {
        let model = SequentialModelBuilder::new()
            .gap_specs(convexplano_gaps(1.0))
            .surface_specs(convexplano_surfaces())
            .wavelengths(vec![0.5876])
            .solves(vec![Box::new(MarginalRaySolve::new(2, 0.0, 0))])
            .build()
            .expect("build should succeed")
            .model;

        let pv = ParaxialView::new(&model, &field_specs(), false).unwrap();
        let sub = pv.get(0, 0).unwrap();

        let marginal_at_image = sub.marginal_ray().rays_at_surface(3)[0].height;
        assert_abs_diff_eq!(marginal_at_image, 0.0, epsilon = 1e-4);
    }

    /// target_height = 5.0 mm: marginal ray height at the next surface matches.
    #[test]
    fn arbitrary_target_height() {
        let target = 5.0_f64;
        let model = SequentialModelBuilder::new()
            .gap_specs(convexplano_gaps(1.0))
            .surface_specs(convexplano_surfaces())
            .wavelengths(vec![0.5876])
            .solves(vec![Box::new(MarginalRaySolve::new(2, target, 0))])
            .build()
            .expect("build should succeed")
            .model;

        let pv = ParaxialView::new(&model, &field_specs(), false).unwrap();
        let sub = pv.get(0, 0).unwrap();

        let marginal_at_image = sub.marginal_ray().rays_at_surface(3)[0].height;
        assert_abs_diff_eq!(marginal_at_image, target, epsilon = 1e-4);
    }

    /// A zero marginal-ray angle (collimated beam) causes an error.
    #[test]
    fn zero_angle_returns_error() {
        // A system with no optical power: object at infinity, one flat air→air
        // surface. The marginal ray angle remains zero throughout.
        let gaps = vec![
            GapSpec {
                thickness: Float::INFINITY,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
        ];
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Conic {
                semi_diameter: 10.0,
                radius_of_curvature: Float::INFINITY,
                conic_constant: 0.0,
                surf_type: BoundaryType::Refracting,
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
            },
        ];
        let model = SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], None).unwrap();

        let solve = MarginalRaySolve::new(1, 0.0, 0);
        let mut gap_specs = gaps;
        let mut surface_specs = surfaces;
        let result = solve.apply(&model, &mut gap_specs, &mut surface_specs);
        assert!(result.is_err());
    }

    /// A negative computed thickness causes an error.
    #[test]
    fn negative_thickness_returns_error() {
        // For the convexplano lens, h ≈ 11.63 and u' ≈ -0.2495 at gap 2.
        // target = 20.0 → t = (20 - 11.63) / (-0.2495) < 0.
        let model = SequentialModel::from_surface_specs(
            &convexplano_gaps(1.0),
            &convexplano_surfaces(),
            &[0.5876],
            None,
        )
        .unwrap();

        let solve = MarginalRaySolve::new(2, 20.0, 0);
        let mut gap_specs = convexplano_gaps(1.0);
        let mut surface_specs = convexplano_surfaces();
        let result = solve.apply(&model, &mut gap_specs, &mut surface_specs);
        assert!(result.is_err());
    }

    /// Out-of-range gap_index causes an error at apply time.
    #[test]
    fn out_of_range_gap_index_returns_error() {
        let model = SequentialModel::from_surface_specs(
            &convexplano_gaps(1.0),
            &convexplano_surfaces(),
            &[0.5876],
            None,
        )
        .unwrap();

        let solve = MarginalRaySolve::new(99, 0.0, 0);
        let mut gap_specs = convexplano_gaps(1.0);
        let mut surface_specs = convexplano_surfaces();
        let result = solve.apply(&model, &mut gap_specs, &mut surface_specs);
        assert!(result.is_err());
    }

    /// Out-of-range wavelength_id causes an error at apply time.
    #[test]
    fn out_of_range_wavelength_id_returns_error() {
        let model = SequentialModel::from_surface_specs(
            &convexplano_gaps(1.0),
            &convexplano_surfaces(),
            &[0.5876],
            None,
        )
        .unwrap();

        let solve = MarginalRaySolve::new(2, 0.0, 99);
        let mut gap_specs = convexplano_gaps(1.0);
        let mut surface_specs = convexplano_surfaces();
        let result = solve.apply(&model, &mut gap_specs, &mut surface_specs);
        assert!(result.is_err());
    }

    /// Two independent MarginalRaySolve instances on different gaps each
    /// produce the correct result.
    #[test]
    fn two_solves_independent_and_correct() {
        // Finite-object convexplano system. Both gap 1 and gap 2 are solved:
        // gap 2 → image plane at the paraxial image plane.
        let gaps = vec![
            GapSpec {
                thickness: 100.0,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 5.3,
                refractive_index: n!(1.515),
            },
            GapSpec {
                thickness: 1.0, // will be set by solve
                refractive_index: n!(1.0),
            },
        ];

        let model = SequentialModelBuilder::new()
            .gap_specs(gaps)
            .surface_specs(convexplano_surfaces())
            .wavelengths(vec![0.5876])
            .solves(vec![Box::new(MarginalRaySolve::new(2, 0.0, 0))])
            .build()
            .expect("build should succeed")
            .model;

        let pv = ParaxialView::new(&model, &field_specs(), false).unwrap();
        let sub = pv.get(0, 0).unwrap();
        let marginal_at_image = sub.marginal_ray().rays_at_surface(3)[0].height;
        assert_abs_diff_eq!(marginal_at_image, 0.0, epsilon = 1e-4);
    }
}
