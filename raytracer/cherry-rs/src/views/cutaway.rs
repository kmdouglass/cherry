use std::collections::HashMap;

use serde::Serialize;

use crate::{
    SequentialModel,
    core::{Float, PI, math::vec3::Vec3, sequential_model::Surface},
};

/// A cutaway view through a center transverse plane of a sequential model.
///
/// The cutaway view is a 2D representation of the surfaces in the y-z plane.
#[derive(Debug, Clone, Serialize)]
pub struct CutawayView {
    pub path_samples: HashMap<usize, Vec<Vec3>>,
    pub semi_diameters: HashMap<usize, Float>,
    pub surface_types: HashMap<usize, String>,
}

impl CutawayView {
    pub fn new(sequential_model: &SequentialModel, num_samples_per_surface: usize) -> CutawayView {
        let largest_semi_diameter = sequential_model.largest_semi_diameter();

        let mut path_samples = HashMap::new();
        let mut semi_diameters = HashMap::new();
        let mut surface_types = HashMap::new();
        for (i, surface) in sequential_model.surfaces().iter().enumerate() {
            let samples = surface.sample_yz(num_samples_per_surface, largest_semi_diameter);
            path_samples.insert(i, samples);

            semi_diameters.insert(i, surface.semi_diameter());
            surface_types.insert(i, surface.to_string());
        }

        CutawayView {
            path_samples,
            semi_diameters,
            surface_types,
        }
    }
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
        let sample_points = Vec3::fan(num_samples, semi_diameter, 0.0, PI / 2.0, 0.0, 0.0);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Float;
    use crate::examples::convexplano_lens::sequential_model;
    use crate::n;

    #[test]
    fn test_cutaway_view() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let sequential_model = sequential_model(air, nbk7, &wavelengths);
        let cutaways = CutawayView::new(&sequential_model, 10);

        assert_eq!(cutaways.path_samples.len(), 4);
        assert_eq!(cutaways.path_samples[&0].len(), 0); // Object is at infinity
        assert_eq!(cutaways.path_samples[&1].len(), 10);
        assert_eq!(cutaways.path_samples[&2].len(), 10);
        assert_eq!(cutaways.path_samples[&3].len(), 10);

        assert_eq!(cutaways.surface_types[&0], "Object");
        assert_eq!(cutaways.surface_types[&1], "Conic");
        assert_eq!(cutaways.surface_types[&2], "Conic");
        assert_eq!(cutaways.surface_types[&3], "Image");
    }
}
