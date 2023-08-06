use std::collections::HashSet;

/// A part is a surface, gap, surface set.
/// 
/// To avoid copying data, only indexes are stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Component {
    surf_idx_1: usize,
    gap_idx: usize,
    surf_idx_2: usize,
}

impl Component {
    pub fn new(surf_idx_1: usize, gap_idx: usize, surf_idx_2: usize) -> Self {
        Self {
            surf_idx_1,
            gap_idx,
            surf_idx_2,
        }
    }
}

struct ComponentModel {
    components: HashSet<Component>,
}

impl ComponentModel {
    pub fn new() -> Self {
        Self {
            components: HashSet::new(),
        }
    }

    pub fn insert_component(&mut self, component: Component) {
        self.components.insert(component);
    }

    pub fn remove_component(&mut self, component: Component) {
        self.components.remove(&component);
    }
}
