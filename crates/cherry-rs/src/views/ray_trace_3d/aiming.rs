/// Constructs the initial rays for a 3D ray trace from a field spec,
/// aperture spec, and pupil-sampling strategy.
use anyhow::{Result, anyhow};
use tracing::trace;

use crate::{
    Pupil,
    core::{
        Float, PI, math::vec3::Vec3, ray::Ray,
        sequential_model::surface_placement::SurfacePlacement,
    },
    specs::{
        aperture::ApertureSpec,
        fields::{FieldSpec, PupilSampling},
    },
};

use super::super::paraxial::ParaxialSubView;

/// The distance to launch the rays before the first surface when the object is
/// at infinity.
const LAUNCH_POINT_BEFORE_SURFACE: Float = 10.0;

/// The distance to launch the rays before the entrance pupil when the object is
/// at infinity.
const LAUNCH_POINT_BEFORE_PUPIL: Float = 10.0;

/// Synthetic local axis frame for one path's own ray-aiming computations.
///
/// Every function below this point computes in this LOCAL frame (where the
/// origin is this path's own first surface, and +Z is the direction the
/// path travels in before that surface's own tilt is applied), then
/// transforms the result to global coordinates via
/// [`SurfacePlacement::to_global`] / [`Ray::i_transform`] — exactly the same
/// mechanism `trace.rs` already relies on for every surface, regardless of
/// prior folds.
///
/// This exists because naive global-axis arithmetic (as this module used to
/// use directly) implicitly assumes a path's own object sits on the global Z
/// axis with X=Y=0 — true for path 0 by construction (nothing precedes it),
/// but not for a later path whose object was placed via
/// `PathSurfaceRef::ObjectLinkedTo` from a folded upstream path.
struct PathAxis {
    /// Anchored at this path's own first surface (`surface_indices[1]`,
    /// always finite), using the rotation of this path's own object's
    /// placement (`surface_indices[0]`, always finite/well-defined even
    /// when the object's *position* is at infinity, since no reflection
    /// occurs between the object and the first surface — an Object surface
    /// is always built with an identity actual rotation, so its
    /// `rotation_matrix` exactly equals the incoming cursor rotation).
    placement: SurfacePlacement,

    /// This path's own object, as a local F-axis (local Z) coordinate.
    /// `Float::NEG_INFINITY` when the object is at infinity.
    obj_z: Float,
}

impl PathAxis {
    fn resolve(placements: &[SurfacePlacement], surface_indices: &[usize]) -> Result<Self> {
        if surface_indices.len() < 2 {
            return Err(anyhow!(
                "A path needs at least an object and a first surface"
            ));
        }
        let obj = &placements[surface_indices[0]];
        let first_surf = &placements[surface_indices[1]];

        let placement = SurfacePlacement::new(
            first_surf.position,
            first_surf.track,
            obj.rotation_matrix,
            obj.rotation_matrix,
        );
        let obj_z = if obj.is_infinite() {
            Float::NEG_INFINITY
        } else {
            placement.to_local(obj.position).z()
        };

        Ok(Self { placement, obj_z })
    }
}

/// Returns the initial rays in a ray bundle to trace through the system.
///
/// # Arguments
///
/// * `placements` - The global placements of every surface in the model.
/// * `surface_indices` - This path's own step-to-global-index mapping.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview. This is used to obtain the
///   pupil.
/// * `field_spec` - The field specification.
/// * `sampling` - The pupil sampling method.
pub(super) fn rays(
    placements: &[SurfacePlacement],
    surface_indices: &[usize],
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    field_spec: &FieldSpec,
    sampling: PupilSampling,
) -> Result<Vec<Ray>> {
    let axis = PathAxis::resolve(placements, surface_indices)?;

    let rays: Vec<Ray> = match field_spec {
        FieldSpec::Angle { chi, phi } => {
            let chi_rad = chi.to_radians();
            let phi_rad = phi.to_radians();

            match sampling {
                PupilSampling::ChiefRay => {
                    chief_ray_from_angle(&axis, aperture_spec, paraxial_subview, phi_rad, chi_rad)?
                }
                PupilSampling::SquareGrid { spacing } => parallel_ray_bundle_on_sq_grid(
                    &axis,
                    aperture_spec,
                    paraxial_subview,
                    spacing,
                    chi_rad,
                )?,
                PupilSampling::TangentialRayFan { n } => {
                    let tan_phi = field_spec.tangential_fan_phi();
                    parallel_ray_fan(
                        &axis,
                        aperture_spec,
                        paraxial_subview,
                        n,
                        tan_phi,
                        tan_phi,
                        chi_rad,
                    )?
                }
                PupilSampling::SagittalRayFan { n } => {
                    let tan_phi = field_spec.tangential_fan_phi();
                    let sag_phi = field_spec.sagittal_fan_phi();
                    parallel_ray_fan(
                        &axis,
                        aperture_spec,
                        paraxial_subview,
                        n,
                        tan_phi,
                        sag_phi,
                        chi_rad,
                    )?
                }
            }
        }

        FieldSpec::PointSource { x, y } => {
            if axis.obj_z.is_infinite() {
                return Err(anyhow!(
                    "Cannot have a point source field with an infinite object distance"
                ));
            }

            let origin = axis.placement.to_global(Vec3::new(*x, *y, axis.obj_z));
            match sampling {
                PupilSampling::ChiefRay => {
                    chief_ray_from_pos(&axis, aperture_spec, paraxial_subview, &origin)?
                }
                PupilSampling::SquareGrid { spacing } => point_source_ray_bundle_on_sq_grid(
                    &axis,
                    aperture_spec,
                    paraxial_subview,
                    spacing,
                    &origin,
                )?,
                PupilSampling::TangentialRayFan { n } => {
                    let fan_phi = field_spec.tangential_fan_phi();
                    point_source_ray_fan(
                        &axis,
                        aperture_spec,
                        paraxial_subview,
                        n,
                        fan_phi,
                        &origin,
                    )?
                }
                PupilSampling::SagittalRayFan { n } => {
                    let fan_phi = field_spec.sagittal_fan_phi();
                    point_source_ray_fan(
                        &axis,
                        aperture_spec,
                        paraxial_subview,
                        n,
                        fan_phi,
                        &origin,
                    )?
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
/// * `axis` - This path's own local axis frame.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `phi` - The azimuthal angle of the ray in the x-y plane, radians.
/// * `chi` - The zenith angle of the ray w.r.t. the z-axis, radians.
fn chief_ray_from_angle(
    axis: &PathAxis,
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    phi: Float,
    chi: Float,
) -> Result<Vec<Ray>> {
    let origin = parallel_ray_bundle_origin(axis, aperture_spec, paraxial_subview, phi, chi);
    let dir = Vec3::new(phi.cos() * chi.sin(), phi.sin() * chi.sin(), chi.cos()).normalize();

    let mut ray = Ray::new(origin, dir);
    ray.i_transform(&axis.placement);

    Ok(vec![ray])
}

/// Creates the chief ray for a given field position.
///
/// # Arguments
///
/// * `axis` - This path's own local axis frame.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `origin` - The origin of the ray, i.e. the field point in the object
///   plane, in GLOBAL coordinates.
fn chief_ray_from_pos(
    axis: &PathAxis,
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    origin: &Vec3,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview);
    let pupil_global = axis.placement.to_global(Vec3::new(0.0, 0.0, enp.location));
    let dir = orient_forward(axis, (pupil_global - *origin).normalize());

    Ok(vec![Ray::new(*origin, dir)])
}

/// Orients a global direction vector to point forward — positive local Z in
/// `axis`'s own frame — flipping it if necessary.
///
/// The (possibly virtual) entrance pupil can lie either ahead of or behind
/// the object along the line a chief/pupil-sampled ray is built from; a
/// naive difference of two points on that line points backward whenever the
/// pupil is the nearer of the two. `Surface::intersect`'s solver doesn't
/// care about a ray's direction sign, so a backward-pointing ray can still
/// "find" an intersection — but every downstream `interact()` call assumes
/// the ray it's given is genuinely traveling forward, so the direction
/// handed to the tracer must always continue into the optical system, not
/// back out through the object.
fn orient_forward(axis: &PathAxis, dir: Vec3) -> Vec3 {
    if (axis.placement.rotation_matrix * dir).z() < 0.0 {
        -dir
    } else {
        dir
    }
}

/// Creates a fan of parallel rays that passes through the entrance pupil.
///
/// This is used to model field angles.
///
/// # Arguments
///
/// * `axis` - This path's own local axis frame.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `num_rays` - The number of rays in the fan.
/// * `fan_origin_phi` - The azimuthal angle of the fan's origin point, radians.
/// * `fan_spread_phi` - The azimuthal angle of the plane containing the ray
///   fan, radians.
/// * `chi` - The zenith angle of the ray w.r.t. the z-axis, radians.
#[allow(clippy::too_many_arguments)]
fn parallel_ray_fan(
    axis: &PathAxis,
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    num_rays: usize,
    fan_origin_phi: Float,
    fan_spread_phi: Float,
    chi: Float,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview);
    let origin =
        parallel_ray_bundle_origin(axis, aperture_spec, paraxial_subview, fan_origin_phi, chi);

    let mut rays = Ray::parallel_ray_fan(
        num_rays,
        enp.semi_diameter,
        origin.z(),
        fan_spread_phi,
        fan_origin_phi,
        chi,
        origin.x(),
        origin.y(),
    );

    if let Some(center) = rays.get(num_rays / 2) {
        trace!(
            fan_origin_phi,
            fan_spread_phi,
            chi,
            origin_x = origin.x(),
            origin_y = origin.y(),
            origin_z = origin.z(),
            dir_x = center.l(),
            dir_y = center.m(),
            dir_z = center.n(),
            "parallel_ray_fan: center ray (p=0) local origin and direction"
        );
    }

    for ray in rays.iter_mut() {
        ray.i_transform(&axis.placement);
    }

    Ok(rays)
}

/// Creates a bundle of parallel rays on a square grid in the entrance pupil.
///
/// This is used to model field angles.
///
/// # Arguments
///
/// * `axis` - This path's own local axis frame.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `spacing` - The spacing between rays in the grid in normalized pupil
///   distances, i.e. [0, 1]. A spacing of 1.0 means that one ray will lie at
///   the pupil center (the chief ray) and the others will lie at the pupil edge
///   (marginal rays).
/// * `chi` - The zenith angle of the ray bundle w.r.t. the z-axis in radians.
fn parallel_ray_bundle_on_sq_grid(
    axis: &PathAxis,
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    spacing: Float,
    chi: Float,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview);
    let abs_spacing = enp.semi_diameter * spacing;
    let origin = parallel_ray_bundle_origin(axis, aperture_spec, paraxial_subview, PI / 2.0, chi);

    let mut rays = Ray::parallel_ray_bundle_on_sq_grid(
        enp.semi_diameter,
        abs_spacing,
        origin.z(),
        chi,
        origin.x(),
        origin.y(),
    );

    for ray in rays.iter_mut() {
        ray.i_transform(&axis.placement);
    }

    Ok(rays)
}

/// Creates a fan of rays from a single point source that passes through the
/// center of the pupil.
///
/// This is used to model point source fields.
///
/// # Arguments
///
/// * `axis` - This path's own local axis frame.
/// * `aperture_spec` : The aperture specification.
/// * `paraxial_subview` : The paraxial subview.
/// * `theta` - The polar angle of the ray fan in the x-y plane.
/// * `origin` : The origin of the rays, i.e. the field point in the object
///   plane, in GLOBAL coordinates.
fn point_source_ray_fan(
    axis: &PathAxis,
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    num_rays: usize,
    theta: Float,
    origin: &Vec3,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview);
    let enp_radius = enp.semi_diameter;
    let enp_z = enp.location;

    let pupil_ray_positions = Vec3::fan(num_rays, enp_radius, enp_z, theta, 0.0, 0.0);

    let directions = pupil_ray_positions
        .iter()
        .map(|pos| orient_forward(axis, (axis.placement.to_global(*pos) - *origin).normalize()))
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
/// * `axis` - This path's own local axis frame.
/// * `aperture_spec` : The aperture specification.
/// * `paraxial_subview` : The paraxial subview.
/// * `spacing` : The spacing between rays in the grid in normalized pupil
///   distances, i.e. [0, 1]. A spacing of 1.0 means that one ray will lie at
///   the pupil center (the chief ray) and the others will lie at the pupil
///   edge.
/// * `origin` : The origin of the rays, i.e. the field point in the object
///   plane, in GLOBAL coordinates.
fn point_source_ray_bundle_on_sq_grid(
    axis: &PathAxis,
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    spacing: Float,
    origin: &Vec3,
) -> Result<Vec<Ray>> {
    let enp = entrance_pupil(aperture_spec, paraxial_subview);
    let enp_radius = enp.semi_diameter;
    let abs_spacing = enp_radius * spacing;

    let pupil_ray_positions =
        Vec3::sq_grid_in_circ(enp_radius, abs_spacing, enp.location, 0.0, 0.0);

    let directions = pupil_ray_positions
        .iter()
        .map(|pos| orient_forward(axis, (axis.placement.to_global(*pos) - *origin).normalize()))
        .collect::<Vec<Vec3>>();

    let rays = directions
        .iter()
        .map(|dir| Ray::new(*origin, *dir))
        .collect::<Vec<Ray>>();

    Ok(rays)
}

/// Determines the entrance pupil of the subview, in this path's own local
/// axis frame.
///
/// `ParaxialSubView::entrance_pupil()` already reports its `location`
/// relative to this path's own first non-object surface (see [`Pupil`]'s
/// docs) — exactly the anchor `PathAxis` uses — so no further translation is
/// needed here.
fn entrance_pupil(aperture_spec: &ApertureSpec, paraxial_sub_view: &ParaxialSubView) -> Pupil {
    let semi_diameter = match aperture_spec {
        ApertureSpec::EntrancePupil { semi_diameter } => *semi_diameter,
    };

    Pupil {
        location: paraxial_sub_view.entrance_pupil().location,
        semi_diameter,
    }
}

/// Determine the axial (local Z) launch point for the rays.
///
/// If the object plane is at infinity, and if the first surface (local
/// Z = 0, by construction of [`PathAxis`]) lies before the entrance pupil,
/// then launch the rays from one unit to the left of the first surface. If
/// the object plane is at infinity, and it comes after the entrance pupil,
/// then launch the rays from one unit in front of the entrance pupil.
/// Otherwise, launch the rays from the object plane.
fn axial_launch_point(obj_z: Float, enp_z: Float) -> Float {
    if obj_z == Float::NEG_INFINITY && enp_z >= 0.0 {
        -LAUNCH_POINT_BEFORE_SURFACE
    } else if obj_z == Float::NEG_INFINITY {
        enp_z - LAUNCH_POINT_BEFORE_PUPIL
    } else {
        obj_z
    }
}

/// Determine the local-frame origin of the center of a parallel ray bundle.
///
/// This will be the origin of the ray that pierces the center of the entrance
/// pupil, a.k.a. the chief ray.
///
/// # Arguments
///
/// * `axis` - This path's own local axis frame.
/// * `aperture_spec` - The aperture specification.
/// * `paraxial_subview` - The paraxial subview.
/// * `phi` - The azimuthal angle of the ray fan in the x-y plane, radians.
/// * `chi` - The zenith angle of the ray w.r.t. the z-axis, radians.
fn parallel_ray_bundle_origin(
    axis: &PathAxis,
    aperture_spec: &ApertureSpec,
    paraxial_subview: &ParaxialSubView,
    phi: Float,
    chi: Float,
) -> Vec3 {
    let enp = entrance_pupil(aperture_spec, paraxial_subview);
    let enp_z = enp.location;

    let launch_point_z = axial_launch_point(axis.obj_z, enp_z);

    // Determine the perpendicular distance from the axis at the launch point for
    // the center of the ray fan.
    let dz = enp_z - launch_point_z;
    let r = -dz * chi.tan();

    let x = r * phi.cos();
    let y = r * phi.sin();

    Vec3::new(x, y, launch_point_z)
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use crate::core::Float;
    use crate::examples::convexplano_lens::sequential_model;
    use crate::n;

    use super::super::super::paraxial::ParaxialView;
    use super::super::{SamplingConfig, ray_trace_3d_view};
    use super::*;

    /// Regression: `chief_ray_from_pos` must always produce a ray whose
    /// direction continues *forward* (positive local Z in the path's own
    /// `PathAxis` frame) into the optical system, even when the paraxial
    /// entrance pupil is virtual and located *behind* the object along the
    /// chief-ray line — a real, common case (`wf_epi_excitation`'s own
    /// entrance pupil sits at -54.5455, further behind the object at -40,
    /// per its own ground-truth test constants).
    ///
    /// The naive `pupil - origin` difference points backward in that case.
    /// `Surface::intersect`'s solver doesn't care about a ray's direction
    /// sign, so a backward-pointing ray can still "find" an intersection —
    /// this bug was invisible only because the (separately, now-fixed) bug
    /// in `Surface::interact`'s default refraction and `ThinLens::interact`
    /// unconditionally forced their *output* direction forward regardless
    /// of the input sign, silently absorbing this one.
    #[test]
    fn chief_ray_from_pos_points_forward_even_when_pupil_is_virtual_and_behind_the_object() {
        use crate::examples::wf_epi_excitation::sequential_model as wf_epi_excitation_model;

        let model = wf_epi_excitation_model(n!(1.0), n!(1.5), &[0.5876]);
        let placements = model.placements();
        let surface_indices = model.path_surface_indices(0);
        let field_specs = vec![FieldSpec::PointSource { x: 0.0, y: 1.5 }];
        let aperture_spec = ApertureSpec::EntrancePupil {
            semi_diameter: 1.5454,
        };
        let paraxial_view =
            ParaxialView::new(&model, std::slice::from_ref(&field_specs), false).unwrap();
        let paraxial_subview = paraxial_view.get(0, 0).unwrap();

        let axis = PathAxis::resolve(placements, surface_indices).unwrap();
        // Sanity: this fixture's entrance pupil really is behind the
        // object, pinning the scenario this test targets.
        assert!(
            paraxial_subview.entrance_pupil().location < axis.obj_z,
            "test fixture assumption broken: pupil should be behind the object"
        );

        let generated = rays(
            placements,
            surface_indices,
            &aperture_spec,
            paraxial_subview,
            &field_specs[0],
            PupilSampling::ChiefRay,
        )
        .expect("rays should generate");

        let local_dir = axis.placement.rotation_matrix * generated[0].dir();
        assert!(
            local_dir.z() > 0.0,
            "chief ray must point forward (positive local Z) even when the \
             pupil is behind the object, got {local_dir:?}"
        );
    }

    struct Setup {
        sequential_model: crate::SequentialModel,
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
    fn test_rays() {
        let s = setup();

        let rays = rays(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            &s.field_specs[0],
            PupilSampling::TangentialRayFan { n: 3 },
        )
        .unwrap();

        assert_eq!(rays.len(), 3); // 3 rays for tangential ray fan
    }

    #[test]
    fn test_rays_sagittal_fan() {
        // For phi=90° (U/YZ plane), the tangential fan spreads along Y;
        // the sagittal fan must spread along X (perpendicular to the meridional plane).
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let seq_model = sequential_model(air, nbk7, &wavelengths);
        let aperture_spec = ApertureSpec::EntrancePupil {
            semi_diameter: 12.5,
        };
        let field_specs = vec![FieldSpec::Angle {
            chi: 5.0,
            phi: 90.0,
        }];
        let paraxial_view =
            ParaxialView::new(&seq_model, std::slice::from_ref(&field_specs), false).unwrap();

        let fan_rays = rays(
            seq_model.placements(),
            seq_model.path_surface_indices(0),
            &aperture_spec,
            paraxial_view.get(0, 0).unwrap(),
            &field_specs[0],
            PupilSampling::SagittalRayFan { n: 3 },
        )
        .unwrap();

        assert_eq!(fan_rays.len(), 3);

        // fan_spread_phi = sagittal_fan_phi = tangential_fan_phi + π/2 = π/2 + π/2 = π.
        // Vec3::fan with phi=π spreads positions along (cos π, sin π) = (-1, 0),
        // so the three ray origins differ in X and share the same Y coordinate.
        let xs: Vec<f64> = fan_rays.iter().map(|r| r.x()).collect();
        let ys: Vec<f64> = fan_rays.iter().map(|r| r.y()).collect();
        assert!(xs.iter().any(|x| (x - xs[0]).abs() > 1e-6));
        assert!(ys.windows(2).all(|w| (w[1] - w[0]).abs() < 1e-6));
        // fan_origin_phi = tangential_fan_phi, so the center ray (index 1 of 3)
        // is displaced in the tangential (Y) direction, not zero.
        assert!(
            ys[1].abs() > 1e-6,
            "sagittal fan center should be offset in the tangential direction"
        );
    }

    /// Regression test: The sagittal fan rays must travel in the field
    /// direction, not in the sagittal-plane direction.  For phi=90°, chi=5°
    /// the field direction is (0, sin 5°, cos 5°); the sagittal fan
    /// incorrectly used fan_spread_phi for the ray direction, giving (-sin
    /// 5°, 0, cos 5°).
    #[test]
    fn test_sagittal_fan_ray_direction_matches_field() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let seq_model = sequential_model(air, nbk7, &wavelengths);
        let aperture_spec = ApertureSpec::EntrancePupil {
            semi_diameter: 12.5,
        };
        let field_spec = FieldSpec::Angle {
            chi: 5.0,
            phi: 90.0,
        };
        let paraxial_view = ParaxialView::new(&seq_model, &[vec![field_spec]], false).unwrap();

        let chief = rays(
            seq_model.placements(),
            seq_model.path_surface_indices(0),
            &aperture_spec,
            paraxial_view.get(0, 0).unwrap(),
            &field_spec,
            PupilSampling::ChiefRay,
        )
        .unwrap();
        let sag_fan = rays(
            seq_model.placements(),
            seq_model.path_surface_indices(0),
            &aperture_spec,
            paraxial_view.get(0, 0).unwrap(),
            &field_spec,
            PupilSampling::SagittalRayFan { n: 3 },
        )
        .unwrap();

        let chief_dir = (chief[0].l(), chief[0].m(), chief[0].n());

        // Every sagittal fan ray must travel in the same direction as the chief ray.
        for (i, r) in sag_fan.iter().enumerate() {
            let dir = (r.l(), r.m(), r.n());
            assert!(
                (dir.0 - chief_dir.0).abs() < 1e-10
                    && (dir.1 - chief_dir.1).abs() < 1e-10
                    && (dir.2 - chief_dir.2).abs() < 1e-10,
                "sagittal fan ray {i} direction {dir:?} != chief ray direction {chief_dir:?}"
            );
        }
    }

    #[test]
    fn test_rays_point_source_incompatible_with_finite_object() {
        let s = setup();

        let field_spec = FieldSpec::PointSource { x: 0.0, y: 0.0 };

        let result = rays(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            &field_spec,
            PupilSampling::TangentialRayFan { n: 3 },
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
        let axis = PathAxis::resolve(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
        )
        .unwrap();

        let rays = chief_ray_from_angle(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            0.0,
            0.0,
        )
        .unwrap();

        assert_eq!(rays.len(), 1);
        assert_abs_diff_eq!(rays[0].x(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].y(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].z(), -LAUNCH_POINT_BEFORE_SURFACE, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].l(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].m(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].n(), 1.0, epsilon = 1e-4);
    }

    #[test]
    fn test_chief_ray_from_off_axis_field_angle() {
        let s = setup();
        let axis = PathAxis::resolve(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
        )
        .unwrap();

        let rays = chief_ray_from_angle(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            PI / 2.0,
            0.08727, // 5 degrees
        )
        .unwrap();

        assert_eq!(rays.len(), 1);
        assert_abs_diff_eq!(rays[0].x(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].y(), -0.8749, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].z(), -LAUNCH_POINT_BEFORE_SURFACE, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].l(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].m(), 0.08716, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].n(), 0.9962, epsilon = 1e-4);
    }

    #[test]
    fn test_chief_ray_from_on_axis_pos() {
        let s = setup();
        let axis = PathAxis::resolve(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
        )
        .unwrap();

        let rays = chief_ray_from_pos(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            &Vec3::new(0.0, 0.0, -1.0),
        )
        .unwrap();

        assert_eq!(rays.len(), 1);
        assert_abs_diff_eq!(rays[0].x(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].y(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].z(), -1.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].l(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].m(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].n(), 1.0, epsilon = 1e-4);
    }

    #[test]
    fn test_chief_ray_from_off_axis_pos() {
        let s = setup();
        let axis = PathAxis::resolve(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
        )
        .unwrap();

        let rays = chief_ray_from_pos(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            &Vec3::new(0.0, -0.08749, -1.0),
        )
        .unwrap();

        assert_eq!(rays.len(), 1);
        assert_abs_diff_eq!(rays[0].x(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].y(), -0.08749, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].z(), -1.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].l(), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].m(), 0.08716, epsilon = 1e-4);
        assert_abs_diff_eq!(rays[0].n(), 0.9962, epsilon = 1e-4);
    }

    #[test]
    fn test_parallel_ray_bundle_on_sq_grid() {
        let s = setup();
        let axis = PathAxis::resolve(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
        )
        .unwrap();

        let rays = parallel_ray_bundle_on_sq_grid(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            1.0,
            0.0,
        );

        // The grid should have 5 points: one in the center and four at the points where
        // the inscribed circle touches the square.
        assert_eq!(rays.unwrap().len(), 5);

        let rays = parallel_ray_bundle_on_sq_grid(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            0.5,
            0.0,
        );

        // Halving the spacing results in 13 out of 25 points in the grid.
        assert_eq!(rays.unwrap().len(), 13);
    }

    #[test]
    fn test_point_source_ray_fan() {
        let s = setup();
        let axis = PathAxis::resolve(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
        )
        .unwrap();
        let enp_radius = s
            .paraxial_view
            .get(0, 0)
            .unwrap()
            .entrance_pupil()
            .semi_diameter;
        let expected_z_dir_cosines: [Float; 3] = [
            (-enp_radius.atan2(1.0)).cos(),
            1.0,
            enp_radius.atan2(1.0).cos(),
        ];

        let rays = point_source_ray_fan(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            3,
            PI / 2.0,
            &Vec3::new(0.0, 0.0, -1.0), // Point source located at z = -1.0
        )
        .unwrap();

        // The fan should have 3 rays: one in the center and two at the pupil edge.
        assert_eq!(rays.len(), 3);

        // Check the directions of the rays.
        for (dir, ray) in expected_z_dir_cosines.iter().zip(rays.iter()) {
            assert_abs_diff_eq!(*dir, ray.n(), epsilon = 1e-4);
        }
    }

    #[test]
    fn test_point_source_ray_bundle_on_sq_grid() {
        let s = setup();
        let axis = PathAxis::resolve(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
        )
        .unwrap();

        let rays = point_source_ray_bundle_on_sq_grid(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            1.0,
            &Vec3::new(0.0, 0.0, -1.0), // Point source located at z = -1.0
        );

        // The grid should have 5 points: one in the center and four at the points where
        // the inscribed circle touches the square.
        assert_eq!(rays.unwrap().len(), 5);

        let rays = point_source_ray_bundle_on_sq_grid(
            &axis,
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            0.5,
            &Vec3::new(0.0, 0.0, -1.0), // Point source located at z = -1.0
        );

        // Halving the spacing results in 13 out of 25 points in the grid.
        assert_eq!(rays.unwrap().len(), 13);
    }

    /// A tangential fan at phi=45° should produce rays whose pupil positions
    /// are spread along the (1,1)/√2 diagonal, not along the Y or X axis.
    #[test]
    fn test_diagonal_tangential_fan_orientation() {
        let s = setup();

        let field_spec = FieldSpec::Angle {
            chi: 5.0,
            phi: 45.0, // diagonal: fan should run along (1,1)/√2
        };

        let generated = rays(
            s.sequential_model.placements(),
            s.sequential_model.path_surface_indices(0),
            &s.aperture_spec,
            s.paraxial_view.get(0, 0).unwrap(),
            &field_spec,
            PupilSampling::TangentialRayFan { n: 3 },
        )
        .expect("rays should be generated for diagonal field");

        assert_eq!(generated.len(), 3);

        // The three fan rays should be spread along the (1,1)/√2 direction.
        // That means x and y offsets from the center ray should be equal in magnitude.
        // Chief ray is the middle ray of the fan (index 1 of 3)
        let center_x = generated[1].x();
        let center_y = generated[1].y();
        let outer_x = generated[2].x();
        let outer_y = generated[2].y();
        let dx = outer_x - center_x;
        let dy = outer_y - center_y;

        // For a 45° fan, dy/dx should equal tan(45°) = 1.0
        assert_abs_diff_eq!(dy / dx, 1.0, epsilon = 1e-6);
    }

    /// Builds a small 2-path fixture where path 1's own object is placed via
    /// `ObjectLinkedTo` from a path 0 that folds (45°-tilted mirror) before
    /// path 1's own object — so path 1's own object→first-surface segment is
    /// NOT aligned with the global Z axis, unlike path 0's. Path 1's own
    /// first surface is also its own aperture stop (the only finite-aperture
    /// surface on that path), reproducing the `wf_epi_microscope` defect
    /// pattern in miniature, without needing any `Shared` surfaces.
    fn folded_object_fixture() -> crate::SequentialModel {
        use std::rc::Rc;

        use crate::RefractiveIndexSpec;
        use crate::core::math::linalg::rotations::{EulerAngles, Rotation3D};
        use crate::core::sequential_model::builder::SequentialModelBuilder;
        use crate::specs::gaps::{ConstantRefractiveIndex, GapSpec};
        use crate::specs::paths::{LinkedObjectOrientation, PathSpec, PathSurfaceRef};
        use crate::specs::surfaces::{BoundaryKind, SurfaceSpec};

        let n_air: Rc<dyn RefractiveIndexSpec> = Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let mirror_rotation =
            Rotation3D::IntrinsicPassiveRUF(EulerAngles((45.0_f64).to_radians(), 0.0, 0.0));

        let img = || SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };

        let path0 = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::New(SurfaceSpec::Object),
                PathSurfaceRef::New(SurfaceSpec::Conic {
                    semi_diameter: 20.0,
                    radius_of_curvature: Float::INFINITY,
                    conic_constant: 0.0,
                    surf_kind: BoundaryKind::Reflecting,
                    rotation: mirror_rotation,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
                PathSurfaceRef::New(img()),
            ],
            gaps: vec![
                GapSpec {
                    thickness: 50.0,
                    refractive_index: n_air.clone(),
                },
                GapSpec {
                    thickness: 50.0,
                    refractive_index: n_air.clone(),
                },
            ],
            beam_splitter_arms: vec![],
            stop_surface: None,
            wavelengths: vec![0.5876],
        };

        let path1 = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::ObjectLinkedTo {
                    path: 0,
                    orientation: LinkedObjectOrientation::Reversed,
                },
                PathSurfaceRef::New(SurfaceSpec::Conic {
                    semi_diameter: 5.0,
                    radius_of_curvature: Float::INFINITY,
                    conic_constant: 0.0,
                    surf_kind: BoundaryKind::Refracting,
                    rotation: Rotation3D::None,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
                PathSurfaceRef::New(img()),
            ],
            gaps: vec![
                GapSpec {
                    thickness: 20.0,
                    refractive_index: n_air.clone(),
                },
                GapSpec {
                    thickness: 30.0,
                    refractive_index: n_air.clone(),
                },
            ],
            beam_splitter_arms: vec![],
            stop_surface: None,
            wavelengths: vec![0.5876],
        };

        SequentialModelBuilder::new()
            .paths(vec![path0, path1])
            .build()
            .expect("fixture model builds")
            .model
    }

    #[test]
    fn path_axis_resolves_to_this_paths_own_object_frame() {
        let model = folded_object_fixture();
        let placements = model.placements();
        let surface_indices = model.path_surface_indices(1);

        let axis = PathAxis::resolve(placements, surface_indices).expect("axis resolves");

        let expected_first_surf = model.path_placement(1, 1);
        assert_abs_diff_eq!(
            axis.placement.position.x(),
            expected_first_surf.position.x(),
            epsilon = 1e-9
        );
        assert_abs_diff_eq!(
            axis.placement.position.y(),
            expected_first_surf.position.y(),
            epsilon = 1e-9
        );
        assert_abs_diff_eq!(
            axis.placement.position.z(),
            expected_first_surf.position.z(),
            epsilon = 1e-9
        );

        let expected_obj = model.path_placement(1, 0);
        for r in 0..3 {
            for c in 0..3 {
                assert_abs_diff_eq!(
                    axis.placement.rotation_matrix.e[r][c],
                    expected_obj.rotation_matrix.e[r][c],
                    epsilon = 1e-9
                );
            }
        }

        // The fold must be real: this must NOT be an identity rotation.
        assert!(
            (axis.placement.rotation_matrix.e[0][0] - 1.0).abs() > 1e-3
                || (axis.placement.rotation_matrix.e[1][1] - 1.0).abs() > 1e-3
                || (axis.placement.rotation_matrix.e[2][2] - 1.0).abs() > 1e-3,
            "expected a non-identity rotation reflecting path 0's fold, got {:?}",
            axis.placement.rotation_matrix.e
        );
    }

    #[test]
    fn rays_point_source_origin_uses_paths_own_object_frame() {
        let model = folded_object_fixture();
        let placements = model.placements();
        let surface_indices = model.path_surface_indices(1);

        let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 2.0 };
        let field_specs = vec![FieldSpec::PointSource { x: 0.3, y: 0.5 }];
        let paraxial_view =
            ParaxialView::new(&model, &[vec![], field_specs.clone()], false).unwrap();
        let tangential_vec_id =
            paraxial_view.tangential_vec_id_for_phi(1, field_specs[0].tangential_fan_phi());
        let paraxial_subview = paraxial_view
            .get_for_path(1, 0, tangential_vec_id)
            .expect("path 1 subview");

        let generated = rays(
            placements,
            surface_indices,
            &aperture_spec,
            paraxial_subview,
            &field_specs[0],
            PupilSampling::ChiefRay,
        )
        .expect("rays should generate for path 1's own object frame");

        // Independent oracle: the object's own placement directly, not
        // PathAxis's own internal formula restated.
        let obj_placement = &placements[surface_indices[0]];
        let expected_origin =
            (obj_placement.inv_rotation_matrix * Vec3::new(0.3, 0.5, 0.0)) + obj_placement.position;

        assert_abs_diff_eq!(generated[0].x(), expected_origin.x(), epsilon = 1e-9);
        assert_abs_diff_eq!(generated[0].y(), expected_origin.y(), epsilon = 1e-9);
        assert_abs_diff_eq!(generated[0].z(), expected_origin.z(), epsilon = 1e-9);
    }

    #[test]
    fn chief_ray_from_pos_on_axis_stays_on_paths_own_axis() {
        let model = folded_object_fixture();
        let placements = model.placements();
        let surface_indices = model.path_surface_indices(1);

        let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 2.0 };
        let field_specs = vec![FieldSpec::PointSource { x: 0.0, y: 0.0 }];
        let paraxial_view =
            ParaxialView::new(&model, &[vec![], field_specs.clone()], false).unwrap();
        let tangential_vec_id =
            paraxial_view.tangential_vec_id_for_phi(1, field_specs[0].tangential_fan_phi());
        let paraxial_subview = paraxial_view
            .get_for_path(1, 0, tangential_vec_id)
            .expect("path 1 subview");

        let generated = rays(
            placements,
            surface_indices,
            &aperture_spec,
            paraxial_subview,
            &field_specs[0],
            PupilSampling::ChiefRay,
        )
        .expect("on-axis chief ray should generate");

        let axis = PathAxis::resolve(placements, surface_indices).unwrap();
        let mut ray = generated[0].clone();
        ray.transform(&axis.placement);

        assert_abs_diff_eq!(ray.pos().x(), 0.0, epsilon = 1e-9);
        assert_abs_diff_eq!(ray.pos().y(), 0.0, epsilon = 1e-9);
        assert_abs_diff_eq!(ray.dir().x(), 0.0, epsilon = 1e-9);
        assert_abs_diff_eq!(ray.dir().y(), 0.0, epsilon = 1e-9);
        assert_abs_diff_eq!(ray.dir().z(), 1.0, epsilon = 1e-9);
    }

    #[test]
    fn no_ray_terminates_for_folded_object_path() {
        let model = folded_object_fixture();
        let field_specs = vec![FieldSpec::PointSource { x: 0.0, y: 0.5 }];
        let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 2.0 };
        let paraxial_view =
            ParaxialView::new(&model, &[vec![], field_specs.clone()], false).unwrap();
        let config = SamplingConfig {
            n_fan_rays: 5,
            full_pupil_spacing: 0.1,
        };

        let trace = ray_trace_3d_view(
            &[aperture_spec, aperture_spec],
            &[vec![], field_specs],
            &model,
            &paraxial_view,
            config,
        )
        .expect("ray trace should succeed");

        for r in trace.iter().filter(|r| r.path_id() == 1) {
            assert!(
                r.chief_ray_reached_image(),
                "chief ray should reach the image, reason: {:?}",
                r.chief_ray().reason_for_termination()
            );
            assert!(
                r.tangential_fan().terminated().iter().all(|&t| t == 0),
                "no tangential fan ray should terminate: {:?}",
                r.tangential_fan().reason_for_termination()
            );
        }
    }
}
