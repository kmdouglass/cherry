/// A paraxial view into an optical system.
use std::collections::HashMap;

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

#[derive(Debug)]
pub struct ParaxialView {
    pub subviews: HashMap<SubModelID, ParaxialSubView>,
}

#[derive(Debug, Serialize)]
pub struct ParaxialViewDescription {
    subviews: HashMap<SubModelID, ParaxialSubViewDescription>,
}

#[derive(Debug)]
pub struct ParaxialSubView {
    is_obj_space_telecentric: bool,

    aperture_stop: usize,
    back_focal_distance: Float,
    back_principal_plane: Float,
    effective_focal_length: Float,
    entrance_pupil: Pupil,
    front_focal_distance: Float,
    marginal_ray: ParaxialRayTraceResults,
}

/// A paraxial description of an optical system.
#[derive(Debug, Serialize)]
pub struct ParaxialSubViewDescription {
    aperture_stop: usize,
    back_focal_distance: Float,
    back_principal_plane: Float,
    effective_focal_length: Float,
    entrance_pupil: Pupil,
    front_focal_distance: Float,
    marginal_ray: ParaxialRayTraceResults,
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

impl ParaxialView {
    pub fn new(sequential_model: &SequentialModel, is_obj_space_telecentric: bool) -> Result<Self> {
        let subviews: Result<HashMap<SubModelID, ParaxialSubView>> = sequential_model
            .submodels()
            .iter()
            .map(|(id, submodel)| {
                let surfaces = sequential_model.surfaces();
                let axis = id.1;
                Ok((
                    *id,
                    ParaxialSubView::new(submodel, surfaces, axis, is_obj_space_telecentric)?,
                ))
            })
            .collect();

        Ok(Self {
            subviews: subviews?,
        })
    }

    pub fn describe(&self) -> ParaxialViewDescription {
        ParaxialViewDescription {
            subviews: self
                .subviews
                .iter()
                .map(|(id, subview)| (*id, subview.describe()))
                .collect(),
        }
    }
}

impl ParaxialSubView {
    /// Create a new paraxial view of an optical system.
    fn new(
        sequential_sub_model: &impl SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
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
        let effective_focal_length = Self::calc_effective_focal_length(&parallel_ray);

        let back_principal_plane =
            Self::calc_back_prinicpal_plane(surfaces, back_focal_distance, effective_focal_length)?;

        Ok(Self {
            is_obj_space_telecentric,

            aperture_stop,
            back_focal_distance,
            back_principal_plane,
            effective_focal_length,
            entrance_pupil,
            front_focal_distance,
            marginal_ray,
        })
    }

    fn describe(&self) -> ParaxialSubViewDescription {
        ParaxialSubViewDescription {
            aperture_stop: self.aperture_stop,
            back_focal_distance: self.back_focal_distance,
            back_principal_plane: self.back_principal_plane,
            effective_focal_length: self.effective_focal_length,
            entrance_pupil: self.entrance_pupil.clone(),
            front_focal_distance: self.front_focal_distance,
            marginal_ray: self.marginal_ray.clone(),
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

    pub fn effective_focal_length(&self) -> &Float {
        &self.effective_focal_length
    }

    pub fn entrance_pupil(&self) -> &Pupil {
        &self.entrance_pupil
    }

    pub fn front_focal_distance(&self) -> &Float {
        &self.front_focal_distance
    }

    pub fn is_obj_space_telecentric(&self) -> &bool {
        &self.is_obj_space_telecentric
    }

    pub fn marginal_ray(&self) -> &ParaxialRayTraceResults {
        &self.marginal_ray
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

        // Principal planes make no sense for surfaces without power
        if delta.is_infinite() {
            return Ok(Float::NAN);
        }

        // Find the z position of the last real surface
        let last_physical_surface_index =
            last_physical_surface(surfaces).ok_or(anyhow!("There are no physical surfaces"))?;
        let last_surface_z = surfaces[last_physical_surface_index].z();

        Ok(last_surface_z + delta)
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
            .get(&SubModelID(Some(0usize), Axis::Y))
            .expect("Submodel not found.");

        (
            ParaxialSubView::new(seq_sub_model, sequential_model.surfaces(), Axis::Y, false)
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
            .get(&SubModelID(Some(0usize), Axis::Y))
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
            .get(&SubModelID(Some(0usize), Axis::Y))
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
