use anyhow::Result;

use super::rays::Ray;
use crate::core::sequential_model::SequentialSubModelIter;

pub type TraceResults = Vec<Vec<Result<Ray>>>;

/// Trace a set of rays through a system using the technique of Spencer and
/// Murty, JOSA (1962).
///
/// The results are stored in a Vec, where the index is the surface number and
/// the value is a Vec of Result. If the ray intersected the surface
/// successfully, then the result contains the ray located at the intersection
/// point and with a (possibly) new direction as determined by the
/// surface interaction, e.g. refraction or reflection.
///
/// If the ray did not intersect the surface, or the intersection point did not
/// converge, then the result contains an error.
///
/// Index 0 of the results corresponds to the rays' initial states. By
/// convention, this is their locations and directions at the axial position z =
/// 0.
pub fn trace(
    sequential_submodel: &mut SequentialSubModelIter,
    mut rays: Vec<Ray>,
) -> Vec<Vec<Result<Ray>>> {
    // Pre-allocate the results. Include the initial ray positions as a "surface."
    let mut results: Vec<Vec<Result<Ray>>> = Vec::with_capacity(sequential_submodel.len() + 1);
    for _ in 0..sequential_submodel.len() + 1 {
        // +1 for the initial ray positions
        results.push(Vec::with_capacity(rays.len()));
    }

    // Add initial ray states to the results.
    for ray in &rays {
        results[0].push(Ok(ray.clone()));
    }

    // Iterate over all pairs of surfaces.
    for (ctr, step) in sequential_submodel.enumerate() {
        let (gap_0, surf, gap_1) = step;

        for ray in &mut rays {
            // Skip rays that have already terminated
            if ray.is_terminated() {
                results[ctr + 1].push(Err(anyhow::anyhow!("Ray terminated")));
                continue;
            }

            // Transform into coordinate system of the surface
            ray.transform(surf);

            // Find the ray intersection with the surface
            let (pos, norm) = match ray.intersect(surf, 1000) {
                Ok((pos, norm)) => (pos, norm),
                Err(e) => {
                    ray.terminate();
                    results[ctr + 1].push(Err(e));
                    continue;
                }
            };

            // Terminate ray if the intersection point is outside the clear aperture of the
            // surface
            if surf.outside_clear_aperture(pos) {
                // Terminate the ray, but keep the results so that we can plot its last end
                // point.
                ray.terminate();
            }

            // Displace the ray to the intersection point
            ray.displace(pos);

            // Redirect the ray due to surface interaction
            ray.redirect(&step, norm);

            // Transform back to the global coordinate system
            ray.i_transform(surf);

            results[ctr + 1].push(Ok(ray.clone()));
        }
    }
    results
}
