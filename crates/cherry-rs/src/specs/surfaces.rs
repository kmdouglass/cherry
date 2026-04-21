use serde::{Deserialize, Serialize};

use crate::core::{Float, math::linalg::rotations::Rotation3D, math::vec3::Vec3};

/// Specifies the type of interaction of light with a sequential model surface.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum BoundaryType {
    Refracting,
    Reflecting,
    NoOp,
}

/// Specifies the clear aperture of a surface.
///
/// This is referred to as a "mask" to avoid confusion with
/// [`specs::apertures::ApertureSpec`] which specifies the physical aperture of
/// the system.
///
/// `Unbounded` surfaces (Object, Image, Probe) pass all rays unconditionally.
#[derive(Debug, Clone, Copy)]
pub enum Mask {
    Circular { semi_diameter: Float },
    Unbounded,
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
        surf_type: BoundaryType,
        rotation: Rotation3D,
    },
    /// A user-defined surface type registered with a [`SurfaceRegistry`].
    ///
    /// `type_id` must match a key registered via
    /// [`SurfaceRegistry::register`]. `params` is forwarded verbatim to the
    /// registered constructor.
    ///
    /// [`SurfaceRegistry`]: crate::core::surfaces::SurfaceRegistry
    /// [`SurfaceRegistry::register`]: crate::core::surfaces::SurfaceRegistry::register
    Custom {
        type_id: String,
        params: serde_json::Value,
        rotation: Rotation3D,
    },
    Image {
        rotation: Rotation3D,
    },
    Object,
    Probe {
        rotation: Rotation3D,
    },
    Stop {
        semi_diameter: Float,
        rotation: Rotation3D,
    },
}

impl Mask {
    /// Returns `true` if `pos` lies outside the clear aperture. The axial
    /// z-component of `pos` is ignored.
    pub fn outside_clear_aperture(&self, pos: Vec3) -> bool {
        match self {
            Mask::Circular { semi_diameter } => {
                let r_transv = pos.x() * pos.x() + pos.y() * pos.y();
                let r_max = *semi_diameter;
                r_transv > r_max * r_max
            }
            Mask::Unbounded => false,
        }
    }

    /// Returns the semi-diameter of the mask.
    ///
    /// Returns [`Float::INFINITY`] for [`Mask::Unbounded`].
    pub fn semi_diameter(&self) -> Float {
        match self {
            Mask::Circular { semi_diameter } => *semi_diameter,
            Mask::Unbounded => Float::INFINITY,
        }
    }
}
