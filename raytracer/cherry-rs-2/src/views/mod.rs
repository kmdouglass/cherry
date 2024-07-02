/// Views represent calculations that are performed on a sequential model. They
/// can be used to calculate quantities such as paraxial ray paths, Gaussian
/// beam parameters, and full 3D ray paths. They are called *Views* because the
/// model properties themselves do not change under different views;
/// rather the results are just different ways of looking at the same model.
///
/// A View is a collection of subviews, each one of which is applied to a
/// `SequentialSubModel` in the optical system.
use std::any::Any;

use crate::core::sequential_model::SequentialModel;

pub mod paraxial;

pub trait View {
    /// Initializes the view with the given `SequentialModel`.
    fn init(&mut self, sequential_model: &SequentialModel);

    /// Returns the name of the view.
    ///
    /// Note that every View known to the system must have a unique name.
    fn name(&self) -> &str;

    /// Returns the View as an `Any` reference.
    ///
    /// This is used by the system to downcast the View to its specific type.
    fn as_any(&self) -> &dyn Any;

    /// Returns the dependencies of the View.
    ///
    /// Views can depend on the results of other Views. This method provides a
    /// way for a View to specify which other Views it depends on to the system.
    ///
    /// If there are no dependencies, an empty vector should be returned.
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
}
