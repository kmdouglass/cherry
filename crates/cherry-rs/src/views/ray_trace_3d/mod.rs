/// Performs a 3D ray trace on the system.
mod aiming;
mod trace;

use anyhow::{Result, anyhow};
use rayon::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;
use tracing::trace;

use crate::{
    core::{
        Float,
        sequential_model::{
            CursorPlacement, SequentialModel, SequentialSubModel,
            surface_placement::SurfacePlacement,
        },
        surfaces::Surface,
    },
    specs::{
        aperture::ApertureSpec,
        fields::{FieldSpec, PupilSampling},
        surfaces::BeamSplitterPathKind,
    },
};

use aiming::rays;
use trace::trace;

pub use trace::RayBundle;

use super::paraxial::{ParaxialSubView, ParaxialView};

/// Configuration for the pupil sampling used in a 3D ray trace.
#[derive(Debug, Clone, Copy)]
pub struct SamplingConfig {
    /// Number of rays in the tangential and sagittal ray fans.
    pub n_fan_rays: usize,
    /// Grid spacing for the full-pupil square-grid sampling, in normalised
    /// pupil coordinates [0, 1].
    pub full_pupil_spacing: Float,
}

/// The collection of all trace results for a 3D ray trace.
///
/// We expect to have on the order of 10 different sets of results, each
/// having on the order of 1000 to 100,000 or more rays. The
/// results are stored internally as a Vec and not a HashMap because the O(1)
/// lookup time is not likely to outweigth the overhead of the HashMap in these
/// conditions.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TraceResultsCollection {
    results: Vec<TraceResults>,
}

/// The results of a 3D ray trace.
///
/// This represents the results of a 3D ray trace for a single set of values of
/// 1. path ID,
/// 2. wavelength ID, and
/// 3. field ID.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TraceResults {
    path_id: usize,
    wavelength_id: usize,
    field_id: usize,

    /// A single chief ray through the pupil center.
    chief_ray: RayBundle,

    /// A full-pupil square-grid ray bundle.
    full_pupil: RayBundle,

    /// A tangential ray fan lying in the meridional plane.
    tangential_fan: RayBundle,

    /// A sagittal ray fan perpendicular to the meridional plane.
    sagittal_fan: RayBundle,
}

/// Trace a single ray bundle type across all (field, wavelength) pairs.
///
/// This is the lower-level counterpart to [`ray_trace_3d_view`]. Use it when
/// you need a specific sampling that is not one of the four canonical bundles,
/// for example a tangential fan at a different density for a cross-section
/// display.
///
/// # Arguments
/// * `path_id` - Which path to trace. The overlay is strictly one-path-at-a-
///   time by design; it is never extended to trace every path simultaneously.
/// * `aperture_spec` - The aperture specification, for `path_id`.
/// * `field_specs` - The field specifications, for `path_id`.
/// * `sequential_model` - The sequential model.
/// * `paraxial_view` - A paraxial view. Required for locating the entrance
///   pupil.
/// * `sampling` - Which pupil sampling to use for every (field, wavelength)
///   pair.
///
/// # Returns
/// A `Vec` of `(field_id, wavelength_id, RayBundle)` tuples, one per
/// (field, wavelength) pair, in unspecified order.
pub fn trace_ray_bundle(
    path_id: usize,
    aperture_spec: &ApertureSpec,
    field_specs: &[FieldSpec],
    sequential_model: &SequentialModel,
    paraxial_view: &ParaxialView,
    sampling: PupilSampling,
) -> Result<Vec<(usize, usize, RayBundle)>> {
    validate_field_specs(sequential_model, field_specs)?;

    let n_wavelengths = sequential_model.wavelengths_for_path(path_id).len();
    let pairs: Vec<(usize, usize)> = (0..field_specs.len())
        .flat_map(|f| (0..n_wavelengths).map(move |w| (f, w)))
        .collect();

    pairs
        .into_par_iter()
        .map(
            |(field_id, wavelength_id)| -> Result<(usize, usize, RayBundle)> {
                let sequential_submodel = sequential_model
                    .submodels_for_path(path_id)
                    .get(wavelength_id)
                    .ok_or_else(|| anyhow!("Submodel not found"))?;
                let tangential_vec_id = paraxial_view
                    .tangential_vec_id_for_phi(path_id, field_specs[field_id].tangential_fan_phi());
                let paraxial_subview = paraxial_view
                    .get_for_path(path_id, wavelength_id, tangential_vec_id)
                    .ok_or_else(|| anyhow!("Submodel not found"))?;

                let bundle = ray_trace_submodel(
                    sequential_submodel,
                    sequential_model.surfaces(),
                    sequential_model.placements(),
                    sequential_model.path_surface_indices(path_id),
                    sequential_model.path_beam_splitter_arms(path_id),
                    sequential_model.path_steps(path_id),
                    aperture_spec,
                    &field_specs[field_id],
                    paraxial_subview,
                    sampling,
                )?;

                Ok((field_id, wavelength_id, bundle))
            },
        )
        .collect::<Result<Vec<_>>>()
}

/// Perform a 3D ray trace on a sequential model.
///
/// # Arguments
/// * `aperture_specs_by_path` - One aperture specification per path, indexed by
///   `path_id`. Must have length equal to `sequential_model.path_count()`.
/// * `field_specs_by_path` - One field-spec list per path, indexed by
///   `path_id`. Must have length equal to `sequential_model.path_count()`.
/// * `sequential_model` - The sequential model.
/// * `paraxial_view` - A paraxial view. This is required for finding a system's
///   entrance pupil.
/// * `config` - Sampling configuration for all four ray bundles computed per
///   field/wavelength combination.
pub fn ray_trace_3d_view(
    aperture_specs_by_path: &[ApertureSpec],
    field_specs_by_path: &[Vec<FieldSpec>],
    sequential_model: &SequentialModel,
    paraxial_view: &ParaxialView,
    config: SamplingConfig,
) -> Result<TraceResultsCollection> {
    let path_count = sequential_model.path_count();
    if aperture_specs_by_path.len() != path_count {
        return Err(anyhow!(
            "aperture_specs_by_path has {} entries but the model has {} path(s)",
            aperture_specs_by_path.len(),
            path_count
        ));
    }
    if field_specs_by_path.len() != path_count {
        return Err(anyhow!(
            "field_specs_by_path has {} entries but the model has {} path(s)",
            field_specs_by_path.len(),
            path_count
        ));
    }
    for field_specs in field_specs_by_path {
        validate_field_specs(sequential_model, field_specs)?;
    }

    // Use Axis::U as the canonical submodel for all field/wavelength combinations.
    // For rotationally symmetric systems only Axis::U exists; for non-symmetric
    // systems this is the fallback that is always present.
    let triples: Vec<(usize, usize, usize)> = (0..path_count)
        .flat_map(|p| {
            let n_wavelengths = sequential_model.wavelengths_for_path(p).len();
            let n_fields = field_specs_by_path[p].len();
            (0..n_fields).flat_map(move |f| (0..n_wavelengths).map(move |w| (p, f, w)))
        })
        .collect();

    let results: Vec<TraceResults> = triples
        .into_par_iter()
        .map(
            |(path_id, field_id, wavelength_id)| -> Result<TraceResults> {
                tracing::trace!(
                    "Tracing rays for path_id={}, field_id={}, wavelength_id={}",
                    path_id,
                    field_id,
                    wavelength_id,
                );

                let field_specs = &field_specs_by_path[path_id];
                let aperture_spec = &aperture_specs_by_path[path_id];

                let sequential_submodel = sequential_model
                    .submodels_for_path(path_id)
                    .get(wavelength_id)
                    .ok_or_else(|| anyhow!("Submodel not found"))?;
                let tangential_vec_id = paraxial_view
                    .tangential_vec_id_for_phi(path_id, field_specs[field_id].tangential_fan_phi());
                let paraxial_subview = paraxial_view
                    .get_for_path(path_id, wavelength_id, tangential_vec_id)
                    .ok_or_else(|| anyhow!("Paraxial subview not found"))?;

                let field_spec = &field_specs[field_id];
                let surfaces = sequential_model.surfaces();
                let placements = sequential_model.placements();
                let surface_indices = sequential_model.path_surface_indices(path_id);
                let beam_splitter_arms = sequential_model.path_beam_splitter_arms(path_id);
                let path_steps = sequential_model.path_steps(path_id);

                let chief_ray = ray_trace_submodel(
                    sequential_submodel,
                    surfaces,
                    placements,
                    surface_indices,
                    beam_splitter_arms,
                    path_steps,
                    aperture_spec,
                    field_spec,
                    paraxial_subview,
                    PupilSampling::ChiefRay,
                )?;
                let full_pupil = ray_trace_submodel(
                    sequential_submodel,
                    surfaces,
                    placements,
                    surface_indices,
                    beam_splitter_arms,
                    path_steps,
                    aperture_spec,
                    field_spec,
                    paraxial_subview,
                    PupilSampling::SquareGrid {
                        spacing: config.full_pupil_spacing,
                    },
                )?;
                let tangential_fan = ray_trace_submodel(
                    sequential_submodel,
                    surfaces,
                    placements,
                    surface_indices,
                    beam_splitter_arms,
                    path_steps,
                    aperture_spec,
                    field_spec,
                    paraxial_subview,
                    PupilSampling::TangentialRayFan {
                        n: config.n_fan_rays,
                    },
                )?;
                let sagittal_fan = ray_trace_submodel(
                    sequential_submodel,
                    surfaces,
                    placements,
                    surface_indices,
                    beam_splitter_arms,
                    path_steps,
                    aperture_spec,
                    field_spec,
                    paraxial_subview,
                    PupilSampling::SagittalRayFan {
                        n: config.n_fan_rays,
                    },
                )?;

                trace!(
                    path_id,
                    field_id,
                    wavelength_id,
                    "Finished tracing all bundles for path {}, field {}, wavelength {}",
                    path_id,
                    field_id,
                    wavelength_id
                );

                Ok(TraceResults {
                    path_id,
                    wavelength_id,
                    field_id,
                    chief_ray,
                    full_pupil,
                    tangential_fan,
                    sagittal_fan,
                })
            },
        )
        .collect::<Result<Vec<_>>>()?;

    Ok(TraceResultsCollection::new(results))
}

impl TraceResultsCollection {
    fn new(results: Vec<TraceResults>) -> Self {
        Self { results }
    }

    /// Get results for path 0, a specific field, and wavelength.
    pub fn get(&self, field_id: usize, wavelength_id: usize) -> Option<&TraceResults> {
        self.get_for_path(0, field_id, wavelength_id)
    }

    /// Get results for a specific path, field, and wavelength.
    pub fn get_for_path(
        &self,
        path_id: usize,
        field_id: usize,
        wavelength_id: usize,
    ) -> Option<&TraceResults> {
        self.results.iter().find(|r| {
            r.path_id == path_id && r.field_id == field_id && r.wavelength_id == wavelength_id
        })
    }

    /// Get all results for a given wavelength.
    pub fn get_by_wavelength_id(&self, wavelength: usize) -> Vec<&TraceResults> {
        self.results
            .iter()
            .filter(|r| r.wavelength_id == wavelength)
            .collect()
    }

    /// Get all results for a given field.
    pub fn get_by_field_id(&self, field_id: usize) -> Vec<&TraceResults> {
        self.results
            .iter()
            .filter(|r| r.field_id == field_id)
            .collect()
    }

    /// Appends all results from `other` into this collection.
    pub fn extend(&mut self, other: TraceResultsCollection) {
        self.results.extend(other.results);
    }

    /// Returns an iterator over all results in the collection.
    pub fn iter(&self) -> impl Iterator<Item = &TraceResults> {
        self.results.iter()
    }

    /// Returns whether the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Returns the number of ray bundles traced through the system.
    pub fn len(&self) -> usize {
        self.results.len()
    }
}

impl TraceResults {
    /// Returns the path ID of this result.
    pub fn path_id(&self) -> usize {
        self.path_id
    }

    // Returns the field ID of the ray bundle.
    pub fn field_id(&self) -> usize {
        self.field_id
    }

    // Returns the chief ray bundle.
    pub fn chief_ray(&self) -> &RayBundle {
        &self.chief_ray
    }

    // Returns the full-pupil ray bundle.
    pub fn full_pupil(&self) -> &RayBundle {
        &self.full_pupil
    }

    // Returns the tangential ray fan bundle.
    pub fn tangential_fan(&self) -> &RayBundle {
        &self.tangential_fan
    }

    // Returns the sagittal ray fan bundle.
    pub fn sagittal_fan(&self) -> &RayBundle {
        &self.sagittal_fan
    }

    // Returns the wavelength ID of the ray bundle.
    pub fn wavelength_id(&self) -> usize {
        self.wavelength_id
    }

    /// Returns true if the chief ray reached the image surface without
    /// terminating early (aperture miss, convergence failure, etc.).
    pub fn chief_ray_reached_image(&self) -> bool {
        self.chief_ray.terminated().first().copied().unwrap_or(0) == 0
    }
}

#[allow(clippy::too_many_arguments)]
fn ray_trace_submodel(
    sequential_submodel: &impl SequentialSubModel,
    surfaces: &[Box<dyn Surface>],
    placements: &[SurfacePlacement],
    surface_indices: &[usize],
    beam_splitter_arms: &[Option<BeamSplitterPathKind>],
    path_steps: &[CursorPlacement],
    aperture_spec: &ApertureSpec,
    field_spec: &FieldSpec,
    paraxial_subview: &ParaxialSubView,
    pupil_sampling: PupilSampling,
) -> Result<RayBundle> {
    let rays = rays(
        placements,
        surface_indices,
        aperture_spec,
        paraxial_subview,
        field_spec,
        pupil_sampling,
    )?;

    let mut sequential_sub_model_iter = sequential_submodel.try_iter(
        surfaces,
        placements,
        surface_indices,
        beam_splitter_arms,
        path_steps,
    )?;
    Ok(trace(&mut sequential_sub_model_iter, rays))
}

/// Validate the field specifications.
///
/// This function checks that the field specifications are valid.
///
/// For example, you cannot have an infinite object distance and a point source
/// field.
fn validate_field_specs(
    sequential_model: &SequentialModel,
    field_specs: &[FieldSpec],
) -> Result<()> {
    for field_spec in field_specs {
        match field_spec {
            FieldSpec::Angle { chi, .. } => {
                if !chi.is_finite() {
                    return Err(anyhow!("Field chi must be finite"));
                }
            }
            FieldSpec::PointSource { .. } => {
                for submodel in sequential_model.submodels() {
                    if submodel.is_obj_at_inf() {
                        return Err(anyhow!(
                            "Cannot have a point source field with an infinite object distance"
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::Float;
    use crate::core::math::vec3::Vec3;
    use crate::examples::convexplano_lens::sequential_model;
    use crate::n;

    use super::*;

    struct Setup {
        sequential_model: SequentialModel,
        aperture_spec: ApertureSpec,
        field_specs: Vec<FieldSpec>,
        paraxial_view: ParaxialView,
    }

    fn setup() -> Setup {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let sequential_model = sequential_model(air, nbk7, &wavelengths);

        let aperture_spec = ApertureSpec::EntrancePupil {
            semi_diameter: 12.5,
        };
        let field_specs = vec![
            FieldSpec::Angle {
                chi: 0.0,
                phi: 90.0,
            },
            FieldSpec::Angle {
                chi: 5.0,
                phi: 90.0,
            },
        ];

        let paraxial_view =
            ParaxialView::new(&sequential_model, std::slice::from_ref(&field_specs), false)
                .unwrap();

        Setup {
            sequential_model,
            aperture_spec,
            field_specs,
            paraxial_view,
        }
    }

    #[test]
    fn test_trace_ray_bundle() {
        let s = setup();

        let bundles = trace_ray_bundle(
            0,
            &s.aperture_spec,
            &s.field_specs,
            &s.sequential_model,
            &s.paraxial_view,
            PupilSampling::TangentialRayFan { n: 7 },
        )
        .unwrap();

        // One tuple per (field, wavelength) pair: 2 fields × 1 wavelength = 2.
        assert_eq!(bundles.len(), 2);
        for (field_id, wavelength_id, bundle) in &bundles {
            assert!(*field_id < s.field_specs.len());
            assert_eq!(*wavelength_id, 0);
            assert_eq!(
                bundle.rays().len() / bundle.num_surfaces(),
                7,
                "field {field_id} bundle should have 7 rays"
            );
        }
    }

    #[test]
    fn test_ray_trace_3d_view() {
        let s = setup();

        let config = SamplingConfig {
            n_fan_rays: 3,
            full_pupil_spacing: 0.1,
        };
        let results = ray_trace_3d_view(
            &[s.aperture_spec],
            std::slice::from_ref(&s.field_specs),
            &s.sequential_model,
            &s.paraxial_view,
            config,
        )
        .unwrap();

        assert_eq!(results.len(), 2); // 2 fields x 1 wavelength
    }

    #[test]
    fn test_ray_trace_3d_view_all_four_bundles() {
        let s = setup();
        let config = SamplingConfig {
            n_fan_rays: 5,
            full_pupil_spacing: 0.1,
        };

        let results = ray_trace_3d_view(
            &[s.aperture_spec],
            std::slice::from_ref(&s.field_specs),
            &s.sequential_model,
            &s.paraxial_view,
            config,
        )
        .unwrap();

        assert_eq!(results.len(), 2); // 2 fields x 1 wavelength
        let r = results.iter().next().unwrap();
        // Each TraceResults must expose all four bundles.
        assert_eq!(r.chief_ray().rays().len() / r.chief_ray().num_surfaces(), 1);
        assert!(r.full_pupil().rays().len() / r.full_pupil().num_surfaces() > 1);
        assert_eq!(
            r.tangential_fan().rays().len() / r.tangential_fan().num_surfaces(),
            5
        );
        assert_eq!(
            r.sagittal_fan().rays().len() / r.sagittal_fan().num_surfaces(),
            5
        );
    }

    #[test]
    fn test_validate_field_specs() {
        let s = setup();

        let field_specs = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];

        let result = validate_field_specs(&s.sequential_model, &field_specs);
        assert!(
            result.is_ok(),
            "Expected Ok result because FieldSpec::Angle is compatible with objects at infinity. Result: {:?}",
            result
        );

        let field_specs = vec![FieldSpec::PointSource { x: 0.0, y: 0.0 }];

        let result = validate_field_specs(&s.sequential_model, &field_specs);
        assert!(
            result.is_err(),
            "Expected Err result because FieldSpec::PointSource is incompatible with objects at infinity. Result: {:?}",
            result
        );
    }

    #[test]
    fn ray_trace_3d_view_result_count() {
        // Verify that the result count is n_fields * n_wavelengths with no axis
        // duplication.
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 3] = [0.4861, 0.5876, 0.6563];
        let sequential_model = sequential_model(air, nbk7, &wavelengths);
        let aperture_spec = ApertureSpec::EntrancePupil {
            semi_diameter: 12.5,
        };
        let field_specs = vec![
            FieldSpec::Angle {
                chi: 0.0,
                phi: 90.0,
            },
            FieldSpec::Angle {
                chi: 5.0,
                phi: 90.0,
            },
        ];
        let paraxial_view =
            ParaxialView::new(&sequential_model, std::slice::from_ref(&field_specs), false)
                .unwrap();
        let config = SamplingConfig {
            n_fan_rays: 3,
            full_pupil_spacing: 0.1,
        };

        let results = ray_trace_3d_view(
            &[aperture_spec],
            std::slice::from_ref(&field_specs),
            &sequential_model,
            &paraxial_view,
            config,
        )
        .unwrap();

        assert_eq!(results.len(), 6); // 2 fields * 3 wavelengths
    }

    #[test]
    fn ray_trace_two_path_model_produces_results_for_both_paths() {
        use std::rc::Rc;

        use crate::examples::beam_splitter::two_path_model;
        use crate::specs::gaps::ConstantRefractiveIndex;

        let n_air = Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let model = two_path_model(n_air, &[0.5876e-3], 10.0, 10.0);
        let field_specs = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 5.0 };
        let paraxial_view =
            ParaxialView::new(&model, &[field_specs.clone(), field_specs.clone()], false).unwrap();
        let config = SamplingConfig {
            n_fan_rays: 3,
            full_pupil_spacing: 0.5,
        };

        let results = ray_trace_3d_view(
            &[aperture_spec, aperture_spec],
            &[field_specs.clone(), field_specs],
            &model,
            &paraxial_view,
            config,
        )
        .unwrap();

        assert!(
            results.get_for_path(0, 0, 0).is_some(),
            "path 0 results missing"
        );
        assert!(
            results.get_for_path(1, 0, 0).is_some(),
            "path 1 results missing"
        );
    }

    #[test]
    fn ray_trace_3d_view_handles_differing_wavelength_counts_per_path() {
        use std::rc::Rc;

        use crate::RefractiveIndexSpec;
        use crate::core::math::linalg::rotations::{EulerAngles, Rotation3D};
        use crate::core::sequential_model::builder::SequentialModelBuilder;
        use crate::specs::gaps::{ConstantRefractiveIndex, GapSpec};
        use crate::specs::paths::{PathSpec, PathSurfaceRef};
        use crate::specs::surfaces::{BeamSplitterPathKind, SurfaceSpec};

        let n_air: Rc<dyn RefractiveIndexSpec> = Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let bs_rotation =
            Rotation3D::IntrinsicPassiveRUF(EulerAngles((-45_f64).to_radians(), 0.0, 0.0));
        let img = || SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_inf = || GapSpec {
            thickness: f64::INFINITY,
            refractive_index: n_air.clone(),
        };

        let path_t = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::New(SurfaceSpec::Object),
                PathSurfaceRef::New(SurfaceSpec::BeamSplitter {
                    semi_diameter: 10.0,
                    rotation: bs_rotation,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
                PathSurfaceRef::New(img()),
            ],
            gaps: vec![
                gap_inf(),
                GapSpec {
                    thickness: 10.0,
                    refractive_index: n_air.clone(),
                },
            ],
            beam_splitter_arms: vec![BeamSplitterPathKind::Transmitting],
            stop_surface: None,
            wavelengths: vec![0.5876], // 1 wavelength
        };
        let path_r = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::Shared(0),
                PathSurfaceRef::Shared(1),
                PathSurfaceRef::New(img()),
            ],
            gaps: vec![
                gap_inf(),
                GapSpec {
                    thickness: 10.0,
                    refractive_index: n_air.clone(),
                },
            ],
            beam_splitter_arms: vec![BeamSplitterPathKind::Reflecting],
            stop_surface: None,
            wavelengths: vec![0.4861, 0.5876, 0.6563], // 3 wavelengths
        };

        let model = SequentialModelBuilder::new()
            .paths(vec![path_t, path_r])
            .build()
            .expect("beam splitter model builds")
            .model;

        let field_specs = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 5.0 };
        let paraxial_view =
            ParaxialView::new(&model, &[field_specs.clone(), field_specs.clone()], false).unwrap();
        let config = SamplingConfig {
            n_fan_rays: 3,
            full_pupil_spacing: 0.5,
        };

        let results = ray_trace_3d_view(
            &[aperture_spec, aperture_spec],
            &[field_specs.clone(), field_specs.clone()],
            &model,
            &paraxial_view,
            config,
        )
        .expect("ray trace should succeed with differing per-path wavelength counts");

        assert_eq!(
            results.iter().filter(|r| r.path_id() == 0).count(),
            field_specs.len()
        );
        assert_eq!(
            results.iter().filter(|r| r.path_id() == 1).count(),
            3 * field_specs.len()
        );
    }

    #[test]
    fn chief_ray_reached_image_on_axis() {
        // On-axis chief ray should always reach the image surface in a standard
        // refractive system.
        let s = setup();
        let config = SamplingConfig {
            n_fan_rays: 3,
            full_pupil_spacing: 0.1,
        };
        let results = ray_trace_3d_view(
            &[s.aperture_spec],
            std::slice::from_ref(&s.field_specs),
            &s.sequential_model,
            &s.paraxial_view,
            config,
        )
        .unwrap();

        let on_axis = results
            .get(0, 0)
            .expect("field 0, wavelength 0 should exist");
        assert!(
            on_axis.chief_ray_reached_image(),
            "On-axis chief ray should reach the image surface"
        );
    }

    /// AT-2: a two-path model called with mismatched-length
    /// `aperture_specs_by_path`/`field_specs_by_path` must return an error,
    /// independently for each list.
    #[test]
    fn at2_ray_trace_3d_view_rejects_mismatched_per_path_list_lengths() {
        use std::rc::Rc;

        use crate::examples::beam_splitter::two_path_model;
        use crate::specs::gaps::ConstantRefractiveIndex;

        let n_air = Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let model = two_path_model(n_air, &[0.5876e-3], 10.0, 10.0);
        let field = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let aperture = ApertureSpec::EntrancePupil { semi_diameter: 5.0 };
        let paraxial_view =
            ParaxialView::new(&model, &[field.clone(), field.clone()], false).unwrap();
        let config = SamplingConfig {
            n_fan_rays: 3,
            full_pupil_spacing: 0.5,
        };

        // Wrong-length aperture_specs_by_path (1 entry for a 2-path model).
        assert!(
            ray_trace_3d_view(
                &[aperture],
                &[field.clone(), field.clone()],
                &model,
                &paraxial_view,
                config,
            )
            .is_err()
        );

        // Wrong-length field_specs_by_path (1 entry for a 2-path model).
        assert!(
            ray_trace_3d_view(
                &[aperture, aperture],
                &[field],
                &model,
                &paraxial_view,
                config,
            )
            .is_err()
        );
    }

    /// AT-4: each path must be traced against its own `ApertureSpec`, not a
    /// shared value. Two paths with different entrance-pupil semi-diameters
    /// must produce `TraceResults` whose full-pupil ray bundle extents
    /// differ correspondingly.
    #[test]
    fn at4_ray_trace_3d_view_traces_each_path_against_its_own_aperture() {
        use std::rc::Rc;

        use crate::examples::beam_splitter::two_path_model;
        use crate::specs::gaps::ConstantRefractiveIndex;

        let n_air = Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let model = two_path_model(n_air, &[0.5876e-3], 10.0, 10.0);
        let field = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let small_aperture = ApertureSpec::EntrancePupil { semi_diameter: 2.0 };
        let large_aperture = ApertureSpec::EntrancePupil {
            semi_diameter: 10.0,
        };
        let paraxial_view =
            ParaxialView::new(&model, &[field.clone(), field.clone()], false).unwrap();
        let config = SamplingConfig {
            n_fan_rays: 3,
            full_pupil_spacing: 0.5,
        };

        let results = ray_trace_3d_view(
            &[small_aperture, large_aperture],
            &[field.clone(), field],
            &model,
            &paraxial_view,
            config,
        )
        .unwrap();

        let extent = |bundle: &RayBundle| -> Float {
            bundle
                .rays()
                .iter()
                .map(|r| r.y().abs())
                .fold(0.0, Float::max)
        };

        let path0 = results.get_for_path(0, 0, 0).expect("path 0 results");
        let path1 = results.get_for_path(1, 0, 0).expect("path 1 results");

        let extent0 = extent(path0.full_pupil());
        let extent1 = extent(path1.full_pupil());

        assert!(
            extent1 > extent0,
            "path 1 (larger aperture) should have greater ray extent than path 0: {extent0} vs {extent1}"
        );
    }
}
