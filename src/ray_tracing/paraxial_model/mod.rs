use crate::ray_tracing::Surface;

/// Find the paraxial image distance from a surface or thin lens.
fn img_dist(obj_dist: f32, focal_length: f32) -> f32 {
    1.0 / (1.0 / focal_length - 1.0 / obj_dist)
}

/// Find the paraxial image locations of a sequence of surfaces in image space.
pub fn img_space_surf_locs(surfs: &[Surface]) -> Vec<f32> {    
    // TODO: Implement this function.
    Vec::new()
}
