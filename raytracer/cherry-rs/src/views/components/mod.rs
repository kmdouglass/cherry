use std::collections::HashSet;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{
    core::{sequential_model::Surface, Float, RefractiveIndex},
    RefractiveIndexSpec, SequentialModel,
};

mod test_cases;

const TOL: Float = 1e-6;

/// A component is a part of an optical system that can interact with light
/// rays.
///
/// Components come in two types: elements, and stops. Elements are the most
/// basic compound optical component and are represented as a set of surfaces
/// pairs. Stops are hard stops that block light rays.
///
/// To avoid copying data, only indexes are stored from the surface models are
/// stored.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Component {
    Element { surf_idxs: (usize, usize) },
    Stop { stop_idx: usize },
    UnpairedSurface { surf_idx: usize },
}

pub fn components_view(
    sequential_model: &SequentialModel,
    background: RefractiveIndexSpec,
) -> Result<HashSet<Component>> {
    let mut components = HashSet::new();

    let surfaces = sequential_model.surfaces();
    let mut surface_pairs = surfaces.iter().zip(surfaces.iter().skip(1)).enumerate();
    let max_idx = surfaces.len() - 1;
    let mut paired_surfaces = HashSet::new();

    // Ignore wavelengths and axes, just get any submodel for now
    let sequential_sub_model = sequential_model
        .submodels()
        .values()
        .next()
        .ok_or(anyhow!("No submodels found in the sequential model."))?;
    let gaps = sequential_sub_model.gaps();

    let background_refractive_index = RefractiveIndex::try_from_spec(&background, None)?;

    if max_idx < 2 {
        // There are no components because only the object and image plane exist.
        return Ok(components);
    }

    while let Some((i, surf_pair)) = surface_pairs.next() {
        if i == 0 || i == max_idx {
            // Don't include the object or image plane surfaces
            continue;
        }

        if let Surface::Stop(_) = surf_pair.0 {
            // Stops are special, so be sure that they're added before anything else.
            components.insert(Component::Stop { stop_idx: i });
            continue;
        }

        if let Surface::Stop(_) = surf_pair.1 {
            // Ensure that stops following surfaces are NOT added as a component
            continue;
        }

        if same_medium(gaps[i].refractive_index, background_refractive_index) {
            // Don't include surface pairs that go from background to another medium because
            // these are gaps.
            continue;
        }

        if let Surface::Image(_) = surf_pair.1 {
            // Check whether the next to last surface has already been paired with another.
            if !paired_surfaces.contains(&i) {
                components.insert(Component::UnpairedSurface { surf_idx: i });
                continue;
            }
        }

        components.insert(Component::Element {
            surf_idxs: (i, i + 1),
        });
        paired_surfaces.insert(i);
        paired_surfaces.insert(i + 1);
    }

    Ok(components)
}

/// Two different media are considered the same if their refractive indices are
/// within a small tolerance of each other.
fn same_medium(eta_1: RefractiveIndex, eta_2: RefractiveIndex) -> bool {
    (eta_1.n() - eta_2.n()).abs() < TOL && (eta_1.k() - eta_2.k()).abs() < TOL
}

#[cfg(test)]
mod tests {
    use crate::RefractiveIndexSpec;

    use super::*;

    use test_cases::{
        empty_system, silly_single_surface_and_stop, silly_unpaired_surface,
        wollaston_landscape_lens,
    };

    const AIR: RefractiveIndexSpec = RefractiveIndexSpec {
        real: crate::RealSpec::Constant(1.0),
        imag: None,
    };

    #[test]
    fn test_new_no_components() {
        let sequential_model = empty_system();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 0);
    }

    #[test]
    fn test_planoconvex_lens() {
        let sequential_model = crate::examples::convexplano_lens::sequential_model();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 1);
        assert!(components.contains(&Component::Element { surf_idxs: (1, 2) }));
    }

    #[test]
    fn test_silly_single_surface_and_stop() {
        // This is not a useful system but a good test.
        let sequential_model = silly_single_surface_and_stop();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 1);
        assert!(components.contains(&Component::Stop { stop_idx: 2 })); // Hard stop
    }

    #[test]
    fn test_silly_unpaired_surface() {
        // This is not a useful system but a good test.
        let sequential_model = silly_unpaired_surface();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 2);
        assert!(components.contains(&Component::Element { surf_idxs: (1, 2) }));
        assert!(components.contains(&Component::UnpairedSurface { surf_idx: 3 }));
    }

    #[test]
    fn test_wollaston_landscape_lens() {
        let sequential_model = wollaston_landscape_lens();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 2);
        assert!(components.contains(&Component::Stop { stop_idx: 1 })); // Hard stop
        assert!(components.contains(&Component::Element { surf_idxs: (2, 3) }));
        // Lens
    }
}
