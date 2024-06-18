/// A paraxial view into an optical system.

use std::cell::OnceCell;

use crate::core::Float;

pub struct ParaxialView {
    pseudo_marginal_ray: OnceCell<Float>,
}

impl ParaxialView {
    /// Create a new paraxial view of an optical system.
    pub fn new() -> Self {
        Self {
            pseudo_marginal_ray: OnceCell::new(),
        }
    }

    /// Compute the pseudo marginal ray.
    pub fn pseudo_marginal_ray(&self) -> &Float {
        self.pseudo_marginal_ray.get_or_init(|| 0.0)
    }
}
