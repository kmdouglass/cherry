use crate::core::seq::{Gap, Step, Surface};

struct SequentialModel {
    surfaces: Vec<Surface>,
    gaps: Vec<Gap>,
}

struct SequentialModelIter<'a> {
    model: &'a SequentialModel,
    index: usize,
}

impl<'a> SequentialModelIter<'a> {
    fn new(model: &'a SequentialModel) -> Self {
        Self { model, index: 0 }
    }
}

impl<'a> Iterator for SequentialModelIter<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.model.gaps.len() - 1 {
            // We are at the last gap
            let result = Some((
                &self.model.gaps[self.index],
                &self.model.surfaces[self.index + 1],
                None,
            ));
            self.index += 1;
            result
        } else if self.index < self.model.gaps.len() {
            let result = Some((
                &self.model.gaps[self.index],
                &self.model.surfaces[self.index + 1],
                Some(&self.model.gaps[self.index + 1]),
            ));
            self.index += 1;
            result
        } else {
            None
        }
    }
}
