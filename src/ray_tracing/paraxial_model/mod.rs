use crate::math::mat2::{mat2, Mat2};
use crate::ray_tracing::{Gap, Surface, SurfacePair, SurfacePairIterator};

/// A paraxial model of an optical system.
/// 
/// The paraxial model comprises a sequence of ray transfer matrices (RTMs), one for each surface
/// and gap.
#[derive(Debug)]
pub struct ParaxialModel {
    gap_rtms: Vec<Mat2>,
    surf_rtms: Vec<Mat2>,
}

impl ParaxialModel {
    pub fn new(surfs: &[Surface]) -> Self {
        let mut gap_rtms = Vec::new();
        let mut surf_rtms = Vec::new();

        // The object plane RTM does not do anything.
        surf_rtms.push(Mat2::eye());

        for pair in SurfacePairIterator::new(surfs) {
            surf_rtms.push(pair.rtm());

            let (_, gap) = pair.into();
            gap_rtms.push(gap.rtm());
        }

        // The image plane RTM does not do anything either, but its refractive index might be
        // different from the previous surface, so force it to the identity matrix.
        let last_surf_rtm = surf_rtms.last_mut().unwrap();
        *last_surf_rtm = Mat2::eye();

        Self {
            gap_rtms,
            surf_rtms,
        }
    }
}

impl SurfacePair {
    /// Return the ray transfer matrix for the second surface in the pair.
    fn rtm(&self) -> Mat2 {
        let surf_0 = self.0;
        let surf_1 = self.1;

        let roc = surf_1.roc();
        let n_0 = surf_0.n();
        let n_1 = surf_1.n();

        let a = 1.0;
        let b = 0.0;
        let c = (n_0 - n_1) / (roc * n_1);
        let d = n_0 / n_1;

        mat2!(a, b, c, d)
    }
}

impl Gap {
    /// Return the ray transfer matrix for a gap.
    fn rtm(&self) -> Mat2 {
        let d = self.thickness();
        mat2!(1.0, d, 0.0, 1.0)
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
