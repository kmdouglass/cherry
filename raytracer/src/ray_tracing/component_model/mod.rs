use std::collections::HashSet;

use crate::ray_tracing::{Component, Surface, SurfacePairIterator};


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
            return Self {
                components: components,
            };
        }

        while let Some((i, surf_pair)) = surf_pairs.next() {
            if i == 0 || i == max_idx {
                // Don't include the object or image plane surfaces
                continue;
            }

            if let Surface::Stop(_) = surf_pair.0 {
                // Stops are special, so be sure that they're added before anything else.
                components.insert(Component::Stop { stop_idx: i });
            }

            if let Surface::Stop(_) = surf_pair.1 {
                // Ensure that stops following surfaces are NOT added as a component
                continue;
            }

            if ComponentModel::same_medium(surf_pair.0.n(), background) {
                // Don't include surface pairs that go from background to another medium because
                // these are gaps.
                continue;
            }

            components.insert(Component::Element {
                surf_idxs: (i, i + 1),
            });
        }

        Self {
            components: components,
        }
    }

    pub fn components(&self) -> &HashSet<Component> {
        &self.components
    }

    fn same_medium(n1: f32, n2: f32) -> bool {
        (n1 - n2).abs() < 1e-6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_cases::{
        empty_system, planoconvex_lens_obj_at_inf, silly_single_surface_and_stop,
        wollaston_landscape_lens,
    };

    #[test]
    fn test_new_no_components() {
        let system_model = empty_system();

        let component_model =
            ComponentModel::new(system_model.seq_model.surfaces(), system_model.background());

        assert_eq!(component_model.components.len(), 0);
    }

    #[test]
    fn test_planoconvex_lens() {
        let system_model = planoconvex_lens_obj_at_inf();

        let component_model =
            ComponentModel::new(system_model.seq_model.surfaces(), system_model.background());

        assert_eq!(component_model.components.len(), 1);
        assert!(component_model
            .components
            .contains(&Component::Element { surf_idxs: (1, 2) }));
    }

    #[test]
    fn test_silly_single_surface_and_stop() {
        // This is not a useful system but a good test.
        let system_model = silly_single_surface_and_stop();

        let component_model =
            ComponentModel::new(system_model.seq_model.surfaces(), system_model.background());

        assert_eq!(component_model.components.len(), 1);
        assert!(component_model
            .components
            .contains(&Component::Stop { stop_idx: 2 })); // Hard stop
    }

    #[test]
    fn test_wollaston_landscape_lens() {
        let system_model = wollaston_landscape_lens();

        let component_model =
            ComponentModel::new(system_model.seq_model.surfaces(), system_model.background());

        assert_eq!(component_model.components.len(), 2);
        assert!(component_model
            .components
            .contains(&Component::Stop { stop_idx: 1 })); // Hard stop
        assert!(component_model
            .components
            .contains(&Component::Element { surf_idxs: (2, 3) })); // Lens
    }
}
