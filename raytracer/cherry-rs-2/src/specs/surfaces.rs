use serde::{Deserialize, Serialize};

use crate::core::Float;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SurfaceType {
    Refracting,
    Reflecting,
    NoOp,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SurfaceSpec {
    Conic {
        semi_diameter: Float,
        radius_of_curvature: Float,
        conic_constant: Float,
        surf_type: SurfaceType,
    },
    Image,
    Object,
    Stop {
        semi_diameter: Float,
    },
    Toric {
        semi_diameter: Float,
        radius_of_curvature_vert: Float,
        radius_of_curvature_horz: Float,
        conic_constant: Float,
        surf_type: SurfaceType,
    },
}
