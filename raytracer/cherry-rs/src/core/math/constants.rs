/// Constants used by the mathematics module.

use crate::core::Float;

/// The tolerance to use when comparing geometrical values to zero.
/// 
/// The rationale for this value is as follows:
/// - The smallest possible feature size in optical systems is typically around the
///   wavelength of light, or 1 micron (1e-6 meters).
/// - Optical systems are specified in units from millimeters to meters.
/// - When specified in millimeters, we can expect distances up to a maximum of
///   10,000 mm. At this scale, 1e-10 = ~55 ULPs in 64-bit floating point arithmetic.
/// - When specified in meters, we can expect distances up to a maximum of 10 m. At
///   this scale, 1e-10 = ~56,000 ULPs.
/// - Most geometrical operations should only accumulate a few ULPs of error because
///   they are non-iterative.
///
/// So in the worst case of a system specified in millimeters with feature sizes up to
/// about 10,000 mm, a tolerance of 1e-10 is still 4 orders of magnitude smaller than
/// the smallest possible feature size, and affords a margin of 55 ULPs for rounding
/// errors.
pub const GEOM_ZERO_TOL: Float = 1e-10;
