/// A paraxial view into an optical system.
///
/// Paraxial optics is a simplified model of optical systems that assumes that
/// rays are close to the optic axis and that angles are small. Rays are traced
/// through the system using ray transfer matrices, which are 2x2 matrices that
/// describe how rays propagate through and interact with optical surfaces. The
/// paraxial view is used to compute the paraxial parameters of an optical
/// system, such as the entrance and exit pupils, the back and front focal
/// distances, and the effective focal length.
use anyhow::{Result, anyhow};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    FieldSpec,
    core::{
        Float,
        math::{linalg::mat2x2::Mat2x2, vec3::Vec3},
        sequential_model::{
            CursorPlacement, SequentialModel, SequentialSubModel, Step, first_physical_step,
            last_physical_step, propagate_tangential_vec, reversed_surface_id,
            surface_placement::SurfacePlacement,
        },
        surfaces::Surface,
    },
    specs::{
        fields::unique_tangential_vecs,
        surfaces::{BeamSplitterPathKind, BoundaryKind},
    },
};

const DEFAULT_THICKNESS: Float = 0.0;

/// A unit vector in the global frame that defines a meridional (tangential)
/// plane. It lies in the transverse R–U plane (z-component = 0 at the object)
/// and is propagated through fold mirrors via the vector law of reflection.
///
/// For phi=0°: `(1, 0, 0)` (cursor-R / global X).
/// For phi=90°: `(0, 1, 0)` (cursor-U / global Y).
type TangentialVector = Vec3;

/// A single paraxial ray with height and angle components.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
/// Subviews are stored in a flat `Vec`, ordered by wavelength index first, then
/// by `tangential_vec_id`. Each subview carries its own `wavelength_id` and
/// `tangential_vec_id`.
#[derive(Debug)]
pub struct ParaxialView {
    tangential_vecs: Vec<TangentialVector>,
    subviews: Vec<ParaxialSubView>,
    wavelengths: Vec<Float>,
}

/// A description of a paraxial optical system.
///
/// This is used primarily for serialization of data for export.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ParaxialViewDescription {
    subviews: Vec<ParaxialSubViewDescription>,
    /// Indexed by tangential_vec_id (index into the tangential-vector table).
    primary_axial_color: Vec<Float>,
}

/// A paraxial subview of an optical system.
///
/// A paraxial subview is identified by a path index, a wavelength index, and a
/// tangential direction index. It is not created by the user, but rather by
/// instantiating a new ParaxialView struct.
#[derive(Debug)]
pub struct ParaxialSubView {
    path_id: usize,
    wavelength_id: usize,
    tangential_vec_id: usize,
    is_obj_space_telecentric: bool,

    aperture_stop: usize,
    back_focal_distance: Float,
    back_principal_plane: Float,
    chief_ray: ParaxialRayBundle,
    effective_focal_length: Float,
    entrance_pupil: Pupil,
    exit_pupil: Pupil,
    front_focal_distance: Float,
    front_focal_length: Float,
    front_principal_plane: Float,
    image_space_fno: Float,
    lagrange_invariants: Vec<Float>,
    marginal_ray: ParaxialRayBundle,
    paraxial_fno: Float,
    paraxial_image_plane: ImagePlane,
}

/// A paraxial description of a submodel of an optical system.
///
/// This is used primarily for serialization of data for export.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ParaxialSubViewDescription {
    path_id: usize,
    wavelength_id: usize,
    tangential_vec_id: usize,
    aperture_stop: usize,
    back_focal_distance: Float,
    back_principal_plane: Float,
    chief_ray: ParaxialRayBundle,
    effective_focal_length: Float,
    entrance_pupil: Pupil,
    exit_pupil: Pupil,
    front_focal_distance: Float,
    front_focal_length: Float,
    front_principal_plane: Float,
    image_space_fno: Float,
    lagrange_invariants: Vec<Float>,
    marginal_ray: ParaxialRayBundle,
    paraxial_fno: Float,
    paraxial_image_plane: ImagePlane,
}

/// A paraxial entrance or exit pupil.
///
/// # Attributes
/// * `location` - The location of the pupil relative to the first non-object
///   surface.
/// * `semi_diameter` - The semi-diameter of the pupil.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
        let placements = sequential_model.placements();
        let tangential_vecs: Vec<TangentialVector> = if sequential_model.is_rotationally_symmetric()
        {
            vec![Vec3::new(0.0, 1.0, 0.0)]
        } else {
            unique_tangential_vecs(field_specs)
        };

        let mut subviews = Vec::new();
        for path_id in 0..sequential_model.path_count() {
            let stop_surface = sequential_model.stop_surface_for_path(path_id);
            let path_steps = sequential_model.path_steps(path_id);
            let surface_indices = sequential_model.path_surface_indices(path_id);
            let beam_splitter_arms = sequential_model.path_beam_splitter_arms(path_id);
            for (wav_idx, submodel) in sequential_model
                .submodels_for_path(path_id)
                .iter()
                .enumerate()
            {
                for (v_idx, &v) in tangential_vecs.iter().enumerate() {
                    let data = SubModelData {
                        sequential_sub_model: submodel as &dyn SequentialSubModel,
                        surfaces,
                        placements,
                        surface_indices,
                        beam_splitter_arms,
                        path_steps,
                        field_specs,
                        stop_surface,
                    };
                    let subview = ParaxialSubView::new(
                        path_id,
                        wav_idx,
                        v_idx,
                        &data,
                        v,
                        is_obj_space_telecentric,
                    )?;
                    subviews.push(subview);
                }
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
            subviews: self.subviews.iter().map(|sv| sv.describe()).collect(),
            primary_axial_color: self.primary_axial_color(),
        }
    }

    /// Returns the subview for path 0, the given wavelength, and
    /// tangential-direction indices, or `None` if no such subview exists.
    pub fn get(&self, wavelength_id: usize, tangential_vec_id: usize) -> Option<&ParaxialSubView> {
        self.get_for_path(0, wavelength_id, tangential_vec_id)
    }

    /// Returns the subview for the given path, wavelength, and
    /// tangential-direction indices, or `None` if no such subview exists.
    pub fn get_for_path(
        &self,
        path_id: usize,
        wavelength_id: usize,
        tangential_vec_id: usize,
    ) -> Option<&ParaxialSubView> {
        self.subviews.iter().find(|sv| {
            sv.path_id == path_id
                && sv.wavelength_id == wavelength_id
                && sv.tangential_vec_id == tangential_vec_id
        })
    }

    /// Returns an iterator over all subviews.
    pub fn iter(&self) -> impl Iterator<Item = &ParaxialSubView> {
        self.subviews.iter()
    }

    /// Returns an iterator over all subviews for a given wavelength index.
    pub fn get_by_wavelength_id(
        &self,
        wavelength_id: usize,
    ) -> impl Iterator<Item = &ParaxialSubView> {
        self.subviews
            .iter()
            .filter(move |sv| sv.wavelength_id == wavelength_id)
    }

    /// Returns an iterator over all subviews for a given tangential-direction
    /// index.
    pub fn get_by_tangential_vec_id(
        &self,
        tangential_vec_id: usize,
    ) -> impl Iterator<Item = &ParaxialSubView> {
        self.subviews
            .iter()
            .filter(move |sv| sv.tangential_vec_id == tangential_vec_id)
    }

    /// Returns the tangential direction vector for a given tangential_vec_id.
    pub fn tangential_vec(&self, tangential_vec_id: usize) -> TangentialVector {
        self.tangential_vecs[tangential_vec_id]
    }

    /// Returns the azimuthal angle in degrees for a given tangential_vec_id.
    pub fn phi_deg(&self, tangential_vec_id: usize) -> Float {
        let v = self.tangential_vecs[tangential_vec_id];
        v.y().atan2(v.x()).to_degrees()
    }

    /// Returns the tangential_vec_id whose tangential vector is closest (by dot
    /// product) to the given azimuthal angle in radians.
    ///
    /// For the common case where `phi_rad` exactly matches a stored phi key
    /// (bit-identical `tangential_fan_phi()` value), this finds the exact
    /// entry. Falls back to index 0 if the table is empty.
    pub fn tangential_vec_id_for_phi(&self, phi_rad: Float) -> usize {
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
    pub fn primary_axial_color(&self) -> Vec<Float> {
        let min_watangential_vec_id = self
            .wavelengths
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or_default();
        let max_watangential_vec_id = self
            .wavelengths
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or_default();

        let mut primary_axial_color = vec![0.0; self.tangential_vecs.len()];
        for sv_min in self.get_by_wavelength_id(min_watangential_vec_id) {
            if let Some(sv_max) = self.get(max_watangential_vec_id, sv_min.tangential_vec_id) {
                let diff =
                    (sv_max.effective_focal_length() - sv_min.effective_focal_length()).abs();
                primary_axial_color[sv_min.tangential_vec_id] = diff;
            }
        }

        primary_axial_color
    }
}

/// Bundles the per-submodel slices that `ParaxialSubView::new` needs.
///
/// Grouping these avoids exceeding the clippy `too_many_arguments` limit while
/// keeping the call sites readable.
struct SubModelData<'a> {
    sequential_sub_model: &'a dyn SequentialSubModel,
    surfaces: &'a [Box<dyn Surface>],
    placements: &'a [SurfacePlacement],
    surface_indices: &'a [usize],
    beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
    path_steps: &'a [CursorPlacement],
    field_specs: &'a [FieldSpec],
    stop_surface: Option<usize>,
}

impl ParaxialSubView {
    /// Create a new paraxial subview for the given tangential direction.
    ///
    /// `v` is a `TangentialVector` in the global frame defining the meridional
    /// plane (e.g. `(0,1,0)` for phi=90°). It is propagated through mirror
    /// surfaces internally to compute per-surface foreshortening.
    fn new(
        path_id: usize,
        wavelength_id: usize,
        tangential_vec_id: usize,
        data: &SubModelData<'_>,
        v: TangentialVector,
        is_obj_space_telecentric: bool,
    ) -> Result<Self> {
        let sequential_sub_model = data.sequential_sub_model;
        let surfaces = data.surfaces;
        let placements = data.placements;
        let surface_indices = data.surface_indices;
        let beam_splitter_arms = data.beam_splitter_arms;
        let path_steps = data.path_steps;
        let field_specs = data.field_specs;
        // Propagate v through this path's mirror surfaces to get per-step tangential
        // vectors.
        let per_surf_v: Vec<TangentialVector> =
            propagate_tangential_vec(v, surfaces, placements, surface_indices);

        let pseudo_marginal_ray = Self::calc_pseudo_marginal_ray(
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
        )?;
        let parallel_ray = Self::calc_parallel_ray(
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
        )?;
        let reverse_parallel_ray = Self::calc_reverse_parallel_ray(
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
        )?;

        let aperture_stop = match data.stop_surface {
            Some(i) => resolve_stop_surface_step(i, surface_indices)?,
            None => Self::calc_aperture_stop(
                surfaces,
                placements,
                surface_indices,
                path_steps,
                &pseudo_marginal_ray,
                &per_surf_v,
            ),
        };
        let back_focal_distance =
            Self::calc_back_focal_distance(surfaces, surface_indices, &parallel_ray)?;
        let front_focal_distance = Self::calc_front_focal_distance(
            sequential_sub_model,
            surfaces,
            surface_indices,
            &reverse_parallel_ray,
        )?;
        let marginal_ray = Self::calc_marginal_ray(
            surfaces,
            placements,
            surface_indices,
            path_steps,
            &pseudo_marginal_ray,
            &aperture_stop,
            &per_surf_v,
        );
        let entrance_pupil = Self::calc_entrance_pupil(
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
            is_obj_space_telecentric,
            &aperture_stop,
            &per_surf_v,
            &marginal_ray,
        )?;
        let exit_pupil = Self::calc_exit_pupil(
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
            &aperture_stop,
            &marginal_ray,
        )?;
        let effective_focal_length = Self::calc_effective_focal_length(&parallel_ray);
        let front_focal_length = Self::calc_front_focal_length(
            sequential_sub_model,
            surfaces,
            surface_indices,
            &reverse_parallel_ray,
        )?;

        let back_principal_plane =
            Self::calc_back_principal_plane(back_focal_distance, effective_focal_length)?;
        let front_principal_plane =
            Self::calc_front_principal_plane(front_focal_distance, front_focal_length);

        let chief_ray = Self::calc_chief_ray(
            surfaces,
            sequential_sub_model,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
            v,
            field_specs,
            &entrance_pupil,
        )?;
        let paraxial_image_plane =
            Self::calc_paraxial_image_plane(surfaces, surface_indices, &marginal_ray, &chief_ray)?;

        let last_phys_id = last_physical_step(surface_indices, surfaces)
            .ok_or_else(|| anyhow!("There are no physical surfaces"))?;
        let n_image = sequential_sub_model
            .gaps()
            .last()
            .expect("submodel must have at least one gap")
            .refractive_index
            .n();
        let u_last = marginal_ray.rays_at_surface(last_phys_id)[0].angle;
        let paraxial_fno = 1.0 / (2.0 * n_image * u_last);
        let image_space_fno = effective_focal_length / (2.0 * entrance_pupil.semi_diameter);
        let lagrange_invariants = (0..sequential_sub_model.gaps().len())
            .map(|i| {
                Self::calc_lagrange_invariant(i, &marginal_ray, &chief_ray, sequential_sub_model)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            path_id,
            wavelength_id,
            tangential_vec_id,
            is_obj_space_telecentric,

            aperture_stop,
            back_focal_distance,
            back_principal_plane,
            chief_ray,
            effective_focal_length,
            entrance_pupil,
            exit_pupil,
            front_focal_distance,
            front_focal_length,
            front_principal_plane,
            image_space_fno,
            lagrange_invariants,
            marginal_ray,
            paraxial_fno,
            paraxial_image_plane,
        })
    }

    fn describe(&self) -> ParaxialSubViewDescription {
        ParaxialSubViewDescription {
            path_id: self.path_id,
            wavelength_id: self.wavelength_id,
            tangential_vec_id: self.tangential_vec_id,
            aperture_stop: self.aperture_stop,
            back_focal_distance: self.back_focal_distance,
            back_principal_plane: self.back_principal_plane,
            chief_ray: self.chief_ray.clone(),
            effective_focal_length: self.effective_focal_length,
            entrance_pupil: self.entrance_pupil.clone(),
            exit_pupil: self.exit_pupil.clone(),
            front_focal_distance: self.front_focal_distance,
            front_focal_length: self.front_focal_length,
            front_principal_plane: self.front_principal_plane,
            image_space_fno: self.image_space_fno,
            lagrange_invariants: self.lagrange_invariants.clone(),
            marginal_ray: self.marginal_ray.clone(),
            paraxial_fno: self.paraxial_fno,
            paraxial_image_plane: self.paraxial_image_plane.clone(),
        }
    }

    pub fn path_id(&self) -> usize {
        self.path_id
    }

    pub fn wavelength_id(&self) -> usize {
        self.wavelength_id
    }

    pub fn tangential_vec_id(&self) -> usize {
        self.tangential_vec_id
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

    pub fn front_focal_length(&self) -> &Float {
        &self.front_focal_length
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

    pub fn paraxial_fno(&self) -> Float {
        self.paraxial_fno
    }

    pub fn image_space_fno(&self) -> Float {
        self.image_space_fno
    }

    pub fn lagrange_invariants(&self) -> &[Float] {
        &self.lagrange_invariants
    }

    fn calc_aperture_stop(
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        path_steps: &[CursorPlacement],
        pseudo_marginal_ray: &ParaxialRayBundle,
        per_surf_v: &[TangentialVector],
    ) -> usize {
        calc_aperture_stop(
            surfaces,
            placements,
            surface_indices,
            path_steps,
            pseudo_marginal_ray,
            per_surf_v,
        )
    }

    fn calc_back_focal_distance(
        surfaces: &[Box<dyn Surface>],
        surface_indices: &[usize],
        parallel_ray: &ParaxialRayBundle,
    ) -> Result<Float> {
        let last_physical_step_index = last_physical_step(surface_indices, surfaces)
            .ok_or(anyhow!("There are no physical surfaces"))?;
        let intercepts = axis_intercepts(parallel_ray.rays_at_surface(last_physical_step_index))?;

        let bfd = intercepts[0];

        // Handle edge case for infinite BFD
        if bfd.is_infinite() {
            return Ok(Float::INFINITY);
        }

        Ok(bfd)
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
    #[allow(clippy::too_many_arguments)]
    fn calc_chief_ray(
        surfaces: &[Box<dyn Surface>],
        sequential_sub_model: &dyn SequentialSubModel,
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        beam_splitter_arms: &[Option<BeamSplitterPathKind>],
        path_steps: &[CursorPlacement],
        v: TangentialVector,
        field_specs: &[FieldSpec],
        entrance_pupil: &Pupil,
    ) -> Result<ParaxialRayBundle> {
        let enp_loc = entrance_pupil.location;
        let obj_loc = placements
            .first()
            .ok_or(anyhow!("No surfaces provided"))?
            .track;
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
        Self::trace(
            initial_ray,
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
            false,
        )
    }

    fn calc_effective_focal_length(parallel_ray: &ParaxialRayBundle) -> Float {
        let y_1 = parallel_ray.rays_at_surface(1)[0].height;
        let u_final = parallel_ray.rays_at_surface(parallel_ray.num_surfaces() - 2)[0].angle;

        let efl = -y_1 / u_final;

        // Handle edge case for negatively infinite EFL
        if efl.is_infinite() {
            return Float::INFINITY;
        }

        efl
    }

    #[allow(clippy::too_many_arguments)]
    fn calc_entrance_pupil(
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        beam_splitter_arms: &[Option<BeamSplitterPathKind>],
        path_steps: &[CursorPlacement],
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

        // In case the aperture stop is the first surface (step 1).
        if *aperture_stop == 1usize {
            let store_idx = surface_indices[1];
            let crm = path_steps[1].cursor_rotation_matrix;
            return Ok(Pupil {
                location: 0.0,
                semi_diameter: placements[store_idx].projected_semi_diameter(
                    crm,
                    surfaces[store_idx].mask().semi_diameter(),
                    per_surf_v[1],
                ),
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
            &sequential_sub_model.slice(0..*aperture_stop, surface_indices, beam_splitter_arms),
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
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
            semi_diameter: semi_diameter.abs(), // semi-diameter is always positive
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn calc_exit_pupil(
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        beam_splitter_arms: &[Option<BeamSplitterPathKind>],
        path_steps: &[CursorPlacement],
        aperture_stop: &usize,
        marginal_ray: &ParaxialRayBundle,
    ) -> Result<Pupil> {
        let last_physical_step_id = last_physical_step(surface_indices, surfaces)
            .ok_or(anyhow!("There are no physical surfaces"))?;
        if last_physical_step_id == *aperture_stop {
            let store_idx = surface_indices[last_physical_step_id];
            return Ok(Pupil {
                location: 0.0,
                semi_diameter: surfaces[store_idx].mask().semi_diameter(),
            });
        }

        // Trace a ray through the aperture stop forwards through the system
        let ray = vec![ParaxialRay {
            height: 0.0,
            angle: 1.0,
        }];

        let results = Self::trace(
            ray,
            &sequential_sub_model.slice(
                *aperture_stop..sequential_sub_model.len(),
                surface_indices,
                beam_splitter_arms,
            ),
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
            false,
        )?;

        // Distance is relative to the last physical surface (step-indexed in the slice)
        let sliced_last_physical_step_id = last_physical_step_id - aperture_stop;
        let distance = axis_intercepts(results.rays_at_surface(sliced_last_physical_step_id))?[0];

        // Propagate the marginal ray to the exit pupil location and find its height
        let semi_diameter = propagate(
            marginal_ray.rays_at_surface(last_physical_step_id),
            distance,
        )[0]
        .height;

        Ok(Pupil {
            location: distance,
            semi_diameter,
        })
    }

    fn calc_front_focal_distance(
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        surface_indices: &[usize],
        reverse_parallel_ray: &ParaxialRayBundle,
    ) -> Result<Float> {
        let first_physical_step_index = first_physical_step(surface_indices, surfaces)
            .ok_or(anyhow!("There are no physical surfaces"))?;
        let index = reversed_surface_id(sequential_sub_model.len() + 1, first_physical_step_index);
        let intercepts = axis_intercepts(reverse_parallel_ray.rays_at_surface(index))?;

        let ffd = intercepts[0];

        // Handle edge case for infinite FFD
        if ffd.is_infinite() {
            return Ok(Float::INFINITY);
        }

        Ok(ffd)
    }

    fn calc_front_focal_length(
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        surface_indices: &[usize],
        reverse_parallel_ray: &ParaxialRayBundle,
    ) -> Result<Float> {
        // y_1: height at the first physical surface in the reverse trace
        let last_physical_step_index = last_physical_step(surface_indices, surfaces)
            .ok_or(anyhow!("There are no physical surfaces"))?;
        let y_index = reversed_surface_id(sequential_sub_model.len() + 1, last_physical_step_index);
        let y_1 = reverse_parallel_ray.rays_at_surface(y_index)[0].height;

        // u_final: angle at the last physical surface in the reverse trace
        let first_physical_step_index = first_physical_step(surface_indices, surfaces)
            .ok_or(anyhow!("There are no physical surfaces"))?;
        let u_index =
            reversed_surface_id(sequential_sub_model.len() + 1, first_physical_step_index);
        let u_final = reverse_parallel_ray.rays_at_surface(u_index)[0].angle;

        if u_final == 0.0 {
            return Ok(Float::INFINITY);
        }
        Ok(-y_1 / u_final)
    }

    fn calc_front_principal_plane(front_focal_distance: Float, front_focal_length: Float) -> Float {
        // Principal planes make no sense for lenses without power
        if front_focal_distance.is_infinite() {
            return Float::NAN;
        }

        front_focal_distance - front_focal_length
    }

    fn calc_lagrange_invariant(
        surface_id: usize,
        marginal_ray: &ParaxialRayBundle,
        chief_ray: &ParaxialRayBundle,
        sequential_sub_model: &dyn SequentialSubModel,
    ) -> Result<Float> {
        let n = sequential_sub_model
            .gaps()
            .get(surface_id)
            .ok_or_else(|| anyhow!("surface_id {surface_id} out of range"))?
            .refractive_index
            .n();
        let mr = marginal_ray.rays_at_surface(surface_id);
        let cr = chief_ray.rays_at_surface(surface_id);
        Ok(n * (mr[0].height * cr[0].angle - cr[0].height * mr[0].angle))
    }

    fn calc_marginal_ray(
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        path_steps: &[CursorPlacement],
        pseudo_marginal_ray: &ParaxialRayBundle,
        aperture_stop: &usize,
        per_surf_v: &[TangentialVector],
    ) -> ParaxialRayBundle {
        calc_marginal_ray(
            surfaces,
            placements,
            surface_indices,
            path_steps,
            pseudo_marginal_ray,
            aperture_stop,
            per_surf_v,
        )
    }

    /// Compute the parallel ray.
    fn calc_parallel_ray(
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        beam_splitter_arms: &[Option<BeamSplitterPathKind>],
        path_steps: &[CursorPlacement],
    ) -> Result<ParaxialRayBundle> {
        let ray = vec![ParaxialRay {
            height: 1.0,
            angle: 0.0,
        }];

        Self::trace(
            ray,
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
            false,
        )
    }

    /// Compute the paraxial image plane.
    fn calc_paraxial_image_plane(
        surfaces: &[Box<dyn Surface>],
        surface_indices: &[usize],
        marginal_ray: &ParaxialRayBundle,
        chief_ray: &ParaxialRayBundle,
    ) -> Result<ImagePlane> {
        let last_physical_step_id = last_physical_step(surface_indices, surfaces)
            .ok_or(anyhow!("There are no physical surfaces"))?;

        let d_axis = axis_intercepts(marginal_ray.rays_at_surface(last_physical_step_id))?[0];
        let location = if d_axis.is_infinite() {
            // Ensure positive infinity is returned for infinite image planes
            Float::INFINITY
        } else {
            // Compute the paraxial image plane location relative to the last physical
            // surface
            d_axis
        };

        // Propagate the chief ray from the last physical surface to the image plane to
        // determine its semi-diameter.
        let propagated = propagate(chief_ray.rays_at_surface(last_physical_step_id), d_axis);
        let semi_diameter = propagated[0].height.abs();

        Ok(ImagePlane {
            location,
            semi_diameter,
        })
    }

    /// Compute the pseudo-marginal ray.
    fn calc_pseudo_marginal_ray(
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        beam_splitter_arms: &[Option<BeamSplitterPathKind>],
        path_steps: &[CursorPlacement],
    ) -> Result<ParaxialRayBundle> {
        calc_pseudo_marginal_ray(
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
        )
    }

    /// Compute the reverse parallel ray.
    fn calc_reverse_parallel_ray(
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        beam_splitter_arms: &[Option<BeamSplitterPathKind>],
        path_steps: &[CursorPlacement],
    ) -> Result<ParaxialRayBundle> {
        let ray = vec![ParaxialRay {
            height: 1.0,
            angle: 0.0,
        }];

        Self::trace(
            ray,
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
            true,
        )
    }

    /// Compute the ray transfer matrix for each gap/surface pair.
    fn rtms(
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        beam_splitter_arms: &[Option<BeamSplitterPathKind>],
        path_steps: &[CursorPlacement],
        reverse: bool,
    ) -> Result<Vec<RayTransferMatrix>> {
        let mut txs: Vec<RayTransferMatrix> = Vec::new();
        let mut forward_iter;
        let mut reverse_iter;
        let steps: &mut dyn Iterator<Item = Step> = if reverse {
            reverse_iter = sequential_sub_model
                .try_iter(
                    surfaces,
                    placements,
                    surface_indices,
                    beam_splitter_arms,
                    path_steps,
                )?
                .try_reverse()?;
            &mut reverse_iter
        } else {
            forward_iter = sequential_sub_model.try_iter(
                surfaces,
                placements,
                surface_indices,
                beam_splitter_arms,
                path_steps,
            )?;
            &mut forward_iter
        };
        for Step {
            gap_before: gap_0,
            surface,
            gap_after: gap_1,
            ..
        } in steps
        {
            let t = if gap_0.thickness.is_infinite() {
                DEFAULT_THICKNESS
            } else if reverse {
                // Reverse ray tracing is implemented as negative distances to avoid hassles
                // with inverses of ray transfer matrices.
                -gap_0.thickness
            } else {
                gap_0.thickness
            };

            let n_0 = gap_0.refractive_index.n();
            let n_1 = if let Some(gap_1) = gap_1 {
                gap_1.refractive_index.n()
            } else {
                gap_0.refractive_index.n()
            };

            let rtm = surface_to_rtm(surface, t, n_0, n_1, reverse);
            txs.push(rtm);
        }

        Ok(txs)
    }

    #[allow(clippy::too_many_arguments)]
    fn trace(
        initial_rays: Vec<ParaxialRay>,
        sequential_sub_model: &dyn SequentialSubModel,
        surfaces: &[Box<dyn Surface>],
        placements: &[SurfacePlacement],
        surface_indices: &[usize],
        beam_splitter_arms: &[Option<BeamSplitterPathKind>],
        path_steps: &[CursorPlacement],
        reverse: bool,
    ) -> Result<ParaxialRayBundle> {
        let txs = Self::rtms(
            sequential_sub_model,
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            path_steps,
            reverse,
        )?;
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
    surface: &dyn Surface,
    t: Float,
    n_0: Float,
    n_1: Float,
    reverse: bool,
) -> RayTransferMatrix {
    match surface.boundary_kind() {
        BoundaryKind::Refracting => {
            let phi = if reverse {
                -surface.power(0.0, n_1, n_0)
            } else {
                surface.power(0.0, n_0, n_1)
            };
            Mat2x2::new(1.0, t, -phi / n_1, -phi * t / n_1 + n_0 / n_1)
        }
        BoundaryKind::Reflecting => {
            let roc = surface.roc(0.0);
            Mat2x2::new(1.0, t, 2.0 / roc, 2.0 * t / roc + 1.0)
        }
        BoundaryKind::NoOp => Mat2x2::new(1.0, t, 0.0, 1.0),
    }
}

/// Compute the pseudo-marginal ray for a submodel.
pub(crate) fn calc_pseudo_marginal_ray(
    sequential_sub_model: &dyn SequentialSubModel,
    surfaces: &[Box<dyn Surface>],
    placements: &[SurfacePlacement],
    surface_indices: &[usize],
    beam_splitter_arms: &[Option<BeamSplitterPathKind>],
    path_steps: &[CursorPlacement],
) -> Result<ParaxialRayBundle> {
    let ray = if sequential_sub_model.is_obj_at_inf() {
        vec![ParaxialRay {
            height: 1.0,
            angle: 0.0,
        }]
    } else {
        vec![ParaxialRay {
            height: 0.0,
            angle: 1.0,
        }]
    };
    ParaxialSubView::trace(
        ray,
        sequential_sub_model,
        surfaces,
        placements,
        surface_indices,
        beam_splitter_arms,
        path_steps,
        false,
    )
}

/// Compute the aperture stop step index using the minimum aperture-ratio
/// heuristic.
///
/// Returns a step index (position within the path's traversal), not a store
/// index. `per_surf_v` must be step-indexed, as returned by
/// `propagate_tangential_vec` with the path's `surface_indices`.
pub(crate) fn calc_aperture_stop(
    surfaces: &[Box<dyn Surface>],
    placements: &[SurfacePlacement],
    surface_indices: &[usize],
    path_steps: &[CursorPlacement],
    pseudo_marginal_ray: &ParaxialRayBundle,
    per_surf_v: &[TangentialVector],
) -> usize {
    let ratios: Vec<Float> = surface_indices
        .iter()
        .zip(path_steps.iter())
        .zip(pseudo_marginal_ray.iter_surfaces())
        .zip(per_surf_v.iter())
        .map(|(((idx, step), rays), &v)| {
            let s = &surfaces[*idx];
            let p = &placements[*idx];
            let crm = step.cursor_rotation_matrix;
            (p.projected_semi_diameter(crm, s.mask().semi_diameter(), v) / rays[0].height).abs()
        })
        .collect();
    argmin(&ratios[1..ratios.len() - 1]) + 1
}

/// Scale the pseudo-marginal ray to match the aperture stop semi-diameter.
///
/// `aperture_stop` is a step index. `per_surf_v` must be step-indexed, as
/// returned by `propagate_tangential_vec` with the path's `surface_indices`.
pub(crate) fn calc_marginal_ray(
    surfaces: &[Box<dyn Surface>],
    placements: &[SurfacePlacement],
    surface_indices: &[usize],
    path_steps: &[CursorPlacement],
    pseudo_marginal_ray: &ParaxialRayBundle,
    aperture_stop: &usize,
    per_surf_v: &[TangentialVector],
) -> ParaxialRayBundle {
    let ratios: Vec<Float> = surface_indices
        .iter()
        .zip(path_steps.iter())
        .zip(pseudo_marginal_ray.iter_surfaces())
        .zip(per_surf_v.iter())
        .map(|(((idx, step), rays), &v)| {
            let s = &surfaces[*idx];
            let p = &placements[*idx];
            let crm = step.cursor_rotation_matrix;
            p.projected_semi_diameter(crm, s.mask().semi_diameter(), v) / rays[0].height
        })
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

/// Resolves a user-specified `stop_surface` (a store index, validated at
/// build time against the whole model's surface store —
/// `SequentialModel::validate_stop_surface`) to a step index within one
/// path's own traversal.
///
/// Every paraxial computation that consumes an aperture stop
/// (`calc_marginal_ray`, the ray-height-based ratios in `calc_aperture_stop`)
/// indexes by step position within `surface_indices`, not by store position.
/// For a path whose surfaces are all `New` in store order, store index and step
/// index coincide by construction, so this only has visible effect once a path
/// contains `Shared`/`ObjectLinkedTo` steps ahead of the stop.
///
/// Errors if `store_index` does not appear in `surface_indices` at all, or
/// appears more than once (ambiguous which occurrence is the stop).
fn resolve_stop_surface_step(store_index: usize, surface_indices: &[usize]) -> Result<usize> {
    let mut occurrences = surface_indices
        .iter()
        .enumerate()
        .filter(|&(_, &si)| si == store_index)
        .map(|(step, _)| step);
    match (occurrences.next(), occurrences.next()) {
        (None, _) => Err(anyhow!(
            "stop surface (store index {store_index}) is not visited by this path"
        )),
        (Some(_), Some(_)) => Err(anyhow!(
            "stop surface (store index {store_index}) is visited more than once by this \
             path; ambiguous which occurrence is the aperture stop"
        )),
        (Some(step), None) => Ok(step),
    }
}

/// Compute the paraxial marginal ray bundle for a given path and wavelength.
///
/// Uses the first tangential direction `(0, 1, 0)`, valid for all rotationally
/// symmetric systems.
pub(crate) fn marginal_ray_bundle_for_path(
    model: &SequentialModel,
    path_id: usize,
    wavelength_id: usize,
) -> Result<ParaxialRayBundle> {
    let submodel = model
        .submodels_for_path(path_id)
        .get(wavelength_id)
        .ok_or_else(|| anyhow!("wavelength_id {wavelength_id} out of range for path {path_id}"))?;
    let surfaces = model.surfaces();
    let placements = model.placements();
    let path_steps = model.path_steps(path_id);
    let surface_indices = model.path_surface_indices(path_id);
    let beam_splitter_arms = model.path_beam_splitter_arms(path_id);

    let v = Vec3::new(0.0, 1.0, 0.0);
    let per_surf_v = propagate_tangential_vec(v, surfaces, placements, surface_indices);
    let pseudo = calc_pseudo_marginal_ray(
        submodel,
        surfaces,
        placements,
        surface_indices,
        beam_splitter_arms,
        path_steps,
    )?;
    let stop = match model.stop_surface_for_path(path_id) {
        Some(i) => resolve_stop_surface_step(i, surface_indices)?,
        None => calc_aperture_stop(
            surfaces,
            placements,
            surface_indices,
            path_steps,
            &pseudo,
            &per_surf_v,
        ),
    };
    Ok(calc_marginal_ray(
        surfaces,
        placements,
        surface_indices,
        path_steps,
        &pseudo,
        &stop,
        &per_surf_v,
    ))
}

/// Single-path shorthand; delegates to path 0.
pub(crate) fn marginal_ray_bundle(
    model: &SequentialModel,
    wavelength_id: usize,
) -> Result<ParaxialRayBundle> {
    marginal_ray_bundle_for_path(model, 0, wavelength_id)
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
    use crate::{
        GapSpec, Rotation3D, SequentialModel, SurfaceSpec, Vec3, core::Float, n,
        specs::surfaces::BoundaryKind,
    };

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
        let seq_sub_model = sequential_model.submodel(0).expect("Submodel not found.");
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

        let data = SubModelData {
            sequential_sub_model: seq_sub_model as &dyn SequentialSubModel,
            surfaces: sequential_model.surfaces(),
            placements: sequential_model.placements(),
            surface_indices: sequential_model.path_surface_indices(0),
            beam_splitter_arms: sequential_model.path_beam_splitter_arms(0),
            path_steps: sequential_model.path_steps(0),
            field_specs: &field_specs,
            stop_surface: None,
        };
        (
            ParaxialSubView::new(
                0, // path_id
                0,
                0,
                &data,
                Vec3::new(0.0, 1.0, 0.0), // v = Y (phi=90°)
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
        let seq_sub_model = sequential_model.submodel(0).expect("Submodel not found.");
        let pseudo_marginal_ray = calc_pseudo_marginal_ray(
            seq_sub_model,
            sequential_model.surfaces(),
            sequential_model.placements(),
            sequential_model.path_surface_indices(0),
            sequential_model.path_beam_splitter_arms(0),
            sequential_model.path_steps(0),
        )
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
        let seq_sub_model = sequential_model.submodel(0).expect("Submodel not found.");
        let reverse_parallel_ray = ParaxialSubView::calc_reverse_parallel_ray(
            seq_sub_model,
            sequential_model.surfaces(),
            sequential_model.placements(),
            sequential_model.path_surface_indices(0),
            sequential_model.path_beam_splitter_arms(0),
            sequential_model.path_steps(0),
        )
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

    /// Regression test: aperture stop when the minimum a_k/y_k ratio is NOT at
    /// the surface with the smallest semi-diameter.
    ///
    /// Two flat refracting surfaces, finite object 10 mm away:
    ///   - Surface 1: air→glass (n=1.5), semi-diameter = 10 mm
    ///   - 15 mm glass gap
    ///   - Surface 2: glass→air,         semi-diameter = 14 mm
    ///
    /// Pseudo-marginal ray heights (h=0, u=1 at object):
    ///   y₁ = 10 mm  →  ratio = 10/10 = 1.00
    ///   y₂ = 20 mm  →  ratio = 14/20 = 0.70  ← aperture stop
    ///
    /// Surface 1 has the smaller semi-diameter, so an algorithm that simply
    /// finds the minimum semi-diameter would incorrectly return surface 1.
    #[test]
    fn test_aperture_stop_minimum_ratio_not_minimum_semi_diameter() {
        let air = n!(1.0);
        let glass = n!(1.5);
        let wavelengths = [0.5876];

        let gaps = vec![
            GapSpec {
                thickness: 10.0,
                refractive_index: air.clone(),
            },
            GapSpec {
                thickness: 15.0,
                refractive_index: glass,
            },
            GapSpec {
                thickness: 10.0,
                refractive_index: air,
            },
        ];
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Conic {
                semi_diameter: 10.0,
                radius_of_curvature: Float::INFINITY,
                conic_constant: 0.0,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Conic {
                semi_diameter: 14.0,
                radius_of_curvature: Float::INFINITY,
                conic_constant: 0.0,
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
        ];

        let sequential_model =
            SequentialModel::from_surface_specs(&gaps, &surfaces, &wavelengths, None).unwrap();
        let seq_sub_model = sequential_model.submodel(0).expect("Submodel not found.");
        let field_specs = vec![crate::FieldSpec::PointSource { x: 0.0, y: 0.0 }];

        let data = SubModelData {
            sequential_sub_model: seq_sub_model as &dyn SequentialSubModel,
            surfaces: sequential_model.surfaces(),
            placements: sequential_model.placements(),
            surface_indices: sequential_model.path_surface_indices(0),
            beam_splitter_arms: sequential_model.path_beam_splitter_arms(0),
            path_steps: sequential_model.path_steps(0),
            field_specs: &field_specs,
            stop_surface: None,
        };

        let view = ParaxialSubView::new(0, 0, 0, &data, Vec3::new(0.0, 1.0, 0.0), false).unwrap();

        assert_eq!(*view.aperture_stop(), 2);
    }

    /// When a user-specified stop surface is set on the model, ParaxialView
    /// must use it instead of the auto-derived heuristic result.
    #[test]
    fn user_specified_stop_overrides_auto() {
        // Convexplano lens: auto-derived stop is surface 1 (see test_aperture_stop).
        // Explicitly designate surface 2 and verify the paraxial view honours it.
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let gaps = vec![
            GapSpec {
                thickness: Float::INFINITY,
                refractive_index: air.clone(),
            },
            GapSpec {
                thickness: 5.3,
                refractive_index: nbk7,
            },
            GapSpec {
                thickness: 46.6,
                refractive_index: air,
            },
        ];
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Conic {
                semi_diameter: 12.5,
                radius_of_curvature: 25.8,
                conic_constant: 0.0,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Conic {
                semi_diameter: 12.5,
                radius_of_curvature: Float::INFINITY,
                conic_constant: 0.0,
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
        ];
        let seq =
            SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], Some(2)).unwrap();
        let field = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let pv = ParaxialView::new(&seq, &field, false).unwrap();
        let sub = pv.get(0, 0).unwrap();
        assert_eq!(*sub.aperture_stop(), 2);
    }

    #[test]
    fn paraxial_fno_equals_reciprocal_of_marginal_angle() {
        let (sub, _) = setup();
        // Last physical surface is index 2 (plano surface); n_image = 1.0 (air).
        let u_last = sub.marginal_ray().rays_at_surface(2)[0].angle;
        let expected = 1.0 / (2.0 * u_last);
        assert_abs_diff_eq!(sub.paraxial_fno(), expected, epsilon = 1e-6);
    }

    #[test]
    fn image_space_fno_equals_efl_over_epd() {
        let (sub, _) = setup();
        let expected = *sub.effective_focal_length() / (2.0 * sub.entrance_pupil().semi_diameter);
        assert_abs_diff_eq!(sub.image_space_fno(), expected, epsilon = 1e-6);
    }

    #[test]
    fn paraxial_view_two_path_model_has_subviews_for_both_paths() {
        use std::rc::Rc;

        use crate::examples::beam_splitter::two_path_model;
        use crate::specs::gaps::ConstantRefractiveIndex;

        let n_air = Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let model = two_path_model(n_air, &[0.5876e-3], 10.0, 10.0);
        let field = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let pv = ParaxialView::new(&model, &field, false).unwrap();

        assert!(pv.get_for_path(0, 0, 0).is_some(), "path 0 subview missing");
        assert!(pv.get_for_path(1, 0, 0).is_some(), "path 1 subview missing");
    }

    /// The aperture stop for each path must be computed from that path's own
    /// surfaces, not the store-indexed surface list. This test uses two paths
    /// with different irises in each arm: path 0 (Iris SD=5) and path 1
    /// (Iris SD=15), with a shared BS (SD=10). The BS is the most constraining
    /// surface on path 1, so its aperture stop must be at step 1 (BS), not
    /// step 2.
    #[test]
    fn paraxial_aperture_stop_uses_path_specific_surfaces() {
        use std::rc::Rc;

        use crate::{
            BeamSplitterPathKind, EulerAngles, GapSpec, PathSpec, PathSurfaceRef, Rotation3D,
            SurfaceSpec, Vec3, core::sequential_model::builder::SequentialModelBuilder,
            specs::gaps::ConstantRefractiveIndex,
        };

        let n_air = Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let bs_rotation =
            Rotation3D::IntrinsicPassiveRUF(EulerAngles((-45_f64 as Float).to_radians(), 0.0, 0.0));
        let gap_inf = || GapSpec {
            thickness: Float::INFINITY,
            refractive_index: n_air.clone(),
        };
        let gap_50 = || GapSpec {
            thickness: 50.0,
            refractive_index: n_air.clone(),
        };
        let img = || SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let iris = |sd: Float| SurfaceSpec::Iris {
            semi_diameter: sd,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };

        // Path 0: Object → BS(SD=10) → Iris1(SD=5) → Image_0
        // Store: Object(0), BS(1), Iris1(2), Image_0(3)
        let path_t = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::New(SurfaceSpec::Object),
                PathSurfaceRef::New(SurfaceSpec::BeamSplitter {
                    semi_diameter: 10.0,
                    rotation: bs_rotation,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
                PathSurfaceRef::New(iris(5.0)),
                PathSurfaceRef::New(img()),
            ],
            gaps: vec![gap_inf(), gap_50(), gap_50()],
            beam_splitter_arms: vec![BeamSplitterPathKind::Transmitting],
            stop_surface: None,
            wavelengths: vec![0.5876e-3],
        };

        // Path 1: Object(Shared) → BS(Shared) → Iris2(SD=15) → Image_1
        // Store: ..., Iris2(4), Image_1(5)
        let path_r = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::Shared(0),
                PathSurfaceRef::Shared(1),
                PathSurfaceRef::New(iris(15.0)),
                PathSurfaceRef::New(img()),
            ],
            gaps: vec![gap_inf(), gap_50(), gap_50()],
            beam_splitter_arms: vec![BeamSplitterPathKind::Reflecting],
            stop_surface: None,
            wavelengths: vec![0.5876e-3],
        };

        let model = SequentialModelBuilder::new()
            .paths(vec![path_t, path_r])
            .build()
            .unwrap()
            .model;

        let field = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let pv = ParaxialView::new(&model, &field, false).unwrap();

        // Path 0: BS(SD=10) at step 1, Iris1(SD=5) at step 2. Iris1 is the stop.
        let path0 = pv.get_for_path(0, 0, 0).expect("path 0 subview");
        assert_eq!(
            *path0.aperture_stop(),
            2,
            "path 0 stop should be Iris1 (step 2)"
        );

        // Path 1: BS(SD=10) at step 1, Iris2(SD=15) at step 2. BS is the stop.
        // Bug: if calc_aperture_stop uses store-indexed surfaces, it sees Iris1
        // (SD=5) at step 2 position and incorrectly identifies step 2 as the stop.
        let path1 = pv.get_for_path(1, 0, 0).expect("path 1 subview");
        assert_eq!(
            *path1.aperture_stop(),
            1,
            "path 1 stop should be BS (step 1)"
        );
    }

    /// A path's user-specified `stop_surface` is a *store* index (validated
    /// against the whole model's surface store at build time,
    /// `SequentialModel::validate_stop_surface`), but every downstream
    /// paraxial computation (`calc_marginal_ray`, the `None` branch's
    /// `calc_aperture_stop`) indexes the aperture stop by *step* position
    /// within that path's own traversal. For a path whose surfaces are all
    /// `New` in store order (e.g. path 0 of most models, where store index
    /// and step index coincide by construction), a mismatch between the two
    /// numberspaces is invisible. `wf_epi_microscope`'s emission path uses
    /// `Shared`/`ObjectLinkedTo` steps, so the two numberspaces genuinely
    /// diverge there, catching it.
    #[test]
    fn stop_surface_step_resolves_from_store_index_on_a_reordered_path() {
        use crate::examples::wf_epi_microscope::sequential_model;

        let model = sequential_model(n!(1.0), n!(1.5), &[0.488], &[0.520]);

        // path_emission's surface_indices are [5, 3, 2, 6, 7, 8] (synthesized
        // Object=5, Shared objective=3, Shared beam splitter=2, mirror=6,
        // tube lens=7, Image=8) — store index 3 (the objective, the value
        // configured via `stop_surface`) sits at step 1, not step 3.
        assert_eq!(model.path_surface_indices(1), &[5, 3, 2, 6, 7, 8]);
        assert_eq!(model.stop_surface_for_path(1), Some(3));

        let field_specs = vec![FieldSpec::PointSource { x: 0.0, y: 1.5 }];
        let view = ParaxialView::new(&model, &field_specs, false).unwrap();
        let sub = view.get_for_path(1, 0, 0).expect("path 1 subview");

        assert_eq!(
            *sub.aperture_stop(),
            1,
            "the objective (store index 3) is path 1's step 1, not step 3"
        );
    }

    #[test]
    fn test_lagrange_invariant_is_conserved() {
        let (view, _) = setup();

        // Convexplano lens has 3 gaps; invariant is stored for each (indices 0–2).
        let invariants = view.lagrange_invariants();
        assert!(invariants.len() >= 2);
        for &h in invariants {
            assert_abs_diff_eq!(h, invariants[1], epsilon = 1e-4);
        }
    }
}
