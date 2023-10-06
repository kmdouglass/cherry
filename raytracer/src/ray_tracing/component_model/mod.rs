use std::collections::HashSet;

use anyhow::{bail, Result};

use crate::ray_tracing::{Gap, Surface};

/// A component is a part of an optical system that can interact with light rays.
///
/// Components come in two types: elements and surfaces. Elements are complete optical components
/// and are represented as sets of surfaces with sequentially increasing indexes. Surfaces are 
/// elements without any gaps, e.g. incomplete elements, stops, or object/image planes.
/// 
/// An element can be comprised more than two surfaces, such as an achromatic doublet. In this case,
/// there are three surfaces.
///
/// To avoid copying data, only indexes are stored from the sequential models are stored.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Component {
    Element {
        surf_idxs: Vec<usize>,
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

        Ok(Self::Element { surf_idxs })
    }
}

#[derive(Debug)]
pub struct ComponentModel {
    components: HashSet<Component>,
}

impl ComponentModel {
    /// Build the component model from a collection of surfaces and gaps.
    /// 
    /// This method assumes that the surfaces and gaps are in the correct order and are valid
    /// collections according to the rules for valid system models in this software package.
    pub fn new(surfaces: &[Surface]) -> Self {
        let mut components = HashSet::new();

        for (i, surf) in surfaces[1..surfaces.len() - 1].iter().enumerate() {
            components.insert(Component::Surface { surf_idx: i });
        }
        Self {
            components: HashSet::new(),
        }
    }
}
