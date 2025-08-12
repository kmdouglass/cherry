/// This module contains mathematical operations and types used in the
/// raytracer.
///
/// The direction of dependencies is as follows:
/// - linalg depends on vec3
/// - geometry depends on linalg
pub(super) mod complex;
pub(crate) mod constants;
pub(crate) mod geometry;
pub(crate) mod linalg;
pub(crate) mod vec2;
pub(crate) mod vec3;

pub(super) use complex::Complex;
