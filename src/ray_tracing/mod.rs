pub mod rays;

use anyhow::Result;

use crate::surfaces;
use rays::Ray;

/// Trace a set of rays through a system using the technique of Spencer and Murty, JOSA (1962).
///
/// The results are stored in a Vec, where the index is the surface number and the value is a Vec of
/// Result. If the ray intersected the surface successfully, then the result contains the ray
/// located at the intersection point and with a (possibly) new direction as determined by the
/// surface interaction, e.g. refraction or reflection.
/// 
/// If the ray did not intersect the surface, or the intersection point did not converge, then the
/// result contains an error.
/// 
/// Index 0 of the results corresponds to the rays' initial states. By convention, this is their
/// locations and directions at the axial position z = 0.
pub fn ray_trace(
    surfaces: &[surfaces::Surface],
    mut rays: Vec<Ray>,
    wavelength: f32,
) -> Vec<Vec<Result<Ray>>> {
    // Pre-allocate the results. Include the initial ray positions as a "surface."
    let mut results: Vec<Vec<Result<Ray>>> = Vec::with_capacity(surfaces.len() + 1);
    for i in 0..surfaces.len() {
        results.push(Vec::with_capacity(rays.len()));
    }

    // Add initial ray states to the results.
    for ray in &rays {
        results[0].push(Ok(ray.clone()));
    }

    // Iterate over all pairs of surfaces.
    for (s_ctr, surface_pair) in surfaces.windows(2).enumerate() {
        let (surf_1, surf_2) = (&surface_pair[0], &surface_pair[1]);

        for ray in &mut rays {
            // Transform into coordinate system of the surface
            ray.transform(surf_2);

            // Find the ray intersection with the surface
            let (pos, norm) = match ray.intersect(surf_2, 1e-6, 1000) {
                Ok((pos, norm)) => (pos, norm),
                Err(e) => {
                    results[s_ctr + 1].push(Err(e));
                    continue;
                }
            };

            // Displace the ray to the intersection point
            ray.displace(pos);

            // Redirect the ray due to surface interaction
            ray.redirect(surf_1, surf_2, norm);

            // Transform back to the global coordinate system
            ray.i_transform(surf_2);

            results[s_ctr + 1].push(Ok(ray.clone()));
        }
    }

    results
}
