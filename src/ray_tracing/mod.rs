pub(crate) mod rays;

use crate::surfaces;
use rays::Ray;

pub(crate) fn ray_trace(surfaces: &[surfaces::Surface], rays: Vec<Ray>, wavelength: f32) {
    // Iterate over all pairs of surfaces
    for surface_pair in surfaces.windows(2) {
        let (surf_1, surf_2) = (&surface_pair[0], &surface_pair[1]);
        for ray in &rays {
            // Transform into coordinate system of the surface
            let ray_1 = ray.transform(surf_2);

            // Find the ray intersection with the surface
            let (pos, norm) = ray.intersect(&surf_2, 1e-6, 1000).unwrap();

            // Redirect the ray
            let dir_2 = ray.redirect(&surf_2, norm, surf_1.n(), surf_2.n());

            // Transform back to the global coordinate system
            let ray_2 = ray.i_transform(surf_2);
        }
    }
}
