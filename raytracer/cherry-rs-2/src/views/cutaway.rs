use std::collections::HashMap;

use crate::{
    core::{math::vec3::Vec3, sequential_model::Surface, Float, PI},
    SequentialModel,
};

pub fn cutaway_view(
    sequential_model: &SequentialModel,
    num_samples_per_surface: usize,
) -> HashMap<usize, Vec<Vec3>> {
    let largest_semi_diameter = sequential_model.largest_semi_diameter();

    let mut cutaways = HashMap::new();
    for (i, surface) in sequential_model.surfaces().iter().enumerate() {
        let samples = surface.sample_yz(num_samples_per_surface, largest_semi_diameter);
        cutaways.insert(i, samples);
    }
    cutaways
}

impl Surface {
    /// Determine sequential point samples on the surface in the y-z plane.
    pub fn sample_yz(&self, num_samples: usize, default_semi_diameter: Float) -> Vec<Vec3> {
        // Skip object or image planes at infinity
        match self {
            Self::Object(_) => {
                if self.pos().z().abs() == Float::INFINITY {
                    return Vec::new();
                }
            }
            Self::Image(_) => {
                if self.pos().z().abs() == Float::INFINITY {
                    return Vec::new();
                }
            }
            _ => {}
        }

        // Use the default semi-diameter for object, image, and probe planes because
        // they have no size.
        let semi_diameter = match self {
            Self::Object(_) => default_semi_diameter,
            Self::Image(_) => default_semi_diameter,
            Self::Probe(_) => default_semi_diameter,
            _ => self.semi_diameter(),
        };

        // Sample the surface in in the y,z plane by creating uniformally spaced (0,y,z)
        // coordinates
        let sample_points = Vec3::fan(num_samples, semi_diameter, PI / 2.0, 0.0, 0.0, 0.0);

        let mut sample: Vec3;
        let mut rot_sample: Vec3;
        let mut samples = Vec::with_capacity(sample_points.len());
        for point in sample_points {
            let (sag, _) = self.sag_norm(point);

            // Transform the sample into the global coordinate system.
            sample = Vec3::new(point.x(), point.y(), sag);
            rot_sample = self.rot_mat().transpose() * (sample + self.pos());

            samples.push(rot_sample);
        }

        samples
    }
}
