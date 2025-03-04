use std::collections::HashMap;

use super::rays::Ray;
use crate::core::sequential_model::SequentialSubModelIter;

/// A set of rays traced through an optical system.
///
/// # Attributes
/// - *rays*: A num. of rays x num. of surfaces matrix of rays representing the
///   surface intersection points.
/// - *terminated*: A num. of rays vector containing the surface indexes where
///   any rays have terminated. `0` means the ray has not terminated.
/// - *reason_for_termination*: A hashmap containing the reason for termination.
/// - *num_surfaces*: The number of surfaces in the optical system.
#[derive(Debug)]
pub struct RayBundle {
    rays: Vec<Ray>,
    terminated: Vec<usize>,
    reason_for_termination: HashMap<usize, String>,
    num_surfaces: usize,
}

/// Trace a set of rays through a system using the technique of Spencer and
/// Murty, JOSA (1962).
pub fn trace(sequential_submodel: &mut SequentialSubModelIter, mut rays: Vec<Ray>) -> RayBundle {
    // Pre-allocate the results bundle. Include the initial ray positions as a
    // "surface."
    let mut terminated = vec![0; rays.len()];
    let mut reason_for_termination: HashMap<usize, String> = HashMap::new();
    let num_surfaces = sequential_submodel.len() + 1; // +1 for the initial ray positions
    let mut ray_bundle = initialize_bundle(&rays, num_surfaces);

    for (ctr, step) in sequential_submodel.enumerate() {
        let (_, surf, _) = step;
        let surface_id = ctr + 1;

        // Copy the ray states into here after they have been traced through the surface
        let rays_at_surface = rays_at_surface_mut(&mut ray_bundle, surface_id, num_surfaces);

        for (ray_id, ray) in rays.iter_mut().enumerate() {
            // Skip rays that have already terminated
            if is_terminated(ray_id, &terminated) {
                continue;
            }

            // Transform into coordinate system of the surface
            ray.transform(surf);

            // Find the ray intersection with the surface.
            // Errors if the intersection point does not converge.
            let (pos, norm) = match ray.intersect(surf, 1000) {
                Ok((pos, norm)) => (pos, norm),
                Err(e) => {
                    terminated[ray_id] = surface_id;
                    reason_for_termination.insert(ray_id, e.to_string());
                    continue;
                }
            };

            // Terminate ray if intersection is outside the clear aperture of surface
            if surf.outside_clear_aperture(pos) {
                terminated[ray_id] = surface_id;
                reason_for_termination.insert(ray_id, "Ray outside clear aperture".to_string());
                continue;
            }

            // Displace the ray to the intersection point
            ray.displace(pos);

            // Redirect the ray due to surface interaction
            ray.redirect(&step, norm);

            // Transform back to the global coordinate system
            ray.i_transform(surf);

            rays_at_surface[ray_id] = ray.clone();
        }
    }
    RayBundle {
        rays: ray_bundle,
        terminated,
        reason_for_termination,
        num_surfaces,
    }
}

impl RayBundle {
    /// Returns the rays traced through the system.
    pub fn rays(&self) -> &[Ray] {
        &self.rays
    }

    /// Returns the rays that have terminated.
    ///
    /// The index of the terminated ray corresponds to the surface index where
    /// the ray terminated.
    pub fn terminated(&self) -> &[usize] {
        &self.terminated
    }

    /// Returns the reason for termination of the rays.
    pub fn reason_for_termination(&self) -> &HashMap<usize, String> {
        &self.reason_for_termination
    }

    /// Returns the number of surfaces in the optical system.
    pub fn num_surfaces(&self) -> usize {
        self.num_surfaces
    }
}

/// Initializes a ray bundle for the start of a ray trace.
///
/// # Arguments
/// - *rays*: A slice of the ray states at the initial surface.
/// - *num_surfaces*: The number of surfaces in the optical system.
fn initialize_bundle(initial_rays: &[Ray], num_surfaces: usize) -> Vec<Ray> {
    let num_rays = initial_rays.len();
    let mut bundle: Vec<Ray> = Ray::new_bundle(num_rays * num_surfaces);

    // Copy the initial ray states to the first surface
    for (ray_id, ray) in initial_rays.iter().enumerate() {
        bundle[ray_id] = ray.clone();
    }
    bundle
}

fn is_terminated(ray_id: usize, terminated: &[usize]) -> bool {
    terminated[ray_id] != 0
}

/// Returns the set of rays at a given surface index.
fn rays_at_surface_mut(bundle: &mut [Ray], surface_id: usize, num_surfaces: usize) -> &mut [Ray] {
    let start = surface_id * bundle.len() / num_surfaces;
    let end = (surface_id + 1) * bundle.len() / num_surfaces;
    &mut bundle[start..end]
}

#[cfg(test)]
mod test {
    use crate::core::math::vec3::Vec3;

    use super::*;

    #[test]
    fn test_initialize_bundle() {
        let rays = vec![
            Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)).unwrap(),
            Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)).unwrap(),
        ];
        let num_surfaces = 3;
        let bundle = initialize_bundle(&rays, num_surfaces);
        assert_eq!(bundle.len(), rays.len() * num_surfaces);
    }

    #[test]
    fn test_is_terminated() {
        let terminated = vec![0, 1, 0, 2];
        assert_eq!(is_terminated(0, &terminated), false);
        assert_eq!(is_terminated(1, &terminated), true);
        assert_eq!(is_terminated(2, &terminated), false);
        assert_eq!(is_terminated(3, &terminated), true);
    }

    #[test]
    fn test_rays_at_surface_mut() {
        let mut bundle = vec![
            Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)).unwrap(),
            Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)).unwrap(),
            Ray::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 0.0)).unwrap(),
            Ray::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 0.0)).unwrap(),
            Ray::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(0.0, 0.0, 1.0)).unwrap(),
            Ray::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(0.0, 0.0, 1.0)).unwrap(),
        ];
        let rays_per_surface = 2;
        let num_surfaces = bundle.len() / rays_per_surface;
        let surface_id = 1;
        let rays_at_surface = rays_at_surface_mut(&mut bundle, surface_id, num_surfaces);

        assert_eq!(rays_at_surface.len(), rays_per_surface);

        for ray in rays_at_surface {
            assert_eq!(ray.x(), 1.0);
            assert_eq!(ray.y(), 1.0);
            assert_eq!(ray.z(), 1.0);
            assert_eq!(ray.k(), 0.0);
            assert_eq!(ray.l(), 1.0);
            assert_eq!(ray.m(), 0.0);
        }
    }
}
