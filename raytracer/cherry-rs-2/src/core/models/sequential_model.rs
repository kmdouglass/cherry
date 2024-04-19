use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::core::seq::{Gap, Step, Surface};

pub(crate) struct SequentialModel {
    surfaces: Rc<RefCell<Vec<Surface>>>,
    gaps: Vec<Gap>,
}

struct SequentialModelIter<'a> {
    surfaces: &'a Ref<'a, Vec<Surface>>,
    gaps: &'a Vec<Gap>,
    index: usize,
}

impl<'a> SequentialModelIter<'a> {
    fn new(surfaces: &'a Ref<'a, Vec<Surface>>, gaps: &'a Vec<Gap>) -> Self {
        Self {
            surfaces,
            gaps,
            index: 0,
        }
    }
}

impl<'a> Iterator for SequentialModelIter<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.gaps.len() - 1 {
            // We are at the last gap
            let result = Some((&self.gaps[self.index], &self.surfaces[self.index + 1], None));
            self.index += 1;
            result
        } else if self.index < self.gaps.len() {
            let result = Some((
                &self.gaps[self.index],
                &self.surfaces[self.index + 1],
                Some(&self.gaps[self.index + 1]),
            ));
            self.index += 1;
            result
        } else {
            None
        }
    }
}
