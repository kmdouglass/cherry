use std::collections::HashSet;

/// A part is a surface, gap, surface set.
pub struct Part {}

struct ComponentModel {
    parts: HashSet<Part>,
}
