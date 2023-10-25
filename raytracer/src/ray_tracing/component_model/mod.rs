use std::collections::HashSet;

use anyhow::{bail, Result};

use crate::ray_tracing::{Surface, SurfacePair, SurfacePairIterator};

/// A component is a part of an optical system that can interact with light rays.
///
/// Components come in two types: surfaces and elements. Surfaces are just single, isolated
/// surfaces such as optical interfaces to a different medium. Elements are the most basic compound 
/// optical components and are represented as sets of surfaces pairs.
///
/// To avoid copying data, only indexes are stored from the sequential models are stored.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Component {
    Element {
        surf_idxs: (usize, usize),
    },
    Surface {
        surf_idx: usize,
    },
}

impl Component {
    pub fn new(surf_idxs: Vec<usize>) -> Result<Self> {
        if surf_idxs.is_empty() {
            bail!("Component must have at least one surface");
        }

        if surf_idxs.len() == 1 {
            return Ok(Self::Surface { surf_idx: surf_idxs[0] });
        }

        if surf_idxs.len() == 2 {
            return Ok(Self::Element { surf_idxs: (surf_idxs[0], surf_idxs[1]) });
        }

        bail!("Component must have at most two surfaces");
    }
}

#[derive(Debug)]
pub struct ComponentModel {
    components: HashSet<Component>,
}

impl ComponentModel {
    /// Build the component model from a collection of surfaces.
    /// 
    /// The surfaces must be in sequential order, with the first surface being the object plane and
    /// the last surface being the image plane.
    /// 
    /// # Arguments
    /// * `surfaces` - The surfaces to build the component model from.
    /// * `background` - The refractive index of the background medium.
    pub fn new(surfaces: &[Surface], background: f32) -> Self {
        let mut components = HashSet::new();

        let mut surf_pairs = SurfacePairIterator::new(surfaces).enumerate();
        let max_idx = surfaces.len() - 1;
        
        if max_idx < 2 {
            // There are no components because only the object and image plane exist.
            return Self { components: components };
        }

        while let Some((i, surf_pair)) = surf_pairs.next() {
            if i == 0 || i == max_idx {
                // Don't include the object or image plane surfaces
                continue;
            }

            if ComponentModel::same_medium(surf_pair.0.n(), background) {
                // Don't include surface pairs that go from background to another medium because
                // these are gaps.
                continue;
            }

            components.insert(Component::Surface { surf_idx: i });
        }

        // If the last non-image plane surface goes from background to another medium, then add it
        // as a single isolated surface.
        let last_surf = surfaces[max_idx - 1];
        let next_to_last_surf = surfaces[max_idx - 2];
        if ComponentModel::same_medium(next_to_last_surf.n(), background) &&
            !ComponentModel::same_medium(last_surf.n(), background) {
            components.insert(Component::Surface { surf_idx: max_idx - 1 });
        }

        Self {
            components: components,
        }
    }

    fn same_medium(n1: f32, n2: f32) -> bool {
        (n1 - n2).abs() < 1e-6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_cases::empty_system;

    #[test]
    fn test_new_no_components() {
        let system_model = empty_system();

        let component_model = ComponentModel::new(system_model.seq_model.surfaces(), system_model.background());

        assert_eq!(component_model.components.len(), 0);
    }
}
            