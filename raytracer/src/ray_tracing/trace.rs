use anyhow::Result;

use super::rays::Ray;
use super::Surface;

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
pub fn trace(surfaces: &[Surface], mut rays: Vec<Ray>, wavelength: f32) -> Vec<Vec<Result<Ray>>> {
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
            // Skip rays that have already terminated
            if ray.is_terminated() {
                results[s_ctr + 1].push(Err(anyhow::anyhow!("Ray terminated")));
                continue;
            }

            // Transform into coordinate system of the surface
            ray.transform(surf_2);

            // Find the ray intersection with the surface
            let (pos, norm) = match ray.intersect(surf_2, 1000) {
                Ok((pos, norm)) => (pos, norm),
                Err(e) => {
                    ray.terminate();
                    results[s_ctr + 1].push(Err(e));
                    continue;
                }
            };

            // Terminate ray if the intersection point is outside the clear aperture of the surface
            if surf_2.outside_clear_aperture(pos) {
                // Terminate the ray, but keep the results so that we can plot its last end point.
                ray.terminate();
            }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vec3::Vec3;
    use crate::ray_tracing::{ApertureSpec, FieldSpec, Ray};
    use crate::test_cases::{petzval_lens, planoconvex_lens_obj_at_inf};

    // Regression test for ray intersection that failed to converge in the Petzval lens
    //
    // The ray/surface intersection failed to converge because the tolerance Newton-Raphson method
    // was too strict. The algo was fixed by checking for relative error instead of absolute error.
    #[test]
    fn regression_test_petzval_lens() {
        let model = petzval_lens();
        let surfaces = model.surf_model.surfaces();
        let wavelength = 0.5876;
        let rays = vec![Ray::new(
            Vec3::new(-5.823648, -5.823648, -1.0),
            Vec3::new(-3.809699e-9, 0.087155744, 0.9961947),
        )
        .unwrap()];

        let results = trace(surfaces, rays, wavelength);

        // Check that there are no errors in the flattened results
        for result in results.into_iter().flatten() {
            assert!(result.is_ok());
        }
    }

    // Regression test for ray intersection that failed to converge in the Petzval lens
    //
    // The ray/surface intersection failed to converge because the tolerance Newton-Raphson method
    // was too strict. The algo was fixed by checking for relative error instead of absolute error.
    #[test]
    fn regression_test_planoconvex_outside_clear_aperture() {
        let (_, mut builder) = planoconvex_lens_obj_at_inf();

        // Aperture stop is the first surface with diameter of 25, so this overfills the entrance pupil.
        builder.aperture(ApertureSpec::EntrancePupilDiameter { diam: 26.0 });
        builder.fields(vec![FieldSpec::new(5.0)]);
        let model = builder.build().unwrap();

        let surfaces = model.surf_model.surfaces();
        let wavelength = 0.5876;
        let num_rays = 3;
        let fields = model.field_specs();
        let mut rays = Vec::with_capacity(num_rays * fields.len());

        for field in fields {
            let ray_fan = model
                .pupil_ray_fan(num_rays, f32::to_radians(90.0), field.angle().to_radians())
                .unwrap();

            rays.extend(ray_fan);
        }

        let results = trace(surfaces, rays, wavelength);

        // Check that there are no errors
        for result in results.into_iter().flatten() {
            assert!(result.is_ok());
        }
    }
}
