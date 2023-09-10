use crate::math::mat2::{mat2, Mat2};
use crate::math::vec2::Vec2;
use crate::ray_tracing::{Gap, Surface, SurfacePair, SurfacePairIterator};

/// A paraxial element in an optical system.
/// 
/// The ray transfer matrices (RTM) are stored with each element to facilitate ray tracing.
/// 
/// A surface height is the distance from the optical axis to its greatest extent.
#[derive(Debug)]
enum ParaxSurf {
    Gap{ id: usize, rtm: Mat2 },
    Surf{ id: usize, height: f32, rtm: Mat2},
}

impl ParaxSurf {
    fn new_gap(id: usize, d: f32) -> Self {
        ParaxSurf::Gap{id, rtm: mat2!(1.0, d, 0.0, 1.0)}
    }

    fn new_refr_curved_surf(id: usize, height: f32, n_0: f32, n_1: f32, roc: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = (n_0 - n_1) / (roc * n_1);
        let d = n_0 / n_1;

        ParaxSurf::Surf{id, height, rtm: mat2!(a, b, c, d)}
    }

    fn new_refr_flat_surf(id: usize, height: f32, n_0: f32, n_1: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = 0.0;
        let d = n_0 / n_1;

        ParaxSurf::Surf{ id, height, rtm: mat2!(a, b, c, d)}
    }

    fn new_no_op_surf(id: usize, height: f32) -> Self {
        ParaxSurf::Surf{id, height, rtm: Mat2::eye()}
    }
}

/// The result of tracing a ray through a paraxial model.
#[derive(Debug)]
struct ParaxTraceResult {
    ray: Vec2,
    surf_height: f32,
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
        if let ParaxSurf::Surf { id: _, height, rtm: _ } = obj_surf {
            results.push(ParaxTraceResult {ray: ray.clone(), surf_height: *height});
        };

        for surf in &self.parax_surfs {
            match surf {
                ParaxSurf::Gap{ id: _, rtm} => {
                    ray = rtm * &ray;
                }
                ParaxSurf::Surf{ id: _, height, rtm} => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {ray: ray.clone(), surf_height: *height});
                }
            }
        }

        results
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

/// Find the paraxial image distance of an object from a surface.
fn img_dist_surf(obj_dist: f32, roc: f32, n_o: f32, n_i: f32) -> f32 {
    n_i / ((n_i - n_o) / roc - n_o / obj_dist)
}

/// Find the paraxial image location from an object point through a sequence of surfaces.
///
/// # Arguments
/// * `obj_dist` - The distance from the object to the first surface.
/// * `n_o` - The refractive index of the medium between the object and the first surface.
/// * `surfs` - A sequence of surfaces to image through.
///
/// # Returns
/// * The distance from the last surface to the image.
pub fn img_dist_multi_surf(obj_dist: f32, n_o: f32, surfs: &[Surface]) -> f32 {
    let surf_init = surfs.first().unwrap();
    let mut img_dist = img_dist_surf(obj_dist, surf_init.roc(), n_o, surf_init.n());
    let mut obj_dist = 0.0;

    for pair in SurfacePairIterator::new(surfs) {
        obj_dist = pair.axial_dist() - img_dist;

        let surf = pair.0;
        let next_surf = pair.1;

        img_dist = img_dist_surf(obj_dist, next_surf.roc(), surf.n(), next_surf.n());
    }
    img_dist
}
