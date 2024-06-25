use ndarray::ArrayView1;

use crate::core::Float;

pub fn argmin(ratios: &ArrayView1<Float>) -> usize {
    ratios
        .iter()
        .enumerate()
        .fold((0, Float::MAX), |(min_idx, min_val), (idx, &val)| {
            if val < min_val {
                (idx, val)
            } else {
                (min_idx, min_val)
            }
        })
        .0
}
