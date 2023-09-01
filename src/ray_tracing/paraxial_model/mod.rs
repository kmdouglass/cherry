/// Find the paraxial image distance from a surface or thin lens.
pub fn img_dist(obj_dist: f32, focal_length: f32) -> f32 {
    1.0 / (1.0 / focal_length - 1.0 / obj_dist)
}
