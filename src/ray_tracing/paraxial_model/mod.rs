use std::f32::consts::PI;

use anyhow::{bail, Result};

use crate::math::mat2::{mat2, Mat2};
use crate::math::vec2::Vec2;
use crate::ray_tracing::{Gap, Surface, SurfacePair, SurfacePairIterator};


/// The initial angle of the ray in radians to find the entrance pupil.
const INIT_ANGLE: f32 = 5.0 * PI / 180.0;

/// The initial radius of the ray to find the entrance pupil.
const INIT_RADIUS: f32 = 1.0;

/// A paraxial element in an optical system.
/// 
/// The ray transfer matrices (RTM) are stored with each element to facilitate ray tracing.
/// 
/// A surface radius is the distance from the optical axis to its greatest extent.
#[derive(Debug)]
enum ParaxSurf {
    Gap{ id: usize, rtm: Mat2 },
    Surf{ id: usize, radius: f32, rtm: Mat2},
}

impl ParaxSurf {
    fn new_gap(id: usize, d: f32) -> Self {
        ParaxSurf::Gap{id, rtm: mat2!(1.0, d, 0.0, 1.0)}
    }

    fn new_refr_curved_surf(id: usize, radius: f32, n_0: f32, n_1: f32, roc: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = (n_0 - n_1) / (roc * n_1);
        let d = n_0 / n_1;

        ParaxSurf::Surf{id, radius, rtm: mat2!(a, b, c, d)}
    }

    fn new_refr_flat_surf(id: usize, radius: f32, n_0: f32, n_1: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = 0.0;
        let d = n_0 / n_1;

        ParaxSurf::Surf{ id, radius, rtm: mat2!(a, b, c, d)}
    }

    fn new_no_op_surf(id: usize, radius: f32) -> Self {
        ParaxSurf::Surf{id, radius, rtm: Mat2::eye()}
    }
}

/// The result of tracing a ray through a paraxial model.
#[derive(Debug)]
struct ParaxTraceResult {
    ray: Vec2,
    surf_radius: f32,
}

/// A paraxial model of an optical system.
///
/// The paraxial model comprises a sequence of ray transfer matrices (RTMs), one for each surface
/// and gap.
#[derive(Debug)]
pub struct ParaxialModel {
    parax_surfs: Vec<ParaxSurf>,
}

impl ParaxialModel {
    pub fn new(surfs: &[Surface]) -> Self {
        let obj_plane_radius = surfs.first().unwrap().diam() / 2.0;
        let mut rtms = Vec::new();

        // The object plane RTM does not do anything.
        rtms.push(ParaxSurf::new_no_op_surf(0, obj_plane_radius));

        for (id, pair) in SurfacePairIterator::new(surfs).enumerate() {
            // The ray transfer matrix for the second surface in the pair.
            let surf_rtm = pair.parax_surf(id + 1);
            let (_, gap) = pair.into();
            
            rtms.push(gap.parax_surf(id));
            rtms.push(surf_rtm);
        }

        Self { parax_surfs: rtms }
    }

    fn trace(&self, mut ray: Vec2) -> Vec<ParaxTraceResult> {
        let num_surfs = self.parax_surfs.len() / 2 + 1;
        let mut results = Vec::with_capacity(num_surfs);

        // Save the object plane ray.
        let obj_surf = self.parax_surfs.first().unwrap();
        if let ParaxSurf::Surf { id: _, radius: height, rtm: _ } = obj_surf {
            results.push(ParaxTraceResult {ray: ray.clone(), surf_radius: *height});
        };

        // Trace the ray through the paraxial model and save the results at each surface.
        for surf in &self.parax_surfs {
            match surf {
                ParaxSurf::Gap{ id: _, rtm} => {
                    ray = rtm * &ray;
                }
                ParaxSurf::Surf{ id: _, radius, rtm} => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {ray: ray.clone(), surf_radius: *radius});
                }
            }
        }

        results
    }

    /// Find the index of the surface that is the aperture stop of the paraxial model.
    pub fn find_aperture_stop(&self) -> Result<usize> {
        let init_ray = self.init_ray()?;
        let results = self.trace(init_ray);

        // Find the ID of the surface with the smallest ratio of surface radius to ray height.
        let mut min_ratio = f32::MAX;
        let mut min_id = 0;
        for (id, result) in results.iter().enumerate() {
            let ratio = result.surf_radius / result.ray.y();
            if ratio < min_ratio {
                min_ratio = ratio;
                min_id = id;
            }
        }

        Ok(min_id)
    }

    /// Find the initial ray to trace through the paraxial model.
    /// 
    /// If the object is at infinity, the initial ray is parallel to, but no colinear with, the
    /// optical axis. Otherwise, it starts on the axis with a small angle.
    fn init_ray(&self) -> Result<Vec2> {
        // Get the first gap (second element) in the paraxial model, which is the object space.
        let obj_space_dist = if let ParaxSurf::Gap { id: _, rtm } = self.parax_surfs[1] {
            rtm[0][1]
        } else {
            bail!("The second element in the paraxial model must be the object space gap.");
        };

        if obj_space_dist.is_infinite() {
            Ok(Vec2::new(INIT_RADIUS, 0.0))
        } else {
            Ok(Vec2::new(0.0, INIT_ANGLE))
        }
    }
}

impl SurfacePair {
    /// Return the paraxial surface equivalent for the second surface in the pair.
    fn parax_surf(&self, id: usize) -> ParaxSurf {
        let surf = self.1;
        let n_0 = self.0.n();
        let n_1 = self.1.n();
        
        match surf {
            Surface::RefractingCircularConic(surf) => {
                ParaxSurf::new_refr_curved_surf(id, surf.diam / 2.0,  n_0, n_1, surf.roc)
            }
            Surface::RefractingCircularFlat(surf) => {
                ParaxSurf::new_refr_flat_surf(id, surf.diam / 2.0, n_0, n_1)
            }
            Surface::ObjectOrImagePlane(surf) => {
                ParaxSurf::new_no_op_surf(id, surf.diam / 2.0)
            }
            Surface::Stop(surf) => {
                ParaxSurf::new_no_op_surf(id, surf.diam / 2.0)
            }
        }
    }
}

impl Gap {
    /// Return the ray transfer matrix for a gap.
    fn parax_surf(&self, id: usize) -> ParaxSurf {
        let d = self.thickness();
        ParaxSurf::new_gap(id, d)
    }
}
