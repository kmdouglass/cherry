use serde::{Deserialize, Serialize};

use crate::core::{Float, rotations::Rotation};

/// Specifies the type of interaction of light with a sequential model surface.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SurfaceType {
    Refracting,
    Reflecting,
    NoOp,
}

/// Specifies a surface in a sequential optical system.
///
/// Rotations are optional and specify the active rotation sequence to orient
/// the surface in 3D.
#[derive(Debug, Serialize, Deserialize)]
pub enum SurfaceSpec {
    Conic {
        semi_diameter: Float,
        radius_of_curvature: Float,
        conic_constant: Float,
        surf_type: SurfaceType,
        rotation: Rotation,
    },
    Image {
        rotation: Rotation,
    },
    Object,
    Probe {
        rotation: Rotation,
    },
    Stop {
        semi_diameter: Float,
        rotation: Rotation,
    },
    // Toric {
    //     semi_diameter: Float,
    //     radius_of_curvature_vert: Float,
    //     radius_of_curvature_horz: Float,
    //     conic_constant: Float,
    //     surf_type: SurfaceType,
    // },
}
