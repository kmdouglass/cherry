use anyhow::{Result, anyhow};

use crate::{
    core::{Float, sequential_model::SequentialSubModel},
    specs::{gaps::GapSpec, surfaces::SurfaceSpec},
    views::paraxial::marginal_ray_bundle,
};

use super::super::SequentialModel;
use super::Solve;

/// Adjusts a surface's radius of curvature so the paraxial marginal ray exits
/// that surface at angle `−1/(2F)`, yielding the target paraxial F-number.
pub struct FNumberSolve {
    surface_index: usize,
    target_fno: Float,
    wavelength_id: usize,
}

impl FNumberSolve {
    pub fn new(surface_index: usize, target_fno: Float, wavelength_id: usize) -> Self {
        Self {
            surface_index,
            target_fno,
            wavelength_id,
        }
    }
}

impl Solve for FNumberSolve {
    fn apply(
        &self,
        model: &SequentialModel,
        _gap_specs: &mut Vec<GapSpec>,
        surface_specs: &mut Vec<SurfaceSpec>,
    ) -> Result<()> {
        if self.surface_index >= surface_specs.len() {
            return Err(anyhow!(
                "surface_index {} is out of range (surface_specs has {} surfaces)",
                self.surface_index,
                surface_specs.len()
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

        // incoming angle: state before refraction at surface_index is stored at
        // surface_index − 1
        let prev_idx = self.surface_index.checked_sub(1).ok_or_else(|| {
            anyhow!("surface_index 0 is the object surface; cannot apply F/# solve")
        })?;
        let u = bundle.rays_at_surface(prev_idx)[0].angle;
        let y = bundle.rays_at_surface(self.surface_index)[0].height;

        let eps = Float::EPSILON * y.abs().max(1.0);
        if y.abs() < eps {
            return Err(anyhow!(
                "marginal ray height at surface {} is effectively zero; \
                 ROC is indeterminate",
                self.surface_index
            ));
        }

        let submodel = model
            .submodel(self.wavelength_id)
            .ok_or_else(|| anyhow!("wavelength_id {} out of range", self.wavelength_id))?;
        let gaps = submodel.gaps();
        let n_0 = gaps
            .get(prev_idx)
            .ok_or_else(|| anyhow!("no gap before surface {}", self.surface_index))?
            .refractive_index
            .n();
        let n_1 = gaps
            .get(self.surface_index)
            .ok_or_else(|| anyhow!("no gap after surface {}", self.surface_index))?
            .refractive_index
            .n();

        let denom = n_1 / (2.0 * self.target_fno) + n_0 * u;
        let roc = if denom.abs() < eps {
            Float::INFINITY
        } else {
            (n_1 - n_0) * y / denom
        };

        if roc == 0.0 {
            return Err(anyhow!(
                "computed ROC is zero at surface {}; result is unphysical",
                self.surface_index
            ));
        }

        match &mut surface_specs[self.surface_index] {
            SurfaceSpec::Sphere {
                radius_of_curvature,
                ..
            }
            | SurfaceSpec::Conic {
                radius_of_curvature,
                ..
            } => {
                *radius_of_curvature = roc;
                Ok(())
            }
            _ => Err(anyhow!(
                "surface {} is not a Sphere or Conic; cannot set radius of curvature",
                self.surface_index
            )),
        }
    }

    fn surface_index(&self) -> usize {
        self.surface_index
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
        views::paraxial::{ParaxialView, marginal_ray_bundle},
    };

    use super::*;

    fn convexplano_gaps() -> Vec<GapSpec> {
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
                thickness: 46.2,
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

    // Build a model + ParaxialView with an F/# solve on surface 2 (last physical).
    fn build_with_fno_solve(target_fno: Float) -> (SequentialModel, ParaxialView) {
        let model = SequentialModelBuilder::new()
            .gap_specs(convexplano_gaps())
            .surface_specs(convexplano_surfaces())
            .wavelengths(vec![0.5876])
            .solves(vec![Box::new(FNumberSolve::new(2, target_fno, 0))])
            .build()
            .expect("build should succeed");
        let pv = ParaxialView::new(&model, &field_specs(), false).unwrap();
        (model, pv)
    }

    #[test]
    fn paraxial_fno_matches_target() {
        let target = 4.0;
        let (_, pv) = build_with_fno_solve(target);
        let sub = pv.get(0, 0).unwrap();
        assert_abs_diff_eq!(sub.paraxial_fno(), target, epsilon = 1e-4);
    }

    #[test]
    fn marginal_ray_exit_angle_matches_target() {
        let target = 4.0;
        let (model, _) = build_with_fno_solve(target);
        let bundle = marginal_ray_bundle(&model, 0).unwrap();
        // Surface 2 is the last physical surface; exit angle should be −1/(2F).
        let angle = bundle.rays_at_surface(2)[0].angle;
        assert_abs_diff_eq!(angle, -1.0 / (2.0 * target), epsilon = 1e-4);
    }

    #[test]
    fn out_of_range_surface_index_returns_error() {
        let model = SequentialModel::from_surface_specs(
            &convexplano_gaps(),
            &convexplano_surfaces(),
            &[0.5876],
            None,
        )
        .unwrap();
        let solve = FNumberSolve::new(99, 4.0, 0);
        let mut gap_specs = convexplano_gaps();
        let mut surface_specs = convexplano_surfaces();
        assert!(
            solve
                .apply(&model, &mut gap_specs, &mut surface_specs)
                .is_err()
        );
    }

    #[test]
    fn out_of_range_wavelength_id_returns_error() {
        let model = SequentialModel::from_surface_specs(
            &convexplano_gaps(),
            &convexplano_surfaces(),
            &[0.5876],
            None,
        )
        .unwrap();
        let solve = FNumberSolve::new(2, 4.0, 99);
        let mut gap_specs = convexplano_gaps();
        let mut surface_specs = convexplano_surfaces();
        assert!(
            solve
                .apply(&model, &mut gap_specs, &mut surface_specs)
                .is_err()
        );
    }

    #[test]
    fn non_conic_surface_returns_error() {
        // Iris surface at index 2 — no radius of curvature.
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 12.5,
                radius_of_curvature: 25.8,
                surf_type: BoundaryType::Refracting,
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Iris {
                semi_diameter: 10.0,
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
            },
        ];
        let model =
            SequentialModel::from_surface_specs(&convexplano_gaps(), &surfaces, &[0.5876], None)
                .unwrap();
        let solve = FNumberSolve::new(2, 4.0, 0);
        let mut gap_specs = convexplano_gaps();
        let mut surface_specs = surfaces;
        assert!(
            solve
                .apply(&model, &mut gap_specs, &mut surface_specs)
                .is_err()
        );
    }

    #[test]
    fn fno_and_marginal_ray_solves_compose() {
        use crate::core::sequential_model::solves::MarginalRaySolve;

        let target_fno = 3.0;
        let model = SequentialModelBuilder::new()
            .gap_specs(convexplano_gaps())
            .surface_specs(convexplano_surfaces())
            .wavelengths(vec![0.5876])
            .solves(vec![
                Box::new(FNumberSolve::new(2, target_fno, 0)),
                Box::new(MarginalRaySolve::new(2, 0.0, 0)),
            ])
            .build()
            .expect("build should succeed");

        let pv = ParaxialView::new(&model, &field_specs(), false).unwrap();
        let sub = pv.get(0, 0).unwrap();

        // F/# constraint satisfied.
        assert_abs_diff_eq!(sub.paraxial_fno(), target_fno, epsilon = 1e-3);

        // MarginalRaySolve constraint: marginal ray height at image ≈ 0.
        let bundle = marginal_ray_bundle(&model, 0).unwrap();
        assert_abs_diff_eq!(bundle.rays_at_surface(3)[0].height, 0.0, epsilon = 1e-3);
    }
}
