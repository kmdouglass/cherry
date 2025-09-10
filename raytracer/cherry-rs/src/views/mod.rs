/// Views represent calculations that are performed on a sequential model. They
/// can be used to calculate quantities such as paraxial ray paths, Gaussian
/// beam parameters, and full 3D ray paths. They are called *Views* because the
/// model properties themselves do not change under different views;
/// rather the results are just different ways of looking at the same model.
///
/// A View can be a collection of subviews, each one of which is applied to a
/// `SequentialSubModel` in the optical system.
pub mod components;
pub mod cross_section;
pub mod cutaway;
pub mod paraxial;
pub mod ray_trace_3d;
