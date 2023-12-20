/// Algorithms for sampling from objects such as surfaces and planes.

use std::f32::consts::PI;

use crate::math::vec2::Vec2;

/// Samples a circle using a square grid.
/// 
/// # Arguments
/// * `radius` - The radius of the circle.
/// * `scale` - The scale of the grid. For example, a scale of 1.0 will sample the circle at every
///      pair of integer coordinates, while a scale of 0.5 will sample the circle at every pair of
///      half-integer coordinates.
pub (crate) fn sample_circle_sq_grid(radius: f32, scale: f32) -> Vec<Vec2> {
    // Upper bound is established by the Gauss Circle Problem.
    let r_over_s = radius / scale;
    let num_samples = (PI * r_over_s * r_over_s + 9f32 * r_over_s).ceil() as usize;

    // Bounding box search.
    let mut samples = Vec::with_capacity(num_samples);
    let mut x = -radius;
    while x <= radius {
        let mut y = -radius;
        while y <= radius {
            if x * x + y * y <= radius * radius {
                samples.push(Vec2::new(x, y));
            }
            y += scale;
        }
        x += scale;
    }
    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_circle_sq_grid_unit_circle() {
        let samples = sample_circle_sq_grid(1.0, 1.0);
        assert_eq!(samples.len(), 5);
    }

    #[test]
    fn test_sample_circle_sq_grid_radius_2_scale_2() {
        let samples = sample_circle_sq_grid(2.0, 2.0);
        assert_eq!(samples.len(), 5);
    }

    #[test]
    fn test_sample_circle_sq_grid_radius_2_scale_1() {
        let samples = sample_circle_sq_grid(2.0, 1.0);
        assert_eq!(samples.len(), 13);
    }
}
