pub(crate) mod conic;
pub(crate) mod line;

use anyhow::Result;

use crate::core::math::{geometry::curves::{conic::Conic, line::Line}, vec3::Vec3};

pub enum GeometricCurve {
    Conic(Conic),
    Line(Line)
}

// TODO: Fix this.
// impl GeometricCurve {
//     fn sample(&self, num_samples: usize) -> Result<Vec<Vec3>> {
//         match self {
//             GeometricCurve::Conic(conic) => conic.sample(num_samples),
//             GeometricCurve::Line(line) => line.sample(num_samples),
//         }
//     }
// }
