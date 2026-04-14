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

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    FieldSpec,
    core::{
        Float,
        math::{linalg::mat2x2::Mat2x2, vec3::Vec3},
        sequential_model::{
            SequentialModel, SequentialSubModel, Step, Surface, first_physical_surface,
            last_physical_surface, propagate_tangential_vec, reversed_surface_id,
        },
    },
    specs::{fields::unique_tangential_vecs, surfaces::SurfaceType},
};

/// A unique identifier for a paraxial submodel.
///
/// The first element is the index of the wavelength in the system's list of
/// wavelengths. The second element is the index into `ParaxialView`'s
/// tangential-vector table, which is built from the field specs at view
/// construction time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubModelID(pub usize, pub usize);

impl Serialize for SubModelID {
    // Serialize as a string like "0:0" because tuple keys are awkward in JSON.
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.0, self.1))
    }
}

const DEFAULT_THICKNESS: Float = 0.0;

/// A unit vector in the global frame that defines a meridional (tangential)
/// plane. It lies in the transverse R–U plane (z-component = 0 at the object)
/// and is propagated through fold mirrors via the vector law of reflection.
///
/// For phi=0°: `(1, 0, 0)` (cursor-R / global X).
/// For phi=90°: `(0, 1, 0)` (cursor-U / global Y).
type TangentialVector = Vec3;

/// A single paraxial ray with height and angle components.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ParaxialRay {
    pub height: Float,
    pub angle: Float,
}

/// A set of paraxial rays traced through all surfaces of an optical system.
///
/// Rays are stored in a flat [`Vec`] with a fixed number of rays per surface,
/// laid out as `[surf_0_ray_0, …, surf_0_ray_N, surf_1_ray_0, …]`. The number
/// of rays per surface is `rays.len() / num_surfaces`. Access rays at a given
/// surface with [`ParaxialRayBundle::rays_at_surface`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParaxialRayBundle {
    rays: Vec<ParaxialRay>,
    num_surfaces: usize,
}

impl ParaxialRayBundle {
    /// Returns the rays at the given surface index.
    pub fn rays_at_surface(&self, surface_id: usize) -> &[ParaxialRay] {
        let num_rays = self.rays.len() / self.num_surfaces;
        let start = surface_id * num_rays;
        &self.rays[start..start + num_rays]
    }

    /// Returns the number of surfaces in the bundle.
    pub fn num_surfaces(&self) -> usize {
        self.num_surfaces
    }

    /// Returns an iterator over the rays at each surface.
    pub fn iter_surfaces(&self) -> impl Iterator<Item = &[ParaxialRay]> + '_ {
        (0..self.num_surfaces).map(move |i| self.rays_at_surface(i))
    }

    /// Returns the rays at the last surface, or `None` if there are no
    /// surfaces.
    pub fn last_surface(&self) -> Option<&[ParaxialRay]> {
        if self.num_surfaces == 0 {
            None
        } else {
            Some(self.rays_at_surface(self.num_surfaces - 1))
        }
    }
}

/// A 2 x 2 ray transfer matrix for paraxial rays.
type RayTransferMatrix = Mat2x2;

/// A paraxial view into an optical system.
///
/// A paraxial view is a set of paraxial subviews that describe the first order
/// properties of an optical system, such as the entrance and exit pupils, the
/// back and front focal distances, and the effective focal length.
///
/// Subviews are indexed by `SubModelID(wavelength_idx, v_index)` where
/// `v_index` refers to an entry in `tangential_vecs`.
#[derive(Debug)]
pub struct ParaxialView {
    tangential_vecs: Vec<TangentialVector>,
    subviews: HashMap<SubModelID, ParaxialSubView>,
    wavelengths: Vec<Float>,
}

/// A description of a paraxial optical system.
///
/// This is used primarily for serialization of data for export.
#[derive(Debug, Serialize)]
pub struct ParaxialViewDescription {
    subviews: HashMap<SubModelID, ParaxialSubViewDescription>,
    /// Keyed by v_index (index into the tangential-vector table).
    primary_axial_color: HashMap<usize, Float>,
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
    chief_ray: ParaxialRayBundle,
    effective_focal_length: Float,
    entrance_pupil: Pupil,
    exit_pupil: Pupil,
    front_focal_distance: Float,
    front_principal_plane: Float,
    marginal_ray: ParaxialRayBundle,
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
    chief_ray: ParaxialRayBundle,
    effective_focal_length: Float,
    entrance_pupil: Pupil,
    exit_pupil: Pupil,
    front_focal_distance: Float,
    front_principal_plane: Float,
    marginal_ray: ParaxialRayBundle,
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
fn propagate(rays: &[ParaxialRay], distance: Float) -> Vec<ParaxialRay> {
    rays.iter()
        .map(|r| ParaxialRay {
            height: r.height + distance * r.angle,
            angle: r.angle,
        })
        .collect()
}

/// Compute the axis-intercepts of a set of paraxial rays.
///
/// The axis-intercept is the signed distance along the local propagation axis
/// from the current surface to where the ray crosses that axis. In an unfolded
/// system this coincides with the z-axis; in a folded system it is the local
/// optical axis at each segment.
///
/// This will return an error if any of the intercepts are NaNs.
fn axis_intercepts(rays: &[ParaxialRay]) -> Result<Vec<Float>> {
    let results: Vec<Float> = rays.iter().map(|r| -r.height / r.angle).collect();

    if results.iter().any(|&x| x.is_nan()) {
        return Err(anyhow!("Some axis_intercepts are NaNs"));
    }

    Ok(results)
}

/// Compute the maximum field angle given a set of field specs.
///
/// The maximum field angle is the maximum absolute value of the paraxial angle.
///
/// # Arguments
/// * `obj_pupil_separation` - The separation between the object and the
///   entrance pupil.
/// * `field_specs` - The field specs.
///
/// # Returns
/// A tuple containing the maximum field angle and the height of the field.
fn max_field(obj_pupil_separation: Float, field_specs: &[FieldSpec]) -> (Float, Float) {
    let mut max_angle = 0.0;
    let mut max_height = 0.0;

    for field_spec in field_specs {
        let (height, paraxial_angle) = match field_spec {
            FieldSpec::Angle { chi, phi: _ } => {
                let paraxial_angle = chi.to_radians().tan();
                let height = -obj_pupil_separation * paraxial_angle;
                (height, paraxial_angle)
            }
            FieldSpec::PointSource { x, y } => {
                let height = (x.powi(2) + y.powi(2)).sqrt();
                let paraxial_angle = -height / obj_pupil_separation;
                (height, paraxial_angle)
            }
        };

        if paraxial_angle.abs() > max_angle {
            max_angle = paraxial_angle;
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
        let surfaces = sequential_model.surfaces();
        let tangential_vecs: Vec<TangentialVector> =
            if SequentialModel::is_rotationally_symmetric(surfaces) {
                vec![Vec3::new(0.0, 1.0, 0.0)]
            } else {
                unique_tangential_vecs(field_specs)
            };

        let mut subviews = HashMap::new();
        for (&wav_idx, submodel) in sequential_model.submodels() {
            for (v_idx, &v) in tangential_vecs.iter().enumerate() {
                let id = SubModelID(wav_idx, v_idx);
                let subview = ParaxialSubView::new(
                    submodel,
                    surfaces,
                    v,
                    field_specs,
                    is_obj_space_telecentric,
                )?;
                subviews.insert(id, subview);
            }
        }

        Ok(Self {
            tangential_vecs,
            subviews,
            wavelengths: sequential_model.wavelengths().to_vec(),
        })
    }

    /// Returns a description of the paraxial view.
    ///
    /// This is used primarily for serialization of data for export.
    pub fn describe(&self) -> ParaxialViewDescription {
        ParaxialViewDescription {
            subviews: self
                .subviews
                .iter()
                .map(|(id, subview)| (*id, subview.describe()))
                .collect(),
            primary_axial_color: self.primary_axial_color(),
        }
    }

    /// Returns the subviews of the paraxial view.
    pub fn subviews(&self) -> &HashMap<SubModelID, ParaxialSubView> {
        &self.subviews
    }

    /// Returns the tangential direction vector for a given v_index.
    pub fn tangential_vec(&self, v_index: usize) -> TangentialVector {
        self.tangential_vecs[v_index]
    }

    /// Returns the azimuthal angle in degrees for a given v_index.
    pub fn phi_deg(&self, v_index: usize) -> Float {
        let v = self.tangential_vecs[v_index];
        v.y().atan2(v.x()).to_degrees()
    }

    /// Returns the v_index whose tangential vector is closest (by dot product)
    /// to the given azimuthal angle in radians.
    ///
    /// For the common case where `phi_rad` exactly matches a stored phi key
    /// (bit-identical `tangential_fan_phi()` value), this finds the exact
    /// entry. Falls back to index 0 if the table is empty.
    pub fn v_index_for_phi(&self, phi_rad: Float) -> usize {
        let target: TangentialVector = Vec3::new(phi_rad.cos(), phi_rad.sin(), 0.0);
        self.tangential_vecs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let da = a.x() * target.x() + a.y() * target.y();
                let db = b.x() * target.x() + b.y() * target.y();
                da.total_cmp(&db)
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Computes the primary axial color aberration of the optical system.
    ///
    /// Primary axial color is the absolute difference in EFL between the
    /// maximum and minimum wavelengths, reported per tangential-vector index.
    pub fn primary_axial_color(&self) -> HashMap<usize, Float> {
        let min_wav_index = self
            .wavelengths
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or_default();
        let max_wav_index = self
            .wavelengths
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or_default();

        let mut efls_min_wav: HashMap<SubModelID, Float> = HashMap::new();
        let mut efls_max_wav: HashMap<SubModelID, Float> = HashMap::new();

        for (id, subview) in &self.subviews {
            if id.0 == min_wav_index {
                efls_min_wav.insert(*id, *subview.effective_focal_length());
            } else if id.0 == max_wav_index {
                efls_max_wav.insert(*id, *subview.effective_focal_length());
            }
        }

        // Pair entries that share the same v_index.
        let mut primary_axial_color: HashMap<usize, Float> = HashMap::new();
        for (id_min, efl_min) in &efls_min_wav {
            for (id_max, efl_max) in &efls_max_wav {
                if id_min.1 == id_max.1 {
                    primary_axial_color.insert(id_min.1, (efl_max - efl_min).abs());
                }
            }
        }

        primary_axial_color
    }
}

impl ParaxialSubView {
    /// Create a new paraxial subview for the given tangential direction.
    ///
    /// `v` is a `TangentialVector` in the global frame defining the meridional
    /// plane (e.g. `(0,1,0)` for phi=90°). It is propagated through mirror
    /// surfaces internally to compute per-surface foreshortening.
    fn new(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        v: TangentialVector,
        field_specs: &[FieldSpec],
        is_obj_space_telecentric: bool,
    ) -> Result<Self> {
        // Propagate v through mirror surfaces to get per-surface tangential vectors.
        let per_surf_v: Vec<TangentialVector> = propagate_tangential_vec(v, surfaces);

        let pseudo_marginal_ray = Self::calc_pseudo_marginal_ray(sequential_sub_model, surfaces)?;
        let parallel_ray = Self::calc_parallel_ray(sequential_sub_model, surfaces)?;
        let reverse_parallel_ray = Self::calc_reverse_parallel_ray(sequential_sub_model, surfaces)?;

        let aperture_stop = Self::calc_aperture_stop(surfaces, &pseudo_marginal_ray, &per_surf_v);
        let back_focal_distance = Self::calc_back_focal_distance(surfaces, &parallel_ray)?;
        let front_focal_distance =
            Self::calc_front_focal_distance(surfaces, &reverse_parallel_ray)?;
        let marginal_ray =
            Self::calc_marginal_ray(surfaces, &pseudo_marginal_ray, &aperture_stop, &per_surf_v);
        let entrance_pupil = Self::calc_entrance_pupil(
            sequential_sub_model,
            surfaces,
            is_obj_space_telecentric,
            &aperture_stop,
            &per_surf_v,
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
            Self::calc_back_principal_plane(back_focal_distance, effective_focal_length)?;
        let front_principal_plane =
            Self::calc_front_principal_plane(front_focal_distance, effective_focal_length);

        let chief_ray = Self::calc_chief_ray(
            surfaces,
            sequential_sub_model,
            v,
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

    pub fn chief_ray(&self) -> &ParaxialRayBundle {
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

    pub fn marginal_ray(&self) -> &ParaxialRayBundle {
        &self.marginal_ray
    }

    pub fn paraxial_image_plane(&self) -> &ImagePlane {
        &self.paraxial_image_plane
    }

    fn calc_aperture_stop(
        surfaces: &[Surface],
        pseudo_marginal_ray: &ParaxialRayBundle,
        per_surf_v: &[TangentialVector],
    ) -> usize {
        // Get all the projected semi-diameters of the surfaces. For tilted surfaces,
        // projected_semi_diameter accounts for the foreshortening of the clear aperture
        // as seen by a paraxial ray traveling along the cursor axis.
        //
        // Absolute value is necessary because the pseudo-marginal ray trace can result
        // in surface intersections that are negative.
        let last_surface_height = pseudo_marginal_ray.last_surface().unwrap()[0].height;
        let ratios: Vec<Float> = surfaces
            .iter()
            .zip(per_surf_v.iter())
            .map(|(s, &v)| (s.projected_semi_diameter(v) / last_surface_height).abs())
            .collect();

        // Do not include the object or image surfaces when computing the aperture stop.
        argmin(&ratios[1..ratios.len() - 1]) + 1
    }

    fn calc_back_focal_distance(
        surfaces: &[Surface],
        parallel_ray: &ParaxialRayBundle,
    ) -> Result<Float> {
        let last_physical_surface_index =
            last_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        let intercepts =
            axis_intercepts(parallel_ray.rays_at_surface(last_physical_surface_index))?;

        let bfd = intercepts[0];

        // Handle edge case for infinite BFD
        if bfd.is_infinite() {
            return Ok(Float::INFINITY);
        }

        // Distance is always positive
        Ok(bfd.abs())
    }

    fn calc_back_principal_plane(
        back_focal_distance: Float,
        effective_focal_length: Float,
    ) -> Result<Float> {
        let delta = back_focal_distance - effective_focal_length;

        // Principal planes make no sense for lenses without power
        if delta.is_infinite() {
            return Ok(Float::NAN);
        }

        // Return a signed distance from the last physical surface along the beam
        // path, matching the convention of front_principal_plane (distance from the
        // first physical surface).
        Ok(delta)
    }

    /// Computes the paraxial chief ray for a given tangential direction.
    ///
    /// Only field specs whose phi angle matches `v` are used. This ensures each
    /// submodel's chief ray is computed from the fields that lie in its
    /// meridional plane.
    fn calc_chief_ray(
        surfaces: &[Surface],
        sequential_sub_model: &impl SequentialSubModel,
        v: TangentialVector,
        field_specs: &[FieldSpec],
        entrance_pupil: &Pupil,
    ) -> Result<ParaxialRayBundle> {
        let enp_loc = entrance_pupil.location;
        let obj_loc = surfaces
            .first()
            .ok_or(anyhow!("No surfaces provided"))?
            .track();
        let sep = if obj_loc.is_infinite() {
            0.0
        } else {
            enp_loc - obj_loc
        };

        // Filter to only the field specs whose phi matches this submodel's v.
        let v_phi = v.y().atan2(v.x());
        let matching: Vec<FieldSpec> = field_specs
            .iter()
            .copied()
            .filter(|f| (f.tangential_fan_phi() - v_phi).abs() < 1e-9)
            .collect();

        let (paraxial_angle, height) = max_field(sep, &matching);

        if paraxial_angle.is_infinite() {
            return Err(anyhow!(
                "Cannot compute chief ray from an infinite field angle"
            ));
        }

        let initial_ray = vec![ParaxialRay {
            height,
            angle: paraxial_angle,
        }];
        Self::trace(initial_ray, sequential_sub_model, surfaces, false)
    }

    fn calc_effective_focal_length(parallel_ray: &ParaxialRayBundle) -> Float {
        let y_1 = parallel_ray.rays_at_surface(1)[0].height;
        let u_final = parallel_ray.rays_at_surface(parallel_ray.num_surfaces() - 2)[0].angle;

        // There should be a negative sign here for lens only systems, but we take abs
        // later so it's not needed
        let efl = y_1 / u_final;

        // Handle edge case for negatively infinite EFL
        if efl.is_infinite() {
            return Float::INFINITY;
        }

        // abs() handles edge case of apparent negative EFLs in reflecting systems
        efl.abs()
    }

    fn calc_entrance_pupil(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        is_obj_space_telecentric: bool,
        aperture_stop: &usize,
        per_surf_v: &[TangentialVector],
        marginal_ray: &ParaxialRayBundle,
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
                semi_diameter: surfaces[1].projected_semi_diameter(per_surf_v[1]),
            });
        }

        // Trace a ray from the aperture stop to the object space to determine the
        // entrance pupil location.
        let ray = vec![ParaxialRay {
            height: 0.0,
            angle: 1.0,
        }];
        let results = Self::trace(
            ray,
            &sequential_sub_model.slice(0..*aperture_stop),
            &surfaces[0..aperture_stop + 1],
            true,
        )?;
        let location = axis_intercepts(results.last_surface().unwrap())?[0];

        // Propagate the marginal ray to the entrance pupil location to determine its
        // semi-diameter.
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
        let semi_diameter = propagate(&marginal_ray.rays_at_surface(0)[..1], distance)[0].height;

        Ok(Pupil {
            location,
            semi_diameter,
        })
    }

    fn calc_exit_pupil(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        aperture_stop: &usize,
        marginal_ray: &ParaxialRayBundle,
    ) -> Result<Pupil> {
        let last_physical_surface_id =
            last_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        if last_physical_surface_id == *aperture_stop {
            return Ok(Pupil {
                location: 0.0,
                semi_diameter: surfaces[last_physical_surface_id].semi_diameter(),
            });
        }

        // Trace a ray through the aperture stop forwards through the system
        let ray = vec![ParaxialRay {
            height: 0.0,
            angle: 1.0,
        }];

        let results = Self::trace(
            ray,
            &sequential_sub_model.slice(*aperture_stop..sequential_sub_model.len()),
            &surfaces[*aperture_stop..],
            false,
        )?;

        // Distance is relative to the last physical surface
        let sliced_last_physical_surface_id = last_physical_surface_id - aperture_stop;
        let distance =
            axis_intercepts(results.rays_at_surface(sliced_last_physical_surface_id))?[0];

        // Propagate the marginal ray to the exit pupil location and find its height
        let semi_diameter = propagate(
            marginal_ray.rays_at_surface(last_physical_surface_id),
            distance,
        )[0]
        .height;

        Ok(Pupil {
            location: distance,
            semi_diameter,
        })
    }

    fn calc_front_focal_distance(
        surfaces: &[Surface],
        reverse_parallel_ray: &ParaxialRayBundle,
    ) -> Result<Float> {
        let first_physical_surface_index =
            first_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        let index = reversed_surface_id(surfaces, first_physical_surface_index);
        let intercepts = axis_intercepts(reverse_parallel_ray.rays_at_surface(index))?;

        let ffd = intercepts[0];

        // Handle edge case for infinite FFD
        if ffd.is_infinite() {
            return Ok(Float::INFINITY);
        }

        // Distance is always positive
        Ok(ffd.abs())
    }

    fn calc_front_principal_plane(
        front_focal_distance: Float,
        effective_focal_length: Float,
    ) -> Float {
        // Principal planes make no sense for lenses without power
        if front_focal_distance.is_infinite() {
            return Float::NAN;
        }

        effective_focal_length - front_focal_distance
    }

    fn calc_marginal_ray(
        surfaces: &[Surface],
        pseudo_marginal_ray: &ParaxialRayBundle,
        aperture_stop: &usize,
        per_surf_v: &[TangentialVector],
    ) -> ParaxialRayBundle {
        let ratios: Vec<Float> = surfaces
            .iter()
            .zip(pseudo_marginal_ray.iter_surfaces())
            .zip(per_surf_v.iter())
            .map(|((s, rays), &v)| s.projected_semi_diameter(v) / rays[0].height)
            .collect();
        let scale_factor = ratios[*aperture_stop];

        let rays = pseudo_marginal_ray
            .rays
            .iter()
            .map(|r| ParaxialRay {
                height: r.height * scale_factor,
                angle: r.angle * scale_factor,
            })
            .collect();

        ParaxialRayBundle {
            rays,
            num_surfaces: pseudo_marginal_ray.num_surfaces,
        }
    }

    /// Compute the parallel ray.
    fn calc_parallel_ray(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
    ) -> Result<ParaxialRayBundle> {
        let ray = vec![ParaxialRay {
            height: 1.0,
            angle: 0.0,
        }];

        Self::trace(ray, sequential_sub_model, surfaces, false)
    }

    /// Compute the paraxial image plane.
    fn calc_paraxial_image_plane(
        surfaces: &[Surface],
        marginal_ray: &ParaxialRayBundle,
        chief_ray: &ParaxialRayBundle,
    ) -> Result<ImagePlane> {
        let last_physical_surface_id =
            last_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        let last_surface = surfaces[last_physical_surface_id].borrow();

        let d_axis = axis_intercepts(marginal_ray.rays_at_surface(last_physical_surface_id))?[0];
        let location = if d_axis.is_infinite() {
            // Ensure positive infinity is returned for infinite image planes
            Float::INFINITY
        } else {
            last_surface.track() + d_axis
        };

        // Propagate the chief ray from the last physical surface to the image plane to
        // determine its semi-diameter.
        let propagated = propagate(chief_ray.rays_at_surface(last_physical_surface_id), d_axis);
        let semi_diameter = propagated[0].height.abs();

        Ok(ImagePlane {
            location,
            semi_diameter,
        })
    }

    /// Compute the pseudo-marginal ray.
    fn calc_pseudo_marginal_ray(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
    ) -> Result<ParaxialRayBundle> {
        let ray = if sequential_sub_model.is_obj_at_inf() {
            // Ray parallel to axis at a height of 1
            vec![ParaxialRay {
                height: 1.0,
                angle: 0.0,
            }]
        } else {
            // Ray starting from the axis at an angle of 1
            vec![ParaxialRay {
                height: 0.0,
                angle: 1.0,
            }]
        };

        Self::trace(ray, sequential_sub_model, surfaces, false)
    }

    /// Compute the reverse parallel ray.
    fn calc_reverse_parallel_ray(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
    ) -> Result<ParaxialRayBundle> {
        let ray = vec![ParaxialRay {
            height: 1.0,
            angle: 0.0,
        }];

        Self::trace(ray, sequential_sub_model, surfaces, true)
    }

    /// Compute the ray transfer matrix for each gap/surface pair.
    fn rtms(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
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
        let mut reflected: i8 = 1;

        for (gap_0, surface, gap_1) in steps {
            let t = if gap_0.thickness.is_infinite() {
                DEFAULT_THICKNESS
            } else if reverse {
                // Reverse ray tracing is implemented as negative distances to avoid hassles
                // with inverses of ray transfer matrices.
                -gap_0.thickness
            } else {
                reflected as Float * gap_0.thickness
            };

            let roc = surface.roc();
            if let SurfaceType::Reflecting = surface.surface_type() {
                reflected *= -1;
            }

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
        initial_rays: Vec<ParaxialRay>,
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        reverse: bool,
    ) -> Result<ParaxialRayBundle> {
        let txs = Self::rtms(sequential_sub_model, surfaces, reverse)?;
        let num_surfaces = txs.len() + 1;
        let num_rays = initial_rays.len();
        let mut flat: Vec<ParaxialRay> = Vec::with_capacity(num_surfaces * num_rays);
        flat.extend_from_slice(&initial_rays);

        let mut current = initial_rays;
        for tx in &txs {
            let next: Vec<ParaxialRay> = current
                .iter()
                .map(|r| ParaxialRay {
                    height: tx.e[0][0] * r.height + tx.e[0][1] * r.angle,
                    angle: tx.e[1][0] * r.height + tx.e[1][1] * r.angle,
                })
                .collect();
            flat.extend_from_slice(&next);
            current = next;
        }

        Ok(ParaxialRayBundle {
            rays: flat,
            num_surfaces,
        })
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
    match surface {
        // Conics and torics behave the same in paraxial subviews.
        Surface::Conic(_) => match surface.surface_type() {
            SurfaceType::Refracting => Mat2x2::new(
                1.0,
                t,
                (n_0 - n_1) / n_1 / roc,
                t * (n_0 - n_1) / n_1 / roc + n_0 / n_1,
            ),

            // -1.0 in the second row flips the angle upon reflection so that we don't have to do
            // acrobatics flipping by the +z-direction instead
            SurfaceType::Reflecting => Mat2x2::new(1.0, t, -2.0 / roc, -1.0 - 2.0 * t / roc),
            SurfaceType::NoOp => panic!("Conics and torics cannot be NoOp surfaces."),
        },
        Surface::Image(_) | Surface::Probe(_) | Surface::Stop(_) => Mat2x2::new(1.0, t, 0.0, 1.0),
        Surface::Object(_) => Mat2x2::identity(),
    }
}

fn argmin(ratios: &[Float]) -> usize {
    ratios
        .iter()
        .enumerate()
        .fold((0, Float::MAX), |(min_idx, min_val), (idx, &val)| {
            if val < min_val {
                (idx, val)
            } else {
                (min_idx, min_val)
            }
        })
        .0
}

// Consider moving these to integration tests once the paraxial view and
// sequential models are combined into a system.
#[cfg(test)]
mod test {
    use approx::assert_abs_diff_eq;

    use crate::examples::convexplano_lens;
    use crate::{core::Float, n};

    use super::*;

    #[test]
    fn test_propagate() {
        let rays = vec![
            ParaxialRay {
                height: 1.0,
                angle: 4.0,
            },
            ParaxialRay {
                height: 2.0,
                angle: 5.0,
            },
            ParaxialRay {
                height: 3.0,
                angle: 6.0,
            },
        ];
        let propagated = propagate(&rays, 2.0);

        assert_abs_diff_eq!(propagated[0].height, 9.0, epsilon = 1e-4);
        assert_abs_diff_eq!(propagated[1].height, 12.0, epsilon = 1e-4);
        assert_abs_diff_eq!(propagated[2].height, 15.0, epsilon = 1e-4);
        assert_abs_diff_eq!(propagated[0].angle, 4.0, epsilon = 1e-4);
        assert_abs_diff_eq!(propagated[1].angle, 5.0, epsilon = 1e-4);
        assert_abs_diff_eq!(propagated[2].angle, 6.0, epsilon = 1e-4);
    }

    #[test]
    fn test_axis_intercepts() {
        let rays = vec![
            ParaxialRay {
                height: 1.0,
                angle: 4.0,
            },
            ParaxialRay {
                height: 2.0,
                angle: 5.0,
            },
            ParaxialRay {
                height: 3.0,
                angle: 6.0,
            },
            ParaxialRay {
                height: 0.0,
                angle: 7.0,
            },
        ];
        let intercepts = axis_intercepts(&rays).unwrap();

        assert_abs_diff_eq!(intercepts[0], -0.25, epsilon = 1e-4);
        assert_abs_diff_eq!(intercepts[1], -0.4, epsilon = 1e-4);
        assert_abs_diff_eq!(intercepts[2], -0.5, epsilon = 1e-4);
        assert_abs_diff_eq!(intercepts[3], 0.0, epsilon = 1e-4);
    }

    #[test]
    fn test_axis_intercepts_divide_by_zero() {
        let rays = vec![ParaxialRay {
            height: 1.0,
            angle: 0.0,
        }];
        let intercepts = axis_intercepts(&rays).unwrap();

        assert_eq!(intercepts.len(), 1);
        assert!(intercepts[0].is_infinite());
    }

    #[test]
    fn test_axis_intercepts_zero_height_divide_by_zero() {
        let rays = vec![ParaxialRay {
            height: 0.0,
            angle: 0.0,
        }];
        let intercepts = axis_intercepts(&rays);

        assert!(intercepts.is_err());
    }

    fn setup() -> (ParaxialSubView, SequentialModel) {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let sequential_model = convexplano_lens::sequential_model(air, nbk7, &wavelengths);
        let seq_sub_model = sequential_model
            .submodels()
            .get(&0usize)
            .expect("Submodel not found.");
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

        (
            ParaxialSubView::new(
                seq_sub_model,
                sequential_model.surfaces(),
                Vec3::new(0.0, 1.0, 0.0), // v = Y (phi=90°)
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
        let expected = [
            (12.5000, 0.0),
            (12.5000, -0.1647),
            (11.6271, -0.2495),
            (-0.0003, -0.2495),
        ];

        assert_eq!(marginal_ray.num_surfaces(), expected.len());
        for (surface_rays, (exp_h, exp_a)) in marginal_ray.iter_surfaces().zip(expected.iter()) {
            assert_abs_diff_eq!(surface_rays[0].height, *exp_h, epsilon = 1e-4);
            assert_abs_diff_eq!(surface_rays[0].angle, *exp_a, epsilon = 1e-4);
        }
    }

    #[test]
    fn test_pseudo_marginal_ray() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let sequential_model = convexplano_lens::sequential_model(air, nbk7, &wavelengths);
        let seq_sub_model = sequential_model
            .submodels()
            .get(&0usize)
            .expect("Submodel not found.");
        let pseudo_marginal_ray =
            ParaxialSubView::calc_pseudo_marginal_ray(seq_sub_model, sequential_model.surfaces())
                .unwrap();

        let expected = [
            (1.0000, 0.0),
            (1.0000, -0.0132),
            (0.9302, -0.0200),
            (0.0, -0.0200),
        ];

        assert_eq!(pseudo_marginal_ray.num_surfaces(), expected.len());
        for (surface_rays, (exp_h, exp_a)) in
            pseudo_marginal_ray.iter_surfaces().zip(expected.iter())
        {
            assert_abs_diff_eq!(surface_rays[0].height, *exp_h, epsilon = 1e-4);
            assert_abs_diff_eq!(surface_rays[0].angle, *exp_a, epsilon = 1e-4);
        }
    }

    #[test]
    fn test_reverse_parallel_ray() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let sequential_model = convexplano_lens::sequential_model(air, nbk7, &wavelengths);
        let seq_sub_model = sequential_model
            .submodels()
            .get(&0usize)
            .expect("Submodel not found.");
        let reverse_parallel_ray =
            ParaxialSubView::calc_reverse_parallel_ray(seq_sub_model, sequential_model.surfaces())
                .unwrap();

        let expected = [(1.0000, 0.0), (1.0000, 0.0), (1.0000, 0.0200)];

        assert_eq!(reverse_parallel_ray.num_surfaces(), expected.len());
        for (surface_rays, (exp_h, exp_a)) in
            reverse_parallel_ray.iter_surfaces().zip(expected.iter())
        {
            assert_abs_diff_eq!(surface_rays[0].height, *exp_h, epsilon = 1e-4);
            assert_abs_diff_eq!(surface_rays[0].angle, *exp_a, epsilon = 1e-4);
        }
    }
}
