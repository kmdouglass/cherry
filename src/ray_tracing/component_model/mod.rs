use std::collections::HashSet;

use anyhow::{bail, Result};

use crate::ray_tracing::sequential_model::SequentialModel;

/// A component is a part of an optical system that can interact with light rays.
///
/// Components come in two types: elements and surfaces. Elements are complete optical elements
/// and are represented as sets of {surface, gap, surface}. Surfaces are are elements without
/// any gaps, e.g. incomplete elements.
///
/// To avoid copying data, only indexes are stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Component {
    Element {
        surf_idx_1: usize,
        gap_idx: usize,
        surf_idx_2: usize,
    },
    Surface {
        surf_idx: usize,
    },
    ObjectPlane {
        surf_idx: usize,
    },
    ImagePlane {
        surf_idx: usize,
    },
}

impl Component {
    pub fn new(
        surf_idx_1: usize,
        gap_idx: Option<usize>,
        surf_idx_2: Option<usize>,
    ) -> Result<Self> {
        match (gap_idx, surf_idx_2) {
            (Some(gap_idx), Some(surf_idx_2)) => Ok(Self::Element {
                surf_idx_1,
                gap_idx,
                surf_idx_2,
            }),
            (None, None) => Ok(Self::Surface {
                surf_idx: surf_idx_1,
            }),
            _ => anyhow::bail!(
                "A component must be either a single surface or a surface, gap, surface set."
            ),
        }
    }
}

#[derive(Debug)]
pub struct ComponentModel {
    components: HashSet<Component>,
}

impl ComponentModel {
    pub fn new() -> Self {
        Self {
            components: HashSet::new(),
        }
    }

    fn insert_component(&mut self, component: Component) -> Result<()> {
        // Check for existing object or image planes.
        match component {
            Component::ObjectPlane { .. } => {
                if self.object_plane().is_some() {
                    bail!("An object plane already exists.");
                }
            }
            Component::ImagePlane { .. } => {
                if self.image_plane().is_some() {
                    bail!("An image plane already exists.");
                }
            }
            _ => {}
        }
        self.components.insert(component);

        Ok(())
    }

    fn remove_component(&mut self, component: Component) -> Result<()> {
        self.components.remove(&component);

        Ok(())
    }

    fn object_plane(&self) -> Option<Component> {
        // Get all the object planes.
        let mut object_planes: Vec<Component> = self
            .components
            .iter()
            .filter(|component| match component {
                Component::ObjectPlane { .. } => true,
                _ => false,
            })
            .copied()
            .collect();

        if object_planes.is_empty() {
            return None;
        }

        // By convention there is only one object plane.
        assert_eq!(object_planes.len(), 1);
        Some(object_planes[0])
    }

    fn image_plane(&self) -> Option<Component> {
        // Get all the image planes.
        let mut image_planes: Vec<Component> = self
            .components
            .iter()
            .filter(|component| match component {
                Component::ImagePlane { .. } => true,
                _ => false,
            })
            .copied()
            .collect();

        if image_planes.is_empty() {
            return None;
        }

        // By convention there is only one image plane.
        assert_eq!(image_planes.len(), 1);
        Some(image_planes[0])
    }
}

impl From<&SequentialModel> for ComponentModel {
    fn from(seq_model: &SequentialModel) -> Self {
        let mut components = HashSet::new();

        // Add object and image planes.
        components.insert(Component::ObjectPlane { surf_idx: 0 });
        components.insert(Component::ImagePlane {
            surf_idx: seq_model.surfaces().len() - 1,
        });

        // Only a single surface exists.
        if seq_model.gaps().len() == 2 {
            components.insert(Component::Surface { surf_idx: 1 });
            return Self { components };
        }

        // There is an unpaired surface. Put it at the end, before the image plane.
        if seq_model.gaps().len() % 2 == 0 {
            let surf_idx = seq_model.surfaces().len() - 2;
            components.insert(Component::Surface { surf_idx });
        }

        // Build the set of elements. Start at the first non-object-plane surface.
        let mut gap_idx: usize = 1;
        while gap_idx < seq_model.gaps().len() - 1 {
            let surf_idx_1 = gap_idx;
            let surf_idx_2 = gap_idx + 1;

            let component = Component::new(surf_idx_1, Some(gap_idx), Some(surf_idx_2))
                .expect("This should never fail.");
            components.insert(component);

            // Skip air gaps
            gap_idx += 2;
        }

        Self { components }
    }
}

impl From<&mut SequentialModel> for ComponentModel {
    fn from(seq_model: &mut SequentialModel) -> Self {
        let seq_model = &*seq_model;
        Self::from(seq_model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ray_tracing::sequential_model::{Gap, SurfaceSpec};
    use crate::ray_tracing::SystemModel;

    fn system_model() -> SystemModel {
        let mut system_model = SystemModel::new();
        let seq_model = system_model.seq_model_mut();

        let surf_spec_1 = SurfaceSpec::RefractingCircularConic {
            diam: 25.0,
            roc: 25.8,
            k: 0.0,
        };
        let gap_1 = Gap::new(1.515, 5.3);
        let surf_spec_2 = SurfaceSpec::RefractingCircularFlat { diam: 25.0 };
        let gap_2 = Gap::new(1.0, 46.6);

        seq_model
            .insert_surface_and_gap(1, surf_spec_1, gap_1)
            .unwrap();
        seq_model
            .insert_surface_and_gap(2, surf_spec_2, gap_2)
            .unwrap();

        system_model
    }

    #[test]
    fn test_component_model() {
        let mut sys_model = system_model();
        let seq_model = sys_model.seq_model_mut();

        let comp_model = ComponentModel::from(seq_model);

        // Two Component::Surfaces should exist: object plane and image plane. One
        // Component::Element should exist, comprised of two surfaces and their gap.
        assert_eq!(comp_model.components.len(), 3);
        assert_eq!(
            comp_model.object_plane().unwrap(),
            Component::ObjectPlane { surf_idx: 0 }
        );
        assert_eq!(
            comp_model.image_plane().unwrap(),
            Component::ImagePlane { surf_idx: 3 }
        );
        assert_eq!(
            comp_model.components.contains(&Component::Element {
                surf_idx_1: 1,
                gap_idx: 1,
                surf_idx_2: 2
            }),
            true
        );
    }

    #[test]
    fn test_component_model_even_number_of_gaps() {
        // An even number of gaps indicates that the last surface is unpaired.
        let mut sys_model = system_model();
        let seq_model = sys_model.seq_model_mut();

        let surf_spec_3 = SurfaceSpec::RefractingCircularConic {
            diam: 25.0,
            roc: 25.8,
            k: 0.0,
        };
        let gap_3 = Gap::new(1.515, 5.3);
        seq_model
            .insert_surface_and_gap(3, surf_spec_3, gap_3)
            .unwrap();

        let comp_model = ComponentModel::from(seq_model);

        // Three Component::Surfaces should exist: object plane, unpaired surface, and image plane. One
        // Component::Element should exist, comprised of two surfaces and their gap.
        assert_eq!(comp_model.components.len(), 4);
        assert_eq!(
            comp_model.object_plane().unwrap(),
            Component::ObjectPlane { surf_idx: 0 }
        );
        assert_eq!(
            comp_model.image_plane().unwrap(),
            Component::ImagePlane { surf_idx: 4 }
        );
        assert_eq!(
            comp_model.components.contains(&Component::Element {
                surf_idx_1: 1,
                gap_idx: 1,
                surf_idx_2: 2
            }),
            true
        );
        assert_eq!(
            comp_model
                .components
                .contains(&Component::Surface { surf_idx: 3 }),
            true
        );
    }

    #[test]
    fn test_component_model_cannot_insert_multiple_object_planes() {
        let mut sys_model = system_model();
        let seq_model = sys_model.seq_model_mut();

        let mut comp_model = ComponentModel::from(seq_model);

        let result = comp_model.insert_component(Component::ObjectPlane { surf_idx: 5 });

        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_component_model_cannot_insert_multiple_image_planes() {
        let mut sys_model = system_model();
        let seq_model = sys_model.seq_model_mut();

        let mut comp_model = ComponentModel::from(seq_model);

        let result = comp_model.insert_component(Component::ImagePlane { surf_idx: 5 });

        assert_eq!(result.is_err(), true);
    }
}
