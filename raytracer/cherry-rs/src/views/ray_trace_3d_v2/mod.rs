/// Performs a 3D ray trace on the system.
mod rays;
mod trace;

use std::collections::HashMap;

use anyhow::Result;

use crate::{
    core::{
        sequential_model::{SequentialModel, SequentialSubModel, SubModelID, Surface},
        Float, PI,
    },
    specs::{
        aperture::ApertureSpec,
        fields::{FieldSpec, PupilSampling},
    },
    Axis, Pupil,
};

use trace::trace;

pub use rays::Ray as RayV2;
pub use trace::TraceSubResults;

use super::paraxial::{ParaxialSubView, ParaxialView};

/// The default capacity for the results collection.
///
/// Increase this to avoid reallocations if you expect to have more results. The
/// tradeoff is larger memory usage.
///
/// Its current value was derived from: 3 wavelengths x 3 fields x 2 axes = 18.
const RESULTS_CAPACITY: usize = 20;

/// The collection of all trace results for a 3D ray trace.
///
/// We expect to have on the order of 10 different sets of results, each
/// having on the order of 1000 to 100,000 or more rays. The
/// results are stored internally as a Vec and not a HashMap because the O(1)
/// lookup time is not likely to outweigth the overhead of the HashMap in these
/// conditions.
pub struct TraceResultsCollection {
    results: Vec<TraceResultsV2>,
}

/// The results of a 3D ray trace.
///
/// This represents the results of a 3D ray trace for a single set of values of
/// 1. wavelength ID,
/// 2. field ID, and
/// 3. axis.
pub struct TraceResultsV2 {
    wavelength_id: usize,
    field_id: usize,
    axis: Axis,
    data: TraceSubResults,
}

/// Perform a 3D ray trace on a sequential model.
///
/// # Arguments
/// * `aperture_spec` - The aperture specification.
/// * `field_specs` - The field specifications.
/// * `sequential_model` - The sequential model.
/// * `paraxial_view` - A paraxial view. This is required for finding a system's
///   entrance pupil.
/// * `pupil_sampling` - The pupil sampling method. This will override the
///   sampling method specified in the field specs for every field if provided.
pub fn ray_trace_3d_view_v2(
    aperture_spec: &ApertureSpec,
    field_specs: &[FieldSpec],
    sequential_model: &SequentialModel,
    paraxial_view: &ParaxialView,
    pupil_sampling: Option<PupilSampling>,
) -> Result<HashMap<SubModelID, TraceSubResults>> {
    let results = sequential_model
        .submodels()
        .iter()
        .map(|(id, submodel)| {
            let surfaces = sequential_model.surfaces();
            let paraxial_sub_view = paraxial_view.subviews().get(id).unwrap();
            Ok((
                *id,
                ray_trace_sub_model(
                    field_specs,
                    submodel,
                    surfaces,
                    aperture_spec,
                    paraxial_sub_view,
                    pupil_sampling,
                )?,
            ))
        })
        .collect();

    results
}

impl TraceResultsCollection {
    fn new() -> Self {
        Self {
            results: Vec::with_capacity(RESULTS_CAPACITY),
        }
    }

    /// Get all results for a given axis.
    pub fn get_by_axis(&self, axis: Axis) -> Vec<&TraceResultsV2> {
        self.results.iter().filter(|r| r.axis == axis).collect()
    }

    /// Get all results for a given wavelength.
    pub fn get_by_wavelength(&self, wavelength: usize) -> Vec<&TraceResultsV2> {
        self.results
            .iter()
            .filter(|r| r.wavelength_id == wavelength)
            .collect()
    }

    /// Get all results for a given field.
    pub fn get_by_field_id(&self, field_id: usize) -> Vec<&TraceResultsV2> {
        self.results
            .iter()
            .filter(|r| r.field_id == field_id)
            .collect()
    }

    /// Get results for a specific field, wavelength, and axis.
    pub fn get(
        &self,
        field_id: usize,
        wavelength_id: usize,
        axis: Axis,
    ) -> Option<&TraceResultsV2> {
        self.results
            .iter()
            .find(|r| r.field_id == field_id && r.wavelength_id == wavelength_id && r.axis == axis)
    }
}

fn ray_trace_sub_model(
    field_specs: &[FieldSpec],
    sequential_sub_model: &impl SequentialSubModel,
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_sub_view: &ParaxialSubView,
    pupil_sampling: Option<PupilSampling>,
) -> Result<TraceSubResults> {
    let rays = rays(
        surfaces,
        aperture_spec,
        paraxial_sub_view,
        field_specs,
        pupil_sampling,
    )?;

    let mut sequential_sub_model_iter = sequential_sub_model.try_iter(surfaces)?;
    Ok(trace(&mut sequential_sub_model_iter, rays))
}

/// Returns the rays to trace through the system as defined by the fields.
///
/// # Arguments
///
/// * `sampling` - The pupil sampling method. This will override the sampling
///   method specified in the field specs for every field if provided.
fn rays(
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_sub_view: &ParaxialSubView,
    field_specs: &[FieldSpec],
    sampling: Option<PupilSampling>,
) -> Result<Vec<RayV2>> {
    let mut rays = Vec::new();

    for (field_id, field) in field_specs.iter().enumerate() {
        match field {
            FieldSpec::Angle {
                angle,
                pupil_sampling,
            } => {
                let angle = angle.to_radians();

                let pupil_sampling = match sampling {
                    Some(sampling) => sampling,
                    None => *pupil_sampling,
                };

                let rays_field = match pupil_sampling {
                    PupilSampling::SquareGrid { spacing } => pupil_ray_sq_grid(
                        surfaces,
                        aperture_spec,
                        paraxial_sub_view,
                        spacing,
                        angle,
                        field_id,
                    )?,
                    PupilSampling::ChiefAndMarginalRays => {
                        // 3 rays -> two diametrically-opposed marginal rays at the pupil edge
                        // and a chief ray in the center
                        pupil_ray_fan(
                            surfaces,
                            aperture_spec,
                            paraxial_sub_view,
                            3,
                            PI / 2.0,
                            angle,
                            field_id,
                        )?
                    }
                };

                rays.extend(rays_field);
            }
            _ => unimplemented!(),
        }
    }

    Ok(rays)
}

/// Create a linear ray fan that passes through the entrance pupil.
///
/// # Arguments
///
/// * `num_rays` - The number of rays in the fan.
/// * `theta` - The polar angle of the ray fan in the x-y plane.
/// * `phi` - The angle of the ray w.r.t. the z-axis.
/// * `field_id` - The ID of the field.
#[allow(clippy::too_many_arguments)]
fn pupil_ray_fan(
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_sub_view: &ParaxialSubView,
    num_rays: usize,
    theta: Float,
    phi: Float,
    field_id: usize,
) -> Result<Vec<RayV2>> {
    let ep = entrance_pupil(aperture_spec, paraxial_sub_view)?;
    let obj_z = surfaces[0].pos().z();
    let sur_z = surfaces[1].pos().z();
    let enp_z = ep.pos().z();

    let launch_point_z = axial_launch_point(obj_z, sur_z, enp_z);

    // Determine the radial distance from the axis at the launch point for the
    // center of the ray fan.
    let dz = enp_z - launch_point_z;
    let dy = -dz * phi.tan();

    let rays = RayV2::fan(
        num_rays,
        ep.semi_diameter,
        theta,
        launch_point_z,
        phi,
        0.0,
        dy,
        field_id,
    );

    Ok(rays)
}

/// Create a square grid of rays that passes through the entrance pupil.
///
/// # Arguments
///
/// * `spacing` - The spacing between rays in the grid in normalized pupil
///   distances, i.e. [0, 1]. A spacing of 1.0 means that one ray will lie at
///   the pupil center (the chief ray) and the others will lie at the pupil edge
///   (marginal rays).
/// * `phi` - The angle of the ray w.r.t. the z-axis in radians.
/// * `field_id` - The field ID.
fn pupil_ray_sq_grid(
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_sub_view: &ParaxialSubView,
    spacing: Float,
    phi: Float,
    field_id: usize,
) -> Result<Vec<RayV2>> {
    let ep = entrance_pupil(aperture_spec, paraxial_sub_view)?;
    let obj_z = surfaces[0].pos().z();
    let sur_z = surfaces[1].pos().z();
    let enp_z = ep.pos().z();

    let launch_point_z = axial_launch_point(obj_z, sur_z, enp_z);

    let enp_diam = ep.semi_diameter;
    let abs_spacing = enp_diam / 2.0 * spacing;

    // Determine the radial distance from the axis at the launch point for the
    // center of the ray fan.
    let dz = enp_z - launch_point_z;
    let dy = -dz * phi.tan();

    let rays = RayV2::sq_grid_in_circ(
        enp_diam / 2.0,
        abs_spacing,
        launch_point_z,
        phi,
        0.0,
        dy,
        field_id,
    );

    Ok(rays)
}

/// Determines the entrance pupil of the subview.
fn entrance_pupil(
    aperture_spec: &ApertureSpec,
    paraxial_sub_view: &ParaxialSubView,
) -> Result<Pupil> {
    let semi_diameter = match aperture_spec {
        ApertureSpec::EntrancePupil { semi_diameter } => *semi_diameter,
    };

    let entrance_pupil = paraxial_sub_view.entrance_pupil();
    let z = entrance_pupil.pos().z();

    Ok(Pupil {
        location: z,
        semi_diameter,
    })
}

/// Determine the axial launch point for the rays.
///
/// If the object plane is at infinity, and if the first surface lies before
/// the entrance pupil, then launch the rays from one unit to the left
/// of the first surface. If the object plane is at infinity, and if it
/// comes after the entrance pupil, then launch the rays from
/// one unit in front of the entrance pupil. Otherwise, launch the rays from
/// the object plane.
fn axial_launch_point(obj_z: Float, sur_z: Float, enp_z: Float) -> Float {
    if obj_z == Float::NEG_INFINITY && sur_z <= enp_z {
        sur_z - 1.0
    } else if obj_z == Float::NEG_INFINITY && sur_z > enp_z {
        enp_z - 1.0
    } else {
        obj_z
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Float;
    use crate::examples::convexplano_lens::sequential_model;
    use crate::n;

    use super::*;

    #[test]
    fn test_ray_trace_3d_view() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let sequential_model = sequential_model(air, nbk7, &wavelengths);

        let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 5.0 };
        let field_specs = vec![
            FieldSpec::Angle {
                angle: 0.0,
                pupil_sampling: PupilSampling::ChiefAndMarginalRays,
            },
            FieldSpec::Angle {
                angle: 5.0,
                pupil_sampling: PupilSampling::ChiefAndMarginalRays,
            },
        ];

        let paraxial_view = ParaxialView::new(&sequential_model, &field_specs, false).unwrap();

        let results = ray_trace_3d_view_v2(
            &aperture_spec,
            &field_specs,
            &sequential_model,
            &paraxial_view,
            None,
        )
        .unwrap();

        assert_eq!(results.len(), 1);
    }
}
