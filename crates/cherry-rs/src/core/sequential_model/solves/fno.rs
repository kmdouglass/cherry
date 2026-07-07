use anyhow::{Result, anyhow};

use crate::{
    core::{
        Float,
        sequential_model::{SequentialSubModel, locate_surface_owner},
    },
    specs::{
        gaps::GapSpec,
        paths::{PathSpec, PathSurfaceRef},
        surfaces::SurfaceSpec,
    },
    views::paraxial::{marginal_ray_bundle, marginal_ray_bundle_for_path},
};

use super::super::SequentialModel;
use super::Solve;

/// Adjusts a surface's radius of curvature so the paraxial marginal ray exits
/// that surface at angle `−1/(2F)`, yielding the target paraxial F-number.
pub struct FNumberSolve {
    surface_index: usize,
    target_fno: Float,
    wavelength_id: usize,
    path_id: usize,
}

impl FNumberSolve {
    pub fn new(surface_index: usize, target_fno: Float, wavelength_id: usize) -> Self {
        Self {
            surface_index,
            target_fno,
            wavelength_id,
            path_id: 0,
        }
    }

    /// Targets path `path_id` in a multipath model. Defaults to 0; has no
    /// effect on single-path models.
    pub fn with_path_id(mut self, path_id: usize) -> Self {
        self.path_id = path_id;
        self
    }

    /// Computes the radius of curvature that yields `target_fno`, given the
    /// marginal ray's incoming angle/height and the refractive indices on
    /// either side. Shared by `apply` and `apply_multipath`.
    fn solved_roc(
        prev_angle: Float,
        height: Float,
        n_0: Float,
        n_1: Float,
        target_fno: Float,
        surface_index: usize,
    ) -> Result<Float> {
        let eps = Float::EPSILON * height.abs().max(1.0);
        if height.abs() < eps {
            return Err(anyhow!(
                "marginal ray height at surface {surface_index} is effectively zero; \
                 ROC is indeterminate"
            ));
        }

        let denom = n_1 / (2.0 * target_fno) + n_0 * prev_angle;
        let roc = if denom.abs() < eps {
            Float::INFINITY
        } else {
            (n_1 - n_0) * height / denom
        };

        if roc == 0.0 {
            return Err(anyhow!(
                "computed ROC is zero at surface {surface_index}; result is unphysical"
            ));
        }
        Ok(roc)
    }

    /// Writes `roc` into `spec` if it is a `Sphere` or `Conic`. Shared by
    /// `apply` and `apply_multipath`.
    fn write_roc(spec: &mut SurfaceSpec, roc: Float, surface_index: usize) -> Result<()> {
        match spec {
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
                "surface {surface_index} is not a Sphere or Conic; cannot set radius of curvature"
            )),
        }
    }
}

impl Solve for FNumberSolve {
    fn path_id(&self) -> usize {
        self.path_id
    }

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

        let roc = Self::solved_roc(u, y, n_0, n_1, self.target_fno, self.surface_index)?;
        Self::write_roc(
            &mut surface_specs[self.surface_index],
            roc,
            self.surface_index,
        )
    }

    fn apply_multipath(&self, model: &SequentialModel, paths: &mut [PathSpec]) -> Result<()> {
        let path_id = self.path_id();
        if path_id >= paths.len() {
            return Err(anyhow!(
                "solve targets path_id {path_id} but the model has only {} path(s)",
                paths.len()
            ));
        }

        let store_index = self.surface_index;
        let path_surface_indices = model.path_surface_indices(path_id);
        let mut occurrences = path_surface_indices
            .iter()
            .enumerate()
            .filter(|&(_, &si)| si == store_index)
            .map(|(step, _)| step);
        let eval_step = match (occurrences.next(), occurrences.next()) {
            (None, _) => {
                return Err(anyhow!(
                    "path {path_id} does not visit surface {store_index}"
                ));
            }
            (Some(_), Some(_)) => {
                return Err(anyhow!(
                    "surface {store_index} is visited more than once by path {path_id}; \
                     ambiguous which occurrence to evaluate the solve against"
                ));
            }
            (Some(step), None) => step,
        };
        let prev_step = eval_step.checked_sub(1).ok_or_else(|| {
            anyhow!(
                "surface {store_index} is the first step of path {path_id}; \
                 cannot apply F/# solve at the object surface"
            )
        })?;

        let n_wavelengths = model.wavelengths_for_path(path_id).len();
        if self.wavelength_id >= n_wavelengths {
            return Err(anyhow!(
                "wavelength_id {} is out of range (path {path_id} has {} wavelength(s))",
                self.wavelength_id,
                n_wavelengths
            ));
        }

        let bundle = marginal_ray_bundle_for_path(model, path_id, self.wavelength_id)?;
        let u = bundle.rays_at_surface(prev_step)[0].angle;
        let y = bundle.rays_at_surface(eval_step)[0].height;

        let submodel = model
            .submodels_for_path(path_id)
            .get(self.wavelength_id)
            .ok_or_else(|| anyhow!("wavelength_id {} out of range", self.wavelength_id))?;
        let gaps = submodel.gaps();
        let n_0 = gaps
            .get(prev_step)
            .ok_or_else(|| anyhow!("no gap before step {prev_step} on path {path_id}"))?
            .refractive_index
            .n();
        let n_1 = gaps
            .get(eval_step)
            .ok_or_else(|| anyhow!("no gap after step {eval_step} on path {path_id}"))?
            .refractive_index
            .n();

        let roc = Self::solved_roc(u, y, n_0, n_1, self.target_fno, store_index)?;

        let (owner_path, owner_step) =
            locate_surface_owner(paths, store_index).ok_or_else(|| {
                anyhow!("surface {store_index} is not introduced by any path (internal error)")
            })?;
        let spec = match &mut paths[owner_path].surface_refs[owner_step] {
            PathSurfaceRef::New(spec) => spec,
            _ => {
                return Err(anyhow!(
                    "surface {store_index} (owned by path {owner_path}, step {owner_step}) is \
                     not a New surface_ref; Shared/ObjectLinkedTo surfaces cannot be solved \
                     directly"
                ));
            }
        };
        Self::write_roc(spec, roc, store_index)
    }

    fn surface_index(&self) -> usize {
        self.surface_index
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use crate::{
        GapSpec, Rotation3D, SequentialModel, SurfaceSpec, Vec3,
        core::{Float, sequential_model::builder::SequentialModelBuilder},
        n,
        specs::{fields::FieldSpec, surfaces::BoundaryKind},
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
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Sphere {
                semi_diameter: 12.5,
                radius_of_curvature: Float::INFINITY,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
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
            .expect("build should succeed")
            .model;
        let pv = ParaxialView::new(&model, &field_specs(), false).unwrap();
        (model, pv)
    }

    #[test]
    fn paraxial_fno_matches_target() {
        let target = 4.0;
        let (_, pv) = build_with_fno_solve(target);
        let sub = pv.get(0, 0).unwrap();
        assert_abs_diff_eq!(sub.paraxial_fno().abs(), target, epsilon = 1e-4);
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
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Iris {
                semi_diameter: 10.0,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
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
            .expect("build should succeed")
            .model;

        let pv = ParaxialView::new(&model, &field_specs(), false).unwrap();
        let sub = pv.get(0, 0).unwrap();

        // F/# constraint satisfied.
        assert_abs_diff_eq!(sub.paraxial_fno().abs(), target_fno, epsilon = 1e-3);

        // MarginalRaySolve constraint: marginal ray height at image ≈ 0.
        let bundle = marginal_ray_bundle(&model, 0).unwrap();
        assert_abs_diff_eq!(bundle.rays_at_surface(3)[0].height, 0.0, epsilon = 1e-3);
    }

    // ── Per-path solves (multipath) ────────────────────────────────────────

    /// Two paths sharing one refracting Sphere (store index 1). Path 0's own
    /// Iris (SD=3) is more constraining than the Sphere (SD=10), so path 0's
    /// aperture stop is its own Iris; path 1 has no Iris, so the shared
    /// Sphere is path 1's aperture stop.
    ///
    /// Store: Object(0), Sphere(1), Iris(2), Image_0(3), Image_1(4).
    fn shared_sphere_two_path_fixture() -> (PathSpec, PathSpec) {
        use crate::specs::paths::PathSurfaceRef;

        let img = || SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let path0 = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::New(SurfaceSpec::Object),
                PathSurfaceRef::New(SurfaceSpec::Sphere {
                    semi_diameter: 10.0,
                    radius_of_curvature: 100.0, // placeholder; solved below
                    surf_kind: BoundaryKind::Refracting,
                    rotation: Rotation3D::None,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
                PathSurfaceRef::New(SurfaceSpec::Iris {
                    semi_diameter: 3.0,
                    rotation: Rotation3D::None,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
                PathSurfaceRef::New(img()),
            ],
            gaps: vec![
                GapSpec {
                    thickness: Float::INFINITY,
                    refractive_index: n!(1.0),
                },
                GapSpec {
                    thickness: 50.0,
                    refractive_index: n!(1.5),
                },
                GapSpec {
                    thickness: 50.0,
                    refractive_index: n!(1.5),
                },
            ],
            beam_splitter_arms: vec![],
            stop_surface: None,
            wavelengths: vec![0.5876],
        };
        // Path 1's own gaps around the shared Sphere: glass before, air after
        // (image space), so its image-space index matches the assumption
        // `FNumberSolve::solved_roc` makes (exit angle == -1/(2*target_fno)
        // directly, without an image-space index correction) — the same
        // assumption the single-path convexplano fixture above satisfies by
        // using air as its last gap's medium.
        let path1 = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::Shared(0),
                PathSurfaceRef::Shared(1),
                PathSurfaceRef::New(img()),
            ],
            gaps: vec![
                GapSpec {
                    thickness: Float::INFINITY,
                    refractive_index: n!(1.5),
                },
                GapSpec {
                    thickness: 50.0,
                    refractive_index: n!(1.0),
                },
            ],
            beam_splitter_arms: vec![],
            stop_surface: None,
            wavelengths: vec![0.5876],
        };
        (path0, path1)
    }

    #[test]
    fn multipath_fno_solve_on_shared_surface_updates_both_paths() {
        let (path0, path1) = shared_sphere_two_path_fixture();
        let target = 4.0;

        let model = SequentialModelBuilder::new()
            .paths(vec![path0, path1])
            .solves(vec![Box::new(
                FNumberSolve::new(1, target, 0).with_path_id(1),
            )])
            .build()
            .expect("build should succeed")
            .model;

        let field_specs = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let pv = ParaxialView::new(&model, &field_specs, false).unwrap();

        // Path 1 (the evaluation path): F/# matches the target directly.
        let sub1 = pv.get_for_path(1, 0, 0).unwrap();
        assert_abs_diff_eq!(sub1.paraxial_fno().abs(), target, epsilon = 1e-3);

        // Path 0 sees the *same physical surface* — same store index, same
        // curvature — but its own stop is its Iris, not the shared Sphere,
        // so its own F/# generically differs from path 1's target (proving
        // this was evaluated against path 1's own aperture, not path 0's).
        let sub0 = pv.get_for_path(0, 0, 0).unwrap();
        assert!((sub0.paraxial_fno().abs() - target).abs() > 1e-2);

        // Both paths read the identical, updated ROC off the one physical surface.
        let store_idx = model.path_surface_indices(1)[1];
        assert_eq!(model.path_surface_indices(0)[1], store_idx);
        let roc = model.surfaces()[store_idx].roc(0.0);
        assert!(roc.is_finite() && roc != 100.0); // changed from the placeholder
    }

    #[test]
    fn multipath_fno_solve_errors_if_path_does_not_visit_surface() {
        let (path0, path1) = shared_sphere_two_path_fixture();
        // path0's own Iris (store index 2) is never visited by path 1.
        let result = SequentialModelBuilder::new()
            .paths(vec![path0, path1])
            .solves(vec![Box::new(FNumberSolve::new(2, 4.0, 0).with_path_id(1))])
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn multipath_fno_solve_errors_on_repeated_store_index_visit() {
        use crate::specs::paths::PathSurfaceRef;

        // Path: [New(Obj), New(Sphere), Shared(1), New(Img)] — surface_indices = [0, 1,
        // 1, 2].
        let path = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::New(SurfaceSpec::Object),
                PathSurfaceRef::New(SurfaceSpec::Sphere {
                    semi_diameter: 12.7,
                    radius_of_curvature: 65.0,
                    surf_kind: BoundaryKind::Refracting,
                    rotation: Rotation3D::None,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
                PathSurfaceRef::Shared(1),
                PathSurfaceRef::New(SurfaceSpec::Image {
                    rotation: Rotation3D::None,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
            ],
            gaps: vec![
                GapSpec {
                    thickness: Float::INFINITY,
                    refractive_index: n!(1.0),
                },
                GapSpec {
                    thickness: 10.0,
                    refractive_index: n!(1.5),
                },
                GapSpec {
                    thickness: 10.0,
                    refractive_index: n!(1.5),
                },
            ],
            beam_splitter_arms: vec![],
            stop_surface: None,
            wavelengths: vec![0.5876],
        };
        let result = SequentialModelBuilder::new()
            .paths(vec![path])
            .solves(vec![Box::new(FNumberSolve::new(1, 4.0, 0))])
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn two_curvature_solves_on_the_same_store_index_are_rejected() {
        // Two paths sharing one Sphere (store index 1); both declare an
        // FNumberSolve on it — rejected regardless of differing path_id.
        let (path0, path1) = shared_sphere_two_path_fixture();
        let result = SequentialModelBuilder::new()
            .paths(vec![path0, path1])
            .solves(vec![
                Box::new(FNumberSolve::new(1, 4.0, 0)), // path_id defaults to 0
                Box::new(FNumberSolve::new(1, 8.0, 0).with_path_id(1)), /* different path_id,
                                                         * same target */
            ])
            .build();
        assert!(result.is_err());
    }
}
