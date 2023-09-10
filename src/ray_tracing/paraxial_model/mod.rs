use crate::math::mat2::{mat2, Mat2};
use crate::ray_tracing::{Gap, Surface, SurfacePair, SurfacePairIterator};

/// A ray transfer matrix (RTM) for a paraxial optical system component.
#[derive(Debug)]
enum RTM {
    Gap(usize, Mat2),
    Surf(usize, Mat2),
}

impl RTM {
    fn new_gap(id: usize, d: f32) -> Self {
        RTM::Gap(id, mat2!(1.0, d, 0.0, 1.0))
    }

    fn new_refr_curved_surf(id: usize, n_0: f32, n_1: f32, roc: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = (n_0 - n_1) / (roc * n_1);
        let d = n_0 / n_1;

        RTM::Surf(id, mat2!(a, b, c, d))
    }

    fn new_refr_flat_surf(id: usize, n_0: f32, n_1: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = 0.0;
        let d = n_0 / n_1;

        RTM::Surf(id, mat2!(a, b, c, d))
    }

    fn new_no_op_surf(id: usize) -> Self {
        RTM::Surf(id, Mat2::eye())
    }
}

/// A paraxial model of an optical system.
///
/// The paraxial model comprises a sequence of ray transfer matrices (RTMs), one for each surface
/// and gap.
#[derive(Debug)]
pub struct ParaxialModel {
    rtms: Vec<RTM>,
}

impl ParaxialModel {
    pub fn new(surfs: &[Surface]) -> Self {
        let mut rtms = Vec::new();

        // The object plane RTM does not do anything.
        rtms.push(RTM::new_no_op_surf(0));

        for (id, pair) in SurfacePairIterator::new(surfs).enumerate() {
            // The ray transfer matrix for the second surface in the pair.
            let surf_rtm = pair.rtm(id + 1);
            let (_, gap) = pair.into();
            
            rtms.push(gap.rtm(id));
            rtms.push(surf_rtm);
        }

        Self { rtms }
    }
}

impl SurfacePair {
    /// Return the ray transfer matrix for the second surface in the pair.
    fn rtm(&self, id: usize) -> RTM {
        let surf = self.1;
        let n_0 = self.0.n();
        let n_1 = self.1.n();
        
        match surf {
            Surface::RefractingCircularConic(surf) => {
                RTM::new_refr_curved_surf(id, n_0, n_1, surf.roc)
            }
            Surface::RefractingCircularFlat(surf) => {
                RTM::new_refr_flat_surf(id, n_0, n_1)
            }
            Surface::ObjectOrImagePlane(surf) => {
                RTM::new_no_op_surf(id)
            }
            Surface::Stop(surf) => {
                RTM::new_no_op_surf(id)
            }
        }
    }
}

impl Gap {
    /// Return the ray transfer matrix for a gap.
    fn rtm(&self, id: usize) -> RTM {
        let d = self.thickness();
        RTM::new_gap(id, d)
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
