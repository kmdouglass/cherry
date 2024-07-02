/// Views represent calculations that are performed on a sequential model. They
/// can be used to calculate quantities such as paraxial ray paths, Gaussian
/// beam parameters, and full 3D ray paths. They are called *Views* because the
/// model properties themselves do not change under different views;
/// rather the results are just different ways of looking at the same model.
///
/// A View is a collection of subviews, each one of which is applied to a
/// `SequentialSubModel` in the optical system.
use std::any::Any;

pub mod paraxial;

#[derive(Debug, PartialEq)]
pub enum ViewType {
    Paraxial,
}

/// The order in which the views are initialized.
///
/// Dependencies between Views are handled manually because it is much simpler
/// than maintaining a depedency graph.
pub const VIEW_INIT_ORDER: [ViewType; 1] = [ViewType::Paraxial];

pub trait View {
    /// The name of the view.
    fn name(&self) -> &str;

    /// Returns the View as an `Any` reference.
    ///
    /// This is used by the system to downcast the View to its specific type.
    fn as_any(&self) -> &dyn Any;
}
