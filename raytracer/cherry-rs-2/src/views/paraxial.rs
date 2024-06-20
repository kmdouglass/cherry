/// A paraxial view into an optical system.
use ndarray::{s, Array2, Array3};

use crate::core::{
    sequential_model::{Axis, SequentialSubModel, Surface},
    Float,
};
use crate::systems::SequentialModel;

/// A Nr x 2 array of paraxial rays.
///
/// Nr is the number of rays. The first column is the height of the ray at the
/// surface, and the second column is the paraxial angle of the ray at the
/// surface.
type ParaxialRays = Array2<Float>;

/// A Ns x Nr x 2 array of paraxial ray trace results.
///
/// Ns is the number of surfaces, and Nr is the number of rays. The first column
/// is the height of the ray at the surface, and the second column is the angle
/// of the ray at the surface.
type ParaxialRayTraceResults = Array3<Float>;

struct ParaxialSubView {
    pseudo_marginal_ray: ParaxialRayTraceResults,
}

impl ParaxialSubView {
    /// Create a new paraxial view of an optical system.
    pub fn new(
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
    ) -> Self {
        let pseudo_marginal_ray =
            Self::set_pseudo_marginal_ray(sequential_sub_model, surfaces, axis);

        Self {
            pseudo_marginal_ray: pseudo_marginal_ray,
        }
    }

    /// Compute the pseudo-marginal ray.
    pub fn set_pseudo_marginal_ray(
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
    ) -> ParaxialRayTraceResults {
        let ray_trace_results = Array3::zeros((1, 1, 2));
        ray_trace_results
    }

    fn trace(
        rays: ParaxialRays,
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
    ) -> ParaxialRayTraceResults {
        // TODO: Compute RTMs

        // Initialize the results array by assigning the input rays to the first
        // surface.
        let mut results = Array3::zeros((1, rays.shape()[0], 2));
        results.slice_mut(s![0, .., ..]).assign(&rays);

        // Iterate over the surfaces and compute the ray trace results.
        for step in sequential_sub_model.iter(surfaces) {}

        results
    }
}
