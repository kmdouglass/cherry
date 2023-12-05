use crate::{Gap, Surface, SurfacePairIterator};

#[derive(Debug)]
pub struct SurfaceModel {
    gaps: Vec<Gap>,
    surfaces: Vec<Surface>,
}

impl SurfaceModel {
    pub fn new(surfaces: &[Surface]) -> SurfaceModel {
        // Iterate over SurfacePairs and convert to Surfaces and Gaps
        let mut gaps = Vec::new();
        let mut surfs = Vec::new();
        for pair in SurfacePairIterator::new(surfaces) {
            let (surf, gap) = pair.into();
            surfs.push(surf);
            gaps.push(gap);
        }

        // Add the image plane
        surfs.push(surfaces.last().unwrap().clone());

        Self {
            gaps,
            surfaces: surfs,
        }
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    pub fn gaps(&self) -> &[Gap] {
        &self.gaps
    }
}
