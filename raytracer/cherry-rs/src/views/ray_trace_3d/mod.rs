/// Performs a 3D ray trace on the system.
mod rays;
mod trace;

use anyhow::{Result, anyhow};
use serde::Serialize;

use crate::{
    Axis, Pupil,
    core::{
        Float, PI,
        math::vec3::Vec3,
        sequential_model::{SequentialModel, SequentialSubModel, SubModelID, Surface},
    },
    specs::{
        aperture::ApertureSpec,
        fields::{FieldSpec, PupilSampling},
    },
};

use trace::trace;

pub use rays::Ray;
pub use trace::RayBundle;

use super::paraxial::{ParaxialSubView, ParaxialView};

/// The default capacity for the results collection.
///
/// Increase this to avoid reallocations if you expect to have more results. The
/// tradeoff is larger memory usage.
///
/// Its current value was derived from: 3 wavelengths x 3 fields x 2 axes = 18.
const RESULTS_CAPACITY: usize = 20;

/// The distance to launch the rays before the first surface when the object is
/// at infinity.
const LAUNCH_POINT_BEFORE_SURFACE: Float = 10.0;

/// The distance to launch the rays before the entrance pupil when the object is
/// at infinity.
const LAUNCH_POINT_BEFORE_PUPIL: Float = 10.0;

/// The collection of all trace results for a 3D ray trace.
///
/// We expect to have on the order of 10 different sets of results, each
/// having on the order of 1000 to 100,000 or more rays. The
/// results are stored internally as a Vec and not a HashMap because the O(1)
/// lookup time is not likely to outweigth the overhead of the HashMap in these
/// conditions.
#[derive(Debug, Serialize)]
pub struct TraceResultsCollection {
    results: Vec<TraceResults>,
}

/// The results of a 3D ray trace.
///
/// This represents the results of a 3D ray trace for a single set of values of
/// 1. wavelength ID,
/// 2. field ID, and
/// 3. axis.
#[derive(Debug, Serialize)]
pub struct TraceResults {
    wavelength_id: usize,
    field_id: usize,
    axis: Axis,

    /// A num_surfaces x num_rays matrix of rays traced through the system.
    ray_bundle: RayBundle,

    /// A num_surfaces x 1 chief ray trace through the system.
    chief_ray: RayBundle,
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
pub fn ray_trace_3d_view(
    aperture_spec: &ApertureSpec,
    field_specs: &[FieldSpec],
    sequential_model: &SequentialModel,
    paraxial_view: &ParaxialView,
    pupil_sampling: Option<PupilSampling>,
) -> Result<TraceResultsCollection> {
    validate_field_specs(sequential_model, field_specs)?;

    let combinations = all_combinations(
        field_specs,
        sequential_model.wavelengths(),
        &sequential_model.axes(),
    );

    let mut results: Vec<TraceResults> = Vec::with_capacity(RESULTS_CAPACITY);

    for (field_id, wavelength_id, axis) in combinations {
        let submodel_id = SubModelID(wavelength_id, axis);
        let sequential_submodel = sequential_model
            .submodels()
            .get(&submodel_id)
            .ok_or_else(|| anyhow!("Submodel not found"))?;
        let paraxial_subview = paraxial_view
            .subviews()
            .get(&submodel_id)
            .ok_or_else(|| anyhow!("Submodel not found"))?;
        let ray_bundle = ray_trace_submodel(
            sequential_submodel,
            sequential_model.surfaces(),
            aperture_spec,
            &field_specs[field_id],
            paraxial_subview,
            pupil_sampling,
        )?;
        let chief_ray = ray_trace_submodel(
            sequential_submodel,
            sequential_model.surfaces(),
            aperture_spec,
            &field_specs[field_id],
            paraxial_subview,
            Some(PupilSampling::ChiefRay),
        )?;
        results.push(TraceResults {
            wavelength_id,
            field_id,
            axis,
            ray_bundle,
            chief_ray,
        });
    }

    Ok(TraceResultsCollection::new(results))
}

impl TraceResultsCollection {
    fn new(results: Vec<TraceResults>) -> Self {
        Self { results }
    }

    /// Get results for a specific field, wavelength, and axis.
    pub fn get(&self, field_id: usize, wavelength_id: usize, axis: Axis) -> Option<&TraceResults> {
        self.results
            .iter()
            .find(|r| r.field_id == field_id && r.wavelength_id == wavelength_id && r.axis == axis)
    }

    /// Get all results for a given axis.
    pub fn get_by_axis(&self, axis: Axis) -> Vec<&TraceResults> {
        self.results.iter().filter(|r| r.axis == axis).collect()
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
    // Returns the axis of the ray bundle.
    pub fn axis(&self) -> Axis {
        self.axis
    }

    // Returns the field ID of the ray bundle.
    pub fn field_id(&self) -> usize {
        self.field_id
    }

    // Returns the ray bundle.
    pub fn ray_bundle(&self) -> &RayBundle {
        &self.ray_bundle
    }

    // Returns the wavelength ID of the ray bundle.
    pub fn wavelength_id(&self) -> usize {
        self.wavelength_id
    }
}

fn ray_trace_submodel(
    sequential_submodel: &impl SequentialSubModel,
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    field_spec: &FieldSpec,
    paraxial_subview: &ParaxialSubView,
    pupil_sampling: Option<PupilSampling>,
) -> Result<RayBundle> {
    let rays = rays(
        surfaces,
        aperture_spec,
        paraxial_subview,
        field_spec,
        pupil_sampling,
    )?;

    let mut sequential_sub_model_iter = sequential_submodel.try_iter(surfaces)?;
    Ok(trace(&mut sequential_sub_model_iter, rays))
}

/// Returns the initial rays in a ray bundle to trace through the system.
///
/// # Arguments
///
/// * `surfaces` - The surfaces of the system.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview. This is used to obtain the
///   pupil.
/// * `field_spec` - The field specification.
/// * `sampling` - The pupil sampling method. This will override the sampling
///   method specified in the field specs for every field if provided.
fn rays(
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    field_spec: &FieldSpec,
    sampling: Option<PupilSampling>,
) -> Result<Vec<Ray>> {
    let rays: Vec<Ray> = match field_spec {
        FieldSpec::Angle {
            angle,
            pupil_sampling,
        } => {
            let angle = angle.to_radians();

            let pupil_sampling = match sampling {
                Some(sampling) => sampling,
                None => *pupil_sampling,
            };

            match pupil_sampling {
                PupilSampling::ChiefRay => chief_ray_from_angle(
                    surfaces,
                    aperture_spec,
                    paraxial_subview,
                    PI / 2.0,
                    angle,
                )?,
                PupilSampling::SquareGrid { spacing } => parallel_ray_bundle_on_sq_grid(
                    surfaces,
                    aperture_spec,
                    paraxial_subview,
                    spacing,
                    angle,
                )?,
                PupilSampling::TangentialRayFan => {
                    // 3 rays -> two diametrically-opposed rays at the pupil edge
                    // and a chief ray in the center
                    parallel_ray_fan(
                        surfaces,
                        aperture_spec,
                        paraxial_subview,
                        3,
                        PI / 2.0, // Currently, the ray fan is always in the y-z plane
                        angle,
                    )?
                }
            }
        }

        FieldSpec::PointSource {
            x,
            y,
            pupil_sampling,
        } => {
            let obj_z = surfaces
                .first()
                .expect("There should always be at least two surfaces")
                .pos()
                .z();
            if obj_z.is_infinite() {
                return Err(anyhow!(
                    "Cannot have a point source field with an infinite object distance"
                ));
            }

            let pupil_sampling = match sampling {
                Some(sampling) => sampling,
                None => *pupil_sampling,
            };

            let origin = Vec3::new(*x, *y, obj_z);
            match pupil_sampling {
                PupilSampling::ChiefRay => {
                    chief_ray_from_pos(aperture_spec, paraxial_subview, &origin)?
                }
                PupilSampling::SquareGrid { spacing } => point_source_ray_bundle_on_sq_grid(
                    aperture_spec,
                    paraxial_subview,
                    spacing,
                    &origin,
                )?,
                PupilSampling::TangentialRayFan => {
                    // If on-axis, put the rays in the y-z plane by default.
                    let theta = if *y == 0.0 && *x == 0.0 {
                        PI / 2.0
                    } else {
                        y.atan2(*x)
                    };
                    point_source_ray_fan(aperture_spec, paraxial_subview, 3, theta, &origin)?
                }
            }
        }
    };

    Ok(rays)
}

/// Creates the chief ray for a given field angle.
///
/// # Arguments
///
/// * `surfaces` - The surfaces of the system.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `theta` - The polar angle of the ray in the x-y plane.
/// * `phi` - The angle of the ray w.r.t. the z-axis.
fn chief_ray_from_angle(
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    theta: Float,
    phi: Float,
) -> Result<Vec<Ray>> {
    let origin = parallel_ray_bundle_origin(surfaces, aperture_spec, paraxial_subview, theta, phi)?;
    let dir = Vec3::new(theta.cos() * phi.sin(), theta.sin() * phi.sin(), phi.cos()).normalize();

    Ok(vec![Ray::new(origin, dir)])
}

/// Creates the chief ray for a given field position.
///
/// # Arguments
///
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `origin` - The origin of the rays, i.e. the field point in the object.
fn chief_ray_from_pos(
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    origin: &Vec3,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview)?;
    let dir = (enp.pos() - *origin).normalize();

    Ok(vec![Ray::new(*origin, dir)])
}

/// Creates a fan of parallel rays that passes through the entrance pupil.
///
/// This is used to model field angles.
///
/// # Arguments
///
/// * `surfaces` - The surfaces of the system.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `num_rays` - The number of rays in the fan.
/// * `theta` - The polar angle of the ray fan in the x-y plane.
/// * `phi` - The angle of the ray w.r.t. the z-axis.
#[allow(clippy::too_many_arguments)]
fn parallel_ray_fan(
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    num_rays: usize,
    theta: Float,
    phi: Float,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview)?;
    let origin = parallel_ray_bundle_origin(surfaces, aperture_spec, paraxial_subview, theta, phi)?;

    let rays = Ray::parallel_ray_fan(
        num_rays,
        enp.semi_diameter,
        origin.z(),
        theta,
        phi,
        origin.x(),
        origin.y(),
    );

    Ok(rays)
}

/// Creates a bundle of parallel rays on a square grid in the entrance pupil.
///
/// This is used to model field angles.
///
/// # Arguments
///
/// * `surfaces` - The surfaces of the system.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `spacing` - The spacing between rays in the grid in normalized pupil
///   distances, i.e. [0, 1]. A spacing of 1.0 means that one ray will lie at
///   the pupil center (the chief ray) and the others will lie at the pupil edge
///   (marginal rays).
/// * `phi` - The angle of the ray bundle w.r.t. the z-axis in radians.
fn parallel_ray_bundle_on_sq_grid(
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    spacing: Float,
    phi: Float,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview)?;
    let abs_spacing = enp.semi_diameter * spacing;
    let origin =
        parallel_ray_bundle_origin(surfaces, aperture_spec, paraxial_subview, PI / 2.0, phi)?;

    let rays = Ray::parallel_ray_bundle_on_sq_grid(
        enp.semi_diameter,
        abs_spacing,
        origin.z(),
        phi,
        origin.x(),
        origin.y(),
    );

    Ok(rays)
}

/// Creates a fan of rays from a single point source that passes through the
/// center of the pupil.
///
/// This is used to model point source fields.
///
/// # Arguments
///
/// * `aperture_spec` : The aperture specification.
/// * `paraxial_subview` : The paraxial subview.
/// * `theta` - The polar angle of the ray fan in the x-y plane.
/// * `origin` : The origin of the rays, i.e. the field point in the object
///   plane.
fn point_source_ray_fan(
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    num_rays: usize,
    theta: Float,
    origin: &Vec3,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview)?;
    let enp_radius = enp.semi_diameter;
    let enp_z = enp.pos().z();

    let pupil_ray_positions = Vec3::fan(num_rays, enp_radius, enp_z, theta, 0.0, 0.0);

    let directions = pupil_ray_positions
        .iter()
        .map(|pos| (*pos - *origin).normalize())
        .collect::<Vec<Vec3>>();

    let rays = directions
        .iter()
        .map(|dir| Ray::new(*origin, *dir))
        .collect::<Vec<Ray>>();

    Ok(rays)
}

/// Creates a bundle of rays from a single point on a square grid in the
/// entrance pupil.
///
/// This is used to model point source fields.
///
/// # Arguments
///
/// * `surfaces` : The surfaces of the system.
/// * `aperture_spec` : The aperture specification.
/// * `paraxial_subview` : The paraxial subview.
/// * `spacing` : The spacing between rays in the grid in normalized pupil
///   distances, i.e. [0, 1]. A spacing of 1.0 means that one ray will lie at
///   the pupil center (the chief ray) and the others will lie at the pupil
///   edge.
/// * `origin` : The origin of the rays, i.e. the field point in the object
///   plane.
fn point_source_ray_bundle_on_sq_grid(
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    spacing: Float,
    origin: &Vec3,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview)?;
    let enp_radius = enp.semi_diameter;
    let abs_spacing = enp_radius * spacing;

    let pupil_ray_positions =
        Vec3::sq_grid_in_circ(enp_radius, abs_spacing, enp.pos().z(), 0.0, 0.0);

    let directions = pupil_ray_positions
        .iter()
        .map(|pos| (*pos - *origin).normalize())
        .collect::<Vec<Vec3>>();

    let rays = directions
        .iter()
        .map(|dir| Ray::new(*origin, *dir))
        .collect::<Vec<Ray>>();

    Ok(rays)
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
            FieldSpec::Angle { angle, .. } => {
                if !angle.is_finite() {
                    return Err(anyhow!("Field angle must be finite"));
                }
            }
            FieldSpec::PointSource { .. } => {
                for submodel in sequential_model.submodels().values() {
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
        sur_z - LAUNCH_POINT_BEFORE_SURFACE
    } else if obj_z == Float::NEG_INFINITY && sur_z > enp_z {
        enp_z - LAUNCH_POINT_BEFORE_PUPIL
    } else {
        obj_z
    }
}

/// Determine the origin of the center of a parallel ray bundle.
///
/// This will be the origin of the ray that pierces the center of the entrance
/// pupil, a.k.a. the chief ray.
///
/// # Arguments
///
/// * `surfaces` - The surfaces of the system.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `theta` - The polar angle of the ray fan in the x-y plane.
/// * `phi` - The angle of the ray w.r.t. the z-axis.
fn parallel_ray_bundle_origin(
    surfaces: &[Surface],
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    theta: Float,
    phi: Float,
) -> Result<Vec3> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview)?;
    let obj_z = surfaces[0].pos().z();
    let sur_z = surfaces[1].pos().z();
    let enp_z = enp.pos().z();

    let launch_point_z = axial_launch_point(obj_z, sur_z, enp_z);

    // Determine the radial distance from the axis at the launch point for the
    // center of the ray fan.
    let dz = enp_z - launch_point_z;

    let r = -dz * phi.tan();
    let x = r * theta.cos();
    let y = r * theta.sin();

    Ok(Vec3::new(x, y, launch_point_z))
}

/// Determine every combination of field ID, wavelength ID, and axis.
fn all_combinations(
    field_specs: &[FieldSpec],
    wavelengths: &[Float],
    axes: &[Axis],
) -> Vec<(usize, usize, Axis)> {
    let mut combinations = Vec::new();

    for (field_id, _field) in field_specs.iter().enumerate() {
        for (wavelength_id, _) in wavelengths.iter().enumerate() {
            for axis in axes {
                combinations.push((field_id, wavelength_id, *axis));
            }
        }
    }

    combinations
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use crate::core::Float;
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
                angle: 0.0,
                pupil_sampling: PupilSampling::TangentialRayFan,
            },
            FieldSpec::Angle {
                angle: 5.0,
                pupil_sampling: PupilSampling::TangentialRayFan,
            },
        ];

        let paraxial_view = ParaxialView::new(&sequential_model, &field_specs, false).unwrap();

        Setup {
            sequential_model,
            aperture_spec,
            field_specs,
            paraxial_view,
        }
    }

    #[test]
    fn test_ray_trace_3d_view() {
        let s = setup();

        let results = ray_trace_3d_view(
            &s.aperture_spec,
            &s.field_specs,
            &s.sequential_model,
            &s.paraxial_view,
            None,
        )
        .unwrap();

        assert_eq!(results.len(), 2); // 2 fields x 1 wavelength x 1 axis
        // (system is rotationally symmetric)
    }

    #[test]
    fn test_rays() {
        let s = setup();

        let rays = rays(
            &s.sequential_model.surfaces(),
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            &s.field_specs[0],
            None,
        )
        .unwrap();

        assert_eq!(rays.len(), 3); // 3 rays for tangential ray fan
    }

    #[test]
    fn test_rays_point_source_incompatible_with_finite_object() {
        let s = setup();

        let field_spec = FieldSpec::PointSource {
            x: 0.0,
            y: 0.0,
            pupil_sampling: PupilSampling::TangentialRayFan,
        };

        let result = rays(
            &s.sequential_model.surfaces(),
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            &field_spec,
            None,
        );

        assert!(
            result.is_err(),
            "Expected Err result because FieldSpec::PointSource is incompatible with objects at infinity. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_chief_ray_from_on_axis_field_angle() {
        let s = setup();

        let rays = chief_ray_from_angle(
            &s.sequential_model.surfaces(),
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            0.0,
            0.0,
        )
        .unwrap();

        assert_eq!(rays.len(), 1);
        assert_abs_diff_eq!(rays[0].x(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].y(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].z(), -LAUNCH_POINT_BEFORE_SURFACE, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].k(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].l(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].m(), 1.0, epsilon = 1e-4);
    }

    #[test]
    fn test_chief_ray_from_off_axis_field_angle() {
        let s = setup();

        let rays = chief_ray_from_angle(
            &s.sequential_model.surfaces(),
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            PI / 2.0,
            0.08727, // 5 degrees
        )
        .unwrap();

        assert_eq!(rays.len(), 1);
        assert_abs_diff_eq!(rays[0].x(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].y(), -0.8749, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].z(), -LAUNCH_POINT_BEFORE_SURFACE, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].k(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].l(), 0.08716, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].m(), 0.9962, epsilon = 1e-4);
    }

    #[test]
    fn test_chief_ray_from_on_axis_pos() {
        let s = setup();

        let rays = chief_ray_from_pos(
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            &Vec3::new(0.0, 0.0, -1.0),
        )
        .unwrap();

        assert_eq!(rays.len(), 1);
        assert_abs_diff_eq!(rays[0].x(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].y(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].z(), -1.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].k(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].l(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].m(), 1.0, epsilon = 1e-4);
    }

    #[test]
    fn test_chief_ray_from_off_axis_pos() {
        let s = setup();

        let rays = chief_ray_from_pos(
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            &Vec3::new(0.0, -0.08749, -1.0),
        )
        .unwrap();

        assert_eq!(rays.len(), 1);
        assert_abs_diff_eq!(rays[0].x(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].y(), -0.08749, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].z(), -1.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].k(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].l(), 0.08716, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].m(), 0.9962, epsilon = 1e-4);
    }

    #[test]
    fn test_parallel_ray_bundle_on_sq_grid() {
        let s = setup();

        let rays = parallel_ray_bundle_on_sq_grid(
            &s.sequential_model.surfaces(),
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            1.0,
            0.0,
        );

        // The grid should have 5 points: one in the center and four at the points where
        // the inscribed circle touches the square.
        assert_eq!(rays.unwrap().len(), 5);

        let rays = parallel_ray_bundle_on_sq_grid(
            &s.sequential_model.surfaces(),
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            0.5,
            0.0,
        );

        // Halving the spacing results in 13 out of 25 points in the grid.
        assert_eq!(rays.unwrap().len(), 13);
    }

    #[test]
    fn test_point_source_ray_fan() {
        let s = setup();
        let enp_radius = &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)]
            .entrance_pupil()
            .semi_diameter;
        let expected_z_dir_cosines: [Float; 3] = [
            (-enp_radius.atan2(1.0)).cos(),
            1.0,
            enp_radius.atan2(1.0).cos(),
        ];

        let rays = point_source_ray_fan(
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            3,
            PI / 2.0,
            &Vec3::new(0.0, 0.0, -1.0), // Point source located at z = -1.0
        )
        .unwrap();

        // The fan should have 3 rays: one in the center and two at the pupil edge.
        assert_eq!(rays.len(), 3);

        // Check the directions of the rays.
        for (dir, ray) in expected_z_dir_cosines.iter().zip(rays.iter()) {
            assert_abs_diff_eq!(*dir, ray.m(), epsilon = 1e-4);
        }
    }

    #[test]
    fn test_point_source_ray_bundle_on_sq_grid() {
        let s = setup();

        let rays = point_source_ray_bundle_on_sq_grid(
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            1.0,
            &Vec3::new(0.0, 0.0, -1.0), // Point source located at z = -1.0
        );

        // The grid should have 5 points: one in the center and four at the points where
        // the inscribed circle touches the square.
        assert_eq!(rays.unwrap().len(), 5);

        let rays = point_source_ray_bundle_on_sq_grid(
            &s.aperture_spec,
            &s.paraxial_view.subviews()[&SubModelID(0, Axis::Y)],
            0.5,
            &Vec3::new(0.0, 0.0, -1.0), // Point source located at z = -1.0
        );

        // Halving the spacing results in 13 out of 25 points in the grid.
        assert_eq!(rays.unwrap().len(), 13);
    }

    #[test]
    fn test_validate_field_specs() {
        let s = setup();

        let field_specs = vec![FieldSpec::Angle {
            angle: 0.0,
            pupil_sampling: PupilSampling::TangentialRayFan,
        }];

        let result = validate_field_specs(&s.sequential_model, &field_specs);
        assert!(
            result.is_ok(),
            "Expected Ok result because FieldSpec::Angle is compatible with objects at infinity. Result: {:?}",
            result
        );

        let field_specs = vec![FieldSpec::PointSource {
            x: 0.0,
            y: 0.0,
            pupil_sampling: PupilSampling::TangentialRayFan,
        }];

        let result = validate_field_specs(&s.sequential_model, &field_specs);
        assert!(
            result.is_err(),
            "Expected Err result because FieldSpec::PointSource is incompatible with objects at infinity. Result: {:?}",
            result
        );
    }

    #[test]
    fn test_all_combinations() {
        let field_specs = vec![
            FieldSpec::Angle {
                angle: 0.0,
                pupil_sampling: PupilSampling::TangentialRayFan,
            },
            FieldSpec::Angle {
                angle: 5.0,
                pupil_sampling: PupilSampling::TangentialRayFan,
            },
        ];

        let wavelengths = vec![0.4861, 0.5876, 0.6563];
        let axes = vec![Axis::X, Axis::Y];

        let combinations = all_combinations(&field_specs, &wavelengths, &axes);

        assert_eq!(combinations.len(), 12); // 2 fields x 3 wavelengths x 2 axes
        assert!(combinations.contains(&(0, 0, Axis::X)));
        assert!(combinations.contains(&(0, 0, Axis::Y)));
        assert!(combinations.contains(&(0, 1, Axis::X)));
        assert!(combinations.contains(&(0, 1, Axis::Y)));
        assert!(combinations.contains(&(0, 2, Axis::X)));
        assert!(combinations.contains(&(0, 2, Axis::Y)));
        assert!(combinations.contains(&(1, 0, Axis::X)));
        assert!(combinations.contains(&(1, 0, Axis::Y)));
        assert!(combinations.contains(&(1, 1, Axis::X)));
        assert!(combinations.contains(&(1, 1, Axis::Y)));
        assert!(combinations.contains(&(1, 2, Axis::X)));
        assert!(combinations.contains(&(1, 2, Axis::Y)));
    }
}
