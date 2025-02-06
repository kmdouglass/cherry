/// A paraxial view into an optical system.
///
/// Paraxial optics is a simplified model of optical systems that assumes that
/// rays are close to the optic axis and that angles are small. Rays are traced
/// through the system using ray transfer matrices, which are 2x2 matrices that
/// describe how rays propagate through and interact with optical surfaces. The
/// paraxial view is used to compute the paraxial parameters of an optical
/// system, such as the entrance and exit pupils, the back and front focal
/// distances, and the effective focal length.
use std::{borrow::Borrow, collections::HashMap};

use anyhow::{anyhow, Result};
use ndarray::{arr2, s, Array, Array1, Array2, Array3, ArrayView2};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        argmin,
        math::vec3::Vec3,
        sequential_model::{
            first_physical_surface, last_physical_surface, reversed_surface_id, Axis,
            SequentialModel, SequentialSubModel, Step, SubModelID, Surface,
        },
        Float,
    },
    specs::surfaces::SurfaceType,
    FieldSpec,
};

const DEFAULT_THICKNESS: Float = 0.0;

/// A 2 x Nr array of paraxial rays.
///
/// Nr is the number of rays. The first column is the height of the ray at the
/// surface, and the second column is the paraxial angle of the ray at the
/// surface.
type ParaxialRays = Array2<Float>;

/// A view into an array of Paraxial rays.
type ParaxialRaysView<'a> = ArrayView2<'a, Float>;

/// A Ns x 2 x Nr array of paraxial ray trace results.
///
/// Ns is the number of surfaces, and Nr is the number of rays. The first
/// element of the 2nd dimension is the height of the ray at the surface, and
/// the second element is the angle of the ray at the surface.
type ParaxialRayTraceResults = Array3<Float>;

/// A 2 x 2 array representing a ray transfer matrix for paraxial rays.
type RayTransferMatrix = Array2<Float>;

/// A paraxial view into an optical system.
///
/// A paraxial view is a set of paraxial subviews that describe the first order
/// properties of an optical system, such as the entrance and exit pupils, the
/// back and front focal distances, and the effective focal length.
///
/// Subviews are indexed by a pair of submodel IDs.
#[derive(Debug)]
pub struct ParaxialView {
    subviews: HashMap<SubModelID, ParaxialSubView>,
    wavelengths: Vec<Float>,
}

/// A description of a paraxial optical system.
///
/// This is used primarily for serialization of data for export.
#[derive(Debug, Serialize)]
pub struct ParaxialViewDescription {
    subviews: HashMap<SubModelID, ParaxialSubViewDescription>,
    axial_primary_color: HashMap<Axis, Float>,
}

/// A paraxial subview of an optical system.
///
/// A paraxial subview is identified by a single submodel ID that corresponds to
/// a submodel of a sequential model. It is not created by the user, but rather
/// by instantiating a new ParaxialView struct.
#[derive(Debug)]
pub struct ParaxialSubView {
    is_obj_space_telecentric: bool,

    aperture_stop: usize,
    back_focal_distance: Float,
    back_principal_plane: Float,
    chief_ray: ParaxialRayTraceResults,
    effective_focal_length: Float,
    entrance_pupil: Pupil,
    exit_pupil: Pupil,
    front_focal_distance: Float,
    front_principal_plane: Float,
    marginal_ray: ParaxialRayTraceResults,
    paraxial_image_plane: ImagePlane,
}

/// A paraxial description of a submodel of an optical system.
///
/// This is used primarily for serialization of data for export.
#[derive(Debug, Serialize)]
pub struct ParaxialSubViewDescription {
    aperture_stop: usize,
    back_focal_distance: Float,
    back_principal_plane: Float,
    chief_ray: ParaxialRayTraceResults,
    effective_focal_length: Float,
    entrance_pupil: Pupil,
    exit_pupil: Pupil,
    front_focal_distance: Float,
    front_principal_plane: Float,
    marginal_ray: ParaxialRayTraceResults,
    paraxial_image_plane: ImagePlane,
}

/// A paraxial entrance or exit pupil.
///
/// # Attributes
/// * `location` - The location of the pupil relative to the first non-object
///   surface.
/// * `semi_diameter` - The semi-diameter of the pupil.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pupil {
    pub location: Float,
    pub semi_diameter: Float,
}

/// A paraxial image plane.
///
/// # Attributes
/// * `location` - The location of the image plane relative to the first
///   physical surface
/// * `semi_diameter` - The semi-diameter of the image plane
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImagePlane {
    pub location: Float,
    pub semi_diameter: Float,
}

/// Propagate paraxial rays a distance along the optic axis.
fn propagate(rays: ParaxialRaysView, distance: Float) -> ParaxialRays {
    let mut propagated = rays.to_owned();
    let mut ray_heights = propagated.row_mut(0);

    ray_heights += &(distance * &rays.row(1));

    propagated
}

/// Compute the z-intercepts of a set of paraxial rays.
///
/// This will return an error if any of the z-intercepts are NaNs.
fn z_intercepts(rays: ParaxialRaysView) -> Result<Array1<Float>> {
    let results = (-&rays.row(0) / rays.row(1)).to_owned();

    if results.iter().any(|&x| x.is_nan()) {
        return Err(anyhow!("Some z_intercepts are NaNs"));
    }

    Ok(results)
}

/// Compute the maximum field angle given a set of field specs.
///
/// The maximum field angle is the maximum absolute value of the paraxial angle.
///
/// # Arguments
/// * `obj_pupil_sepration` - The separation between the object and the entrance
///   pupil.
/// * `field_specs` - The field specs.
///
/// # Returns
/// A tuple containing the maximum field angle and the height of the field.
fn max_field(obj_pupil_sepration: Float, field_specs: &[FieldSpec]) -> (Float, Float) {
    let mut max_angle = 0.0;
    let mut max_height = 0.0;

    for field_spec in field_specs {
        let (height, paraxial_angle) = match field_spec {
            FieldSpec::Angle {
                angle,
                pupil_sampling: _,
            } => {
                let paraxial_angle = angle.to_radians().tan();
                let height = -obj_pupil_sepration * paraxial_angle;
                (height, paraxial_angle)
            }
            FieldSpec::ObjectHeight {
                height,
                pupil_sampling: _,
            } => {
                let paraxial_angle = -height / obj_pupil_sepration;
                (*height, paraxial_angle)
            }
        };

        if paraxial_angle.abs() > max_angle {
            max_angle = paraxial_angle.abs();
            max_height = height;
        }
    }

    (max_angle, max_height)
}

impl ParaxialView {
    /// Creates a new ParaxialView of a SequentialModel.
    ///
    /// # Arguments
    /// * `sequential_model` - The sequential model to create a paraxial view
    ///   of.
    /// * `field_specs` - The field specs of the optical system. These are
    ///   necessary to compute parameters such as the chief ray.
    /// * `is_obj_space_telecentric` - Whether the object space is telecentric.
    ///   This forces the chief ray to be parallel to the optic axis.
    ///
    /// # Returns
    /// A new ParaxialView.
    pub fn new(
        sequential_model: &SequentialModel,
        field_specs: &[FieldSpec],
        is_obj_space_telecentric: bool,
    ) -> Result<Self> {
        let subviews: Result<HashMap<SubModelID, ParaxialSubView>> = sequential_model
            .submodels()
            .iter()
            .map(|(id, submodel)| {
                let surfaces = sequential_model.surfaces();
                let axis = id.1;
                Ok((
                    *id,
                    ParaxialSubView::new(
                        submodel,
                        surfaces,
                        axis,
                        field_specs,
                        is_obj_space_telecentric,
                    )?,
                ))
            })
            .collect();

        Ok(Self {
            subviews: subviews?,
            wavelengths: sequential_model.wavelengths().to_vec(),
        })
    }

    /// Returns a description of the paraxial view.
    ///
    /// This is used primarily for serialization of data for export.
    ///
    /// # Returns
    /// A description of the paraxial view.
    pub fn describe(&self) -> ParaxialViewDescription {
        ParaxialViewDescription {
            subviews: self
                .subviews
                .iter()
                .map(|(id, subview)| (*id, subview.describe()))
                .collect(),
            axial_primary_color: self.axial_primary_color()
        }
    }

    /// Returns the subviews of the paraxial view.
    ///
    /// Each subview corresponds to a submodel of the sequential model.
    ///
    /// # Returns
    /// The subviews of the paraxial view.
    pub fn subviews(&self) -> &HashMap<SubModelID, ParaxialSubView> {
        &self.subviews
    }

    /// Computes the axial primary color aberration of the optical system.
    ///
    /// Here, axial primary color is the difference in focal length
    /// between the maximum and minimum wavelengths. If the traditional
    /// defintion of axial primary color is desired, then the user must
    /// enter the wavelengths for the Fraunhofer F and C lines as minimum and
    /// maximum wavelengths to the underlying sequential model.
    ///
    /// # Returns
    /// A HashMap containing the axial primary color for each axis.
    pub fn axial_primary_color(&self) -> HashMap<Axis, Float> {
        // Find the indexes of the minimum and maximum wavelengths. Return with the
        // empty axial primary color if there are no wavelengths.
        let min_wav_index = self.wavelengths
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or_default();
        let max_wav_index = self.wavelengths
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or_default();

        let mut axial_primary_color: HashMap<Axis, Float> = HashMap::new();
        let mut efls_min_wav: HashMap<SubModelID, Float> = HashMap::new();
        let mut efls_max_wav: HashMap<SubModelID, Float> = HashMap::new();

        for (id, subview) in &self.subviews {
            if id.0 == min_wav_index {
                efls_min_wav.insert(*id, *subview.effective_focal_length());
            } else if id.0 == max_wav_index {
                efls_max_wav.insert(*id, *subview.effective_focal_length());
            }
        }

        // Subtract EFLs that have the same Axis value for their submodel ID. They won't
        // have the same wavelength, so we can't use the same key to access them from
        // the EFLs HashMaps.
        for (id_min, efl_min) in efls_min_wav.iter() {
            for (id_max, efl_max) in efls_max_wav.iter() {
                if id_min.1 == id_max.1 {
                    let apc = (efl_max - efl_min).abs();
                    axial_primary_color.insert(id_min.1, apc);
                }
            }
        }

        axial_primary_color
    }
}

impl ParaxialSubView {
    /// Create a new paraxial view of an optical system.
    fn new(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
        field_specs: &[FieldSpec],
        is_obj_space_telecentric: bool,
    ) -> Result<Self> {
        let pseudo_marginal_ray =
            Self::calc_pseudo_marginal_ray(sequential_sub_model, surfaces, axis)?;
        let parallel_ray = Self::calc_parallel_ray(sequential_sub_model, surfaces, axis)?;
        let reverse_parallel_ray =
            Self::calc_reverse_parallel_ray(sequential_sub_model, surfaces, axis)?;

        let aperture_stop = Self::calc_aperture_stop(surfaces, &pseudo_marginal_ray);
        let back_focal_distance = Self::calc_back_focal_distance(surfaces, &parallel_ray)?;
        let front_focal_distance =
            Self::calc_front_focal_distance(surfaces, &reverse_parallel_ray)?;
        let marginal_ray = Self::calc_marginal_ray(surfaces, &pseudo_marginal_ray, &aperture_stop);
        let entrance_pupil = Self::calc_entrance_pupil(
            sequential_sub_model,
            surfaces,
            is_obj_space_telecentric,
            &aperture_stop,
            &axis,
            &marginal_ray,
        )?;
        let exit_pupil = Self::calc_exit_pupil(
            sequential_sub_model,
            surfaces,
            &aperture_stop,
            &marginal_ray,
        )?;
        let effective_focal_length = Self::calc_effective_focal_length(&parallel_ray);

        let back_principal_plane =
            Self::calc_back_prinicpal_plane(surfaces, back_focal_distance, effective_focal_length)?;
        let front_principal_plane =
            Self::calc_front_principal_plane(front_focal_distance, effective_focal_length);

        let chief_ray: ParaxialRayTraceResults = Self::calc_chief_ray(
            surfaces,
            sequential_sub_model,
            &axis,
            field_specs,
            &entrance_pupil,
        )?;
        let paraxial_image_plane =
            Self::calc_paraxial_image_plane(surfaces, &marginal_ray, &chief_ray)?;

        Ok(Self {
            is_obj_space_telecentric,

            aperture_stop,
            back_focal_distance,
            back_principal_plane,
            chief_ray,
            effective_focal_length,
            entrance_pupil,
            exit_pupil,
            front_focal_distance,
            front_principal_plane,
            marginal_ray,
            paraxial_image_plane,
        })
    }

    fn describe(&self) -> ParaxialSubViewDescription {
        ParaxialSubViewDescription {
            aperture_stop: self.aperture_stop,
            back_focal_distance: self.back_focal_distance,
            back_principal_plane: self.back_principal_plane,
            chief_ray: self.chief_ray.clone(),
            effective_focal_length: self.effective_focal_length,
            entrance_pupil: self.entrance_pupil.clone(),
            exit_pupil: self.exit_pupil.clone(),
            front_focal_distance: self.front_focal_distance,
            front_principal_plane: self.front_principal_plane,
            marginal_ray: self.marginal_ray.clone(),
            paraxial_image_plane: self.paraxial_image_plane.clone(),
        }
    }

    pub fn aperture_stop(&self) -> &usize {
        &self.aperture_stop
    }

    pub fn back_focal_distance(&self) -> &Float {
        &self.back_focal_distance
    }

    pub fn back_principal_plane(&self) -> &Float {
        &self.back_principal_plane
    }

    pub fn chief_ray(&self) -> &ParaxialRayTraceResults {
        &self.chief_ray
    }

    pub fn effective_focal_length(&self) -> &Float {
        &self.effective_focal_length
    }

    pub fn entrance_pupil(&self) -> &Pupil {
        &self.entrance_pupil
    }

    pub fn exit_pupil(&self) -> &Pupil {
        &self.exit_pupil
    }

    pub fn front_focal_distance(&self) -> &Float {
        &self.front_focal_distance
    }

    pub fn front_principal_plane(&self) -> &Float {
        &self.front_principal_plane
    }

    pub fn is_obj_space_telecentric(&self) -> &bool {
        &self.is_obj_space_telecentric
    }

    pub fn marginal_ray(&self) -> &ParaxialRayTraceResults {
        &self.marginal_ray
    }

    pub fn paraxial_image_plane(&self) -> &ImagePlane {
        &self.paraxial_image_plane
    }

    fn calc_aperture_stop(
        surfaces: &[Surface],
        pseudo_marginal_ray: &ParaxialRayTraceResults,
    ) -> usize {
        // Get all the semi-diameters of the surfaces and put them in an ndarray.
        let semi_diameters = Array::from_vec(
            surfaces
                .iter()
                .map(|surface| surface.semi_diameter())
                .collect::<Vec<Float>>(),
        );

        // Absolute value is necessary because the pseudo-marginal ray trace
        // can result in surface intersections that are negative.
        let ratios = (semi_diameters
            / pseudo_marginal_ray[[pseudo_marginal_ray.shape()[0] - 1, 0, 0]])
        .mapv(|x| x.abs());

        // Do not include the object or image surfaces when computing the aperture stop.
        argmin(&ratios.slice(s![1..(ratios.len() - 1)])) + 1
    }

    fn calc_back_focal_distance(
        surfaces: &[Surface],
        parallel_ray: &ParaxialRayTraceResults,
    ) -> Result<Float> {
        let last_physical_surface_index =
            last_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        let z_intercepts =
            z_intercepts(parallel_ray.slice(s![last_physical_surface_index, .., ..]))?;

        let bfd = z_intercepts[0];

        // Handle edge case for infinite BFD
        if bfd.is_infinite() {
            return Ok(Float::INFINITY);
        }

        Ok(bfd)
    }

    fn calc_back_prinicpal_plane(
        surfaces: &[Surface],
        back_focal_distance: Float,
        effective_focal_length: Float,
    ) -> Result<Float> {
        let delta = back_focal_distance - effective_focal_length;

        // Principal planes make no sense for lenses without power
        if delta.is_infinite() {
            return Ok(Float::NAN);
        }

        // Find the z position of the last real surface
        let last_physical_surface_index =
            last_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        let last_surface_z = surfaces[last_physical_surface_index].z();

        Ok(last_surface_z + delta)
    }

    /// Computes the paraxial chief ray for a given field.
    fn calc_chief_ray(
        surfaces: &[Surface],
        sequential_sub_model: &impl SequentialSubModel,
        axis: &Axis,
        field_specs: &[FieldSpec],
        entrance_pupil: &Pupil,
    ) -> Result<ParaxialRayTraceResults> {
        let enp_loc = entrance_pupil.location;
        let obj_loc = surfaces.first().ok_or(anyhow!("No surfaces provided"))?.z();
        let sep = if obj_loc.is_infinite() {
            0.0
        } else {
            enp_loc - obj_loc
        };

        let (paraxial_angle, height) = max_field(sep, field_specs);

        if paraxial_angle.is_infinite() {
            return Err(anyhow!(
                "Cannot compute chief ray from an infinite field angle"
            ));
        }

        let initial_ray: ParaxialRays = arr2(&[[height], [paraxial_angle]]);
        Self::trace(initial_ray, sequential_sub_model, surfaces, axis, false)
    }

    fn calc_effective_focal_length(parallel_ray: &ParaxialRayTraceResults) -> Float {
        let y_1 = parallel_ray.slice(s![1, 0, 0]);
        let u_final = parallel_ray.slice(s![-2, 1, 0]);
        let efl = -y_1.into_scalar() / u_final.into_scalar();

        // Handle edge case for negatively infinite EFL
        if efl.is_infinite() {
            return Float::INFINITY;
        }

        efl
    }

    fn calc_entrance_pupil(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        is_obj_space_telecentric: bool,
        aperture_stop: &usize,
        axis: &Axis,
        marginal_ray: &ParaxialRayTraceResults,
    ) -> Result<Pupil> {
        // In case the object space is telecentric, the entrance pupil is at infinity.
        if is_obj_space_telecentric {
            return Ok(Pupil {
                location: Float::INFINITY,
                semi_diameter: Float::NAN,
            });
        }

        // In case the aperture stop is the first surface.
        if *aperture_stop == 1usize {
            return Ok(Pupil {
                location: 0.0,
                semi_diameter: surfaces[1].semi_diameter(),
            });
        }

        // Trace a ray from the aperture stop to the object space to determine the
        // entrance pupil location.
        let ray = arr2(&[[0.0], [1.0]]);
        let results = Self::trace(
            ray,
            &sequential_sub_model.slice(0..*aperture_stop),
            &surfaces[0..aperture_stop + 1],
            axis,
            true,
        )?;
        let location = z_intercepts(results.slice(s![-1, .., ..]))?[0];

        // Propagate the marginal ray to the entrance pupil location to determine its
        // semi-diameter. I'm not sure, but I think the [0, .., ..1] slice on
        // the marginal ray is required by the compiler because otherwise the
        // dimensionality of the slice becomes incompatible with the argument of the
        // propagate function, i.e. the slice [0, .., 0] has the wrong dimensions.
        let distance = if sequential_sub_model.is_obj_at_inf() {
            location
        } else {
            sequential_sub_model
                .gaps()
                .first()
                .expect("A submodel should always have at least one gap.")
                .thickness
                + location
        };
        let init_marginal_ray = marginal_ray.slice(s![0, .., ..1]);
        let semi_diameter = propagate(init_marginal_ray, distance)[[0, 0]];

        Ok(Pupil {
            location,
            semi_diameter,
        })
    }

    fn calc_exit_pupil(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        aperture_stop: &usize,
        marginal_ray: &ParaxialRayTraceResults,
    ) -> Result<Pupil> {
        let last_physical_surface_id =
            last_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        if last_physical_surface_id == *aperture_stop {
            return Ok(Pupil {
                location: surfaces[last_physical_surface_id].z(),
                semi_diameter: surfaces[last_physical_surface_id].semi_diameter(),
            });
        }

        // Trace a ray through the aperture stop forwards through the system
        let ray = arr2(&[[0.0], [1.0]]);

        let results = Self::trace(
            ray,
            &sequential_sub_model.slice(*aperture_stop..sequential_sub_model.len()),
            &surfaces[*aperture_stop..],
            &Axis::Y,
            false,
        )?;

        // Distance is relative to the last physical surface
        let sliced_last_physical_surface_id = last_physical_surface_id - aperture_stop;
        let distance = z_intercepts(results.slice(s![sliced_last_physical_surface_id, .., ..]))?[0];
        let last_physical_surface = surfaces[last_physical_surface_id].borrow();
        let location = last_physical_surface.z() + distance;

        // Propagate the marginal ray to the exit pupil location and find its height
        let semi_diameter = propagate(
            marginal_ray.slice(s![last_physical_surface_id, .., ..]),
            distance,
        )[[0, 0]];

        Ok(Pupil {
            location,
            semi_diameter,
        })
    }

    fn calc_front_focal_distance(
        surfaces: &[Surface],
        reverse_parallel_ray: &ParaxialRayTraceResults,
    ) -> Result<Float> {
        let first_physical_surface_index =
            first_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        let index = reversed_surface_id(surfaces, first_physical_surface_index);
        let z_intercepts = z_intercepts(reverse_parallel_ray.slice(s![index, .., ..]))?;

        let ffd = z_intercepts[0];

        // Handle edge case for infinite FFD
        if ffd.is_infinite() {
            return Ok(Float::INFINITY);
        }

        Ok(ffd)
    }

    fn calc_front_principal_plane(
        front_focal_distance: Float,
        effective_focal_length: Float,
    ) -> Float {
        // Principal planes make no sense for lenses without power
        if front_focal_distance.is_infinite() {
            return Float::NAN;
        }

        front_focal_distance + effective_focal_length
    }

    fn calc_marginal_ray(
        surfaces: &[Surface],
        pseudo_marginal_ray: &ParaxialRayTraceResults,
        aperture_stop: &usize,
    ) -> ParaxialRayTraceResults {
        let semi_diameters = Array::from_vec(
            surfaces
                .iter()
                .map(|surface| surface.semi_diameter())
                .collect::<Vec<Float>>(),
        );
        let ratios = semi_diameters / pseudo_marginal_ray.slice(s![.., 0, 0]);

        let scale_factor = ratios[*aperture_stop];

        pseudo_marginal_ray * scale_factor
    }

    /// Compute the parallel ray.
    fn calc_parallel_ray(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
    ) -> Result<ParaxialRayTraceResults> {
        let ray = arr2(&[[1.0], [0.0]]);

        Self::trace(ray, sequential_sub_model, surfaces, &axis, false)
    }

    /// Compute the paraxial image plane.
    fn calc_paraxial_image_plane(
        surfaces: &[Surface],
        marginal_ray: &ParaxialRayTraceResults,
        chief_ray: &ParaxialRayTraceResults,
    ) -> Result<ImagePlane> {
        let last_physical_surface_id =
            last_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        let last_surface = surfaces[last_physical_surface_id].borrow();

        let dz = z_intercepts(marginal_ray.slice(s![last_physical_surface_id, .., ..]))?[0];
        let location = if dz.is_infinite() {
            // Ensure positive infinity is returned for infinite image planes
            Float::INFINITY
        } else {
            last_surface.z() + dz
        };

        // Propagate the chief ray from the last physical surface to the image plane to
        // determine its semi-diameter.
        let ray = chief_ray.slice(s![last_physical_surface_id, .., ..]);
        let propagated = propagate(ray, dz);
        let semi_diameter = propagated[[0, 0]].abs();

        Ok(ImagePlane {
            location,
            semi_diameter,
        })
    }

    /// Compute the pseudo-marginal ray.
    fn calc_pseudo_marginal_ray(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
    ) -> Result<ParaxialRayTraceResults> {
        let ray = if sequential_sub_model.is_obj_at_inf() {
            // Ray parallel to axis at a height of 1
            arr2(&[[1.0], [0.0]])
        } else {
            // Ray starting from the axis at an angle of 1
            arr2(&[[0.0], [1.0]])
        };

        Self::trace(ray, sequential_sub_model, surfaces, &axis, false)
    }

    /// Compute the reverse parallel ray.
    fn calc_reverse_parallel_ray(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
    ) -> Result<ParaxialRayTraceResults> {
        let ray = arr2(&[[1.0], [0.0]]);

        Self::trace(ray, sequential_sub_model, surfaces, &axis, true)
    }

    /// Compute the ray transfer matrix for each gap/surface pair.
    fn rtms(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        axis: &Axis,
        reverse: bool,
    ) -> Result<Vec<RayTransferMatrix>> {
        let mut txs: Vec<RayTransferMatrix> = Vec::new();
        let mut forward_iter;
        let mut reverse_iter;
        let steps: &mut dyn Iterator<Item = Step> = if reverse {
            reverse_iter = sequential_sub_model.try_iter(surfaces)?.try_reverse()?;
            &mut reverse_iter
        } else {
            forward_iter = sequential_sub_model.try_iter(surfaces)?;
            &mut forward_iter
        };

        for (gap_0, surface, gap_1) in steps {
            let t = if gap_0.thickness.is_infinite() {
                DEFAULT_THICKNESS
            } else if reverse {
                // Reverse ray tracing is implemented as negative distances to avoid hassles
                // with inverses of ray transfer matrices.
                -gap_0.thickness
            } else {
                gap_0.thickness
            };

            let roc = surface.roc(axis);

            let n_0 = gap_0.refractive_index.n();

            let n_1 = if let Some(gap_1) = gap_1 {
                gap_1.refractive_index.n()
            } else {
                gap_0.refractive_index.n()
            };

            let rtm = surface_to_rtm(surface, t, roc, n_0, n_1);
            txs.push(rtm);
        }

        Ok(txs)
    }

    fn trace(
        rays: ParaxialRays,
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        axis: &Axis,
        reverse: bool,
    ) -> Result<ParaxialRayTraceResults> {
        let txs = Self::rtms(sequential_sub_model, surfaces, axis, reverse)?;

        // Initialize the results array by assigning the input rays to the first
        // surface.
        let mut results = Array3::zeros((txs.len() + 1, 2, rays.shape()[1]));
        results.slice_mut(s![0, .., ..]).assign(&rays);

        // Iterate over the surfaces and compute the ray trace results.
        for (i, tx) in txs.iter().enumerate() {
            let rays = results.slice(s![i, .., ..]);
            let rays = tx.dot(&rays);
            results.slice_mut(s![i + 1, .., ..]).assign(&rays);
        }

        Ok(results)
    }
}

impl Pupil {
    pub fn pos(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, self.location)
    }
}

/// Compute the ray transfer matrix for propagation to and interaction with a
/// surface.
fn surface_to_rtm(
    surface: &Surface,
    t: Float,
    roc: Float,
    n_0: Float,
    n_1: Float,
) -> RayTransferMatrix {
    let surface_type = surface.surface_type();

    match surface {
        // Conics and torics behave the same in paraxial subviews.
        //Surface::Conic(_) | Surface::Toric(_) => match surface_type {
        Surface::Conic(_) => match surface_type {
            SurfaceType::Refracting => arr2(&[
                [1.0, t],
                [
                    (n_0 - n_1) / n_1 / roc,
                    t * (n_0 - n_1) / n_1 / roc + n_0 / n_1,
                ],
            ]),
            SurfaceType::Reflecting => arr2(&[[1.0, t], [-2.0 / roc, 1.0 - 2.0 * t / roc]]),
            SurfaceType::NoOp => panic!("Conics and torics cannot be NoOp surfaces."),
        },
        Surface::Image(_) | Surface::Probe(_) | Surface::Stop(_) => arr2(&[[1.0, t], [0.0, 1.0]]),
        Surface::Object(_) => arr2(&[[1.0, 0.0], [0.0, 1.0]]),
    }
}

// Consider moving these to integration tests once the paraxial view and
// sequential models are combined into a system.
#[cfg(test)]
mod test {
    use approx::assert_abs_diff_eq;
    use ndarray::{arr1, arr3};

    use crate::core::sequential_model::SubModelID;
    use crate::examples::convexplano_lens;

    use super::*;

    #[test]
    fn test_propagate() {
        let rays = arr2(&[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        let propagated = propagate(rays.view(), 2.0);

        let expected = arr2(&[[9.0, 12.0, 15.0], [4.0, 5.0, 6.0]]);

        assert_abs_diff_eq!(propagated, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_z_intercepts() {
        let rays = arr2(&[[1.0, 2.0, 3.0, 0.0], [4.0, 5.0, 6.0, 7.0]]);
        let z_intercepts = z_intercepts(rays.view()).unwrap();

        let expected = arr1(&[-0.25, -0.4, -0.5, 0.0]);

        assert_abs_diff_eq!(z_intercepts, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_z_intercepts_divide_by_zero() {
        let rays = arr2(&[[1.0], [0.0]]);
        let z_intercepts = z_intercepts(rays.view()).unwrap();

        assert!(z_intercepts.shape() == [1]);
        assert!(z_intercepts[0].is_infinite());
    }

    #[test]
    fn test_z_intercepts_zero_height_divide_by_zero() {
        let rays = arr2(&[[0.0], [0.0]]);
        let z_intercepts = z_intercepts(rays.view());

        assert!(z_intercepts.is_err());
    }

    fn setup() -> (ParaxialSubView, SequentialModel) {
        let sequential_model = convexplano_lens::sequential_model();
        let seq_sub_model = sequential_model
            .submodels()
            .get(&SubModelID(0usize, Axis::Y))
            .expect("Submodel not found.");
        let field_specs = vec![
            FieldSpec::Angle {
                angle: 0.0,
                pupil_sampling: crate::PupilSampling::SquareGrid { spacing: 0.1 },
            },
            FieldSpec::Angle {
                angle: 5.0,
                pupil_sampling: crate::PupilSampling::SquareGrid { spacing: 0.1 },
            },
        ];

        (
            ParaxialSubView::new(
                seq_sub_model,
                sequential_model.surfaces(),
                Axis::Y,
                &field_specs,
                false,
            )
            .unwrap(),
            sequential_model,
        )
    }

    #[test]
    fn test_aperture_stop() {
        let (view, _) = setup();

        let aperture_stop = view.aperture_stop();
        let expected = 1;

        assert_eq!(*aperture_stop, expected);
    }

    #[test]
    fn test_entrance_pupil() {
        let (view, _) = setup();

        let entrance_pupil = view.entrance_pupil();
        let expected = Pupil {
            location: 0.0,
            semi_diameter: 12.5,
        };

        assert_abs_diff_eq!(entrance_pupil.location, expected.location, epsilon = 1e-4);
        assert_abs_diff_eq!(
            entrance_pupil.semi_diameter,
            expected.semi_diameter,
            epsilon = 1e-4
        );
    }

    #[test]
    fn test_marginal_ray() {
        let (view, _) = setup();

        let marginal_ray = view.marginal_ray();
        let expected = arr3(&[
            [[12.5000], [0.0]],
            [[12.5000], [-0.1647]],
            [[11.6271], [-0.2495]],
            [[-0.0003], [-0.2495]],
        ]);

        assert_abs_diff_eq!(*marginal_ray, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_pseudo_marginal_ray() {
        let sequential_model = convexplano_lens::sequential_model();
        let seq_sub_model = sequential_model
            .submodels()
            .get(&SubModelID(0usize, Axis::Y))
            .expect("Submodel not found.");
        let pseudo_marginal_ray = ParaxialSubView::calc_pseudo_marginal_ray(
            seq_sub_model,
            sequential_model.surfaces(),
            Axis::Y,
        )
        .unwrap();

        let expected = arr3(&[
            [[1.0000], [0.0]],
            [[1.0000], [-0.0132]],
            [[0.9302], [-0.0200]],
            [[0.0], [-0.0200]],
        ]);

        assert_abs_diff_eq!(pseudo_marginal_ray, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_reverse_parallel_ray() {
        let sequential_model = convexplano_lens::sequential_model();
        let seq_sub_model = sequential_model
            .submodels()
            .get(&SubModelID(0usize, Axis::Y))
            .expect("Submodel not found.");
        let reverse_parallel_ray = ParaxialSubView::calc_reverse_parallel_ray(
            seq_sub_model,
            sequential_model.surfaces(),
            Axis::Y,
        )
        .unwrap();

        let expected = arr3(&[[[1.0000], [0.0]], [[1.0000], [0.0]], [[1.0000], [0.0200]]]);

        assert_abs_diff_eq!(reverse_parallel_ray, expected, epsilon = 1e-4);
    }
}
