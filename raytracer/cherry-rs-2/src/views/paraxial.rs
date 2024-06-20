/// A paraxial view into an optical system.
use std::cell::OnceCell;

use ndarray::Array3;

use crate::core::Float;
use crate::systems::SequentialModel;

/// A Ns x Nr x 2 array of ray trace results.
///
/// Ns is the number of surfaces, and Nr is the number of rays. The first column
/// is the height of the ray at the surface, and the second column is the angle
/// of the ray at the surface.
type ParaxialRayTraceResults = Array3<Float>;

pub struct ParaxialView<'a> {
    sequential_model: &'a SequentialModel,
    pseudo_marginal_ray: OnceCell<ParaxialRayTraceResults>,
}

impl<'a> ParaxialView<'a> {
    /// Create a new paraxial view of an optical system.
    pub fn new(sequential_model: &'a SequentialModel) -> Self {
        Self {
            sequential_model,
            pseudo_marginal_ray: OnceCell::new(),
        }
    }

    /// Compute the pseudo marginal ray.
    pub fn pseudo_marginal_ray(&self) -> &ParaxialRayTraceResults {
        self.pseudo_marginal_ray.get_or_init(|| {
            let ray_trace_results = Array3::zeros((1, 1, 2));
            ray_trace_results
        })
    }
}
