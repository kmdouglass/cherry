/// Implementations of routines used to render the system in the UI.
use std::f32::consts::PI;

use wasm_bindgen::prelude::*;

use crate::math::vec3::Vec3;
use crate::surfaces::Surface;
use crate::SystemModel;

/// Returns a 3D bounding box of a set of points in the global coordinate system.
fn bounding_box(points: Vec<Vec3>) -> (Vec3, Vec3) {
    let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

    for point in points {
        if point.x() < min.x() {
            min.set_x(point.x());
        }
        if point.y() < min.y() {
            min.set_y(point.y());
        }
        if point.z() < min.z() {
            min.set_z(point.z());
        }

        if point.x() > max.x() {
            max.set_x(point.x());
        }
        if point.y() > max.y() {
            max.set_y(point.y());
        }
        if point.z() > max.z() {
            max.set_z(point.z());
        }
    }

    (min, max)
}

#[wasm_bindgen]
impl SystemModel {
    /// Returns point samples from the surfaces in the system.
    pub fn render(&self) -> JsValue {
        let mut samples: Vec<Vec3> = Vec::new();
        for surface in &self.surfaces {
            samples.extend(surface.sample_yz());
        }

        serde_wasm_bindgen::to_value(&samples).unwrap()
    }
}

impl Surface {
    /// Sample the surface in the local y,z plane, returning points in the global coordinate system.
    pub fn sample_yz(&self) -> Vec<Vec3> {
        // Skip object or image planes at infinity
        if let Self::ObjectOrImagePlane(surf) = self {
            if surf.pos.z().abs() == f32::INFINITY {
                return Vec::new();
            }
        }

        let diam = self.diam();

        // Sample the surface in in the y,z plane by creating uniformally spaced (0,y,z) coordinates
        let n = 100;
        let sample_points = Vec3::fan(n, diam / 2.0, PI / 2.0, 0.0);

        let mut samples = Vec::with_capacity(sample_points.len());
        for point in sample_points {
            let (sag, _) = match self {
                Self::ObjectOrImagePlane(surf) => surf.sag_norm(point),
                Self::RefractingCircularConic(surf) => surf.sag_norm(point),
                Self::RefractingCircularFlat(surf) => surf.sag_norm(point),
            };

            // Transform the sample into the global coordinate system.
            let sample = Vec3::new(point.x(), point.y(), sag);
            let rot_sample = self.rot_mat().transpose() * (sample + self.pos());

            samples.push(rot_sample);
        }

        samples
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pt_cloud_yz_object_plane_at_infinity() {
        let surf = Surface::new_obj_or_img_plane(f32::NEG_INFINITY, 4.0);
        let samples = surf.sample_yz();
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_pt_cloud_yz_image_plane_at_infinity() {
        let surf = Surface::new_obj_or_img_plane(f32::INFINITY, 4.0);
        let samples = surf.sample_yz();
        assert_eq!(samples.len(), 0);
    }
}
