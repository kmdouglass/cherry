#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::{Float, math::linalg::rotations::Rotation3D, math::vec3::Vec3};

/// Specifies the kind of interaction of light with a sequential model surface.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BoundaryKind {
    Refracting,
    Reflecting,
    NoOp,
}

/// Selects which optical path a [`BeamSplitter`] surface models.
///
/// A beam splitter creates two output paths from one input. Because
/// [`SequentialModel`] is single-path, each path is a separate model whose
/// beam splitter surface is configured with the appropriate variant here.
///
/// [`BeamSplitter`]: crate::core::surfaces::BeamSplitter
/// [`SequentialModel`]: crate::core::sequential_model::SequentialModel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BeamSplitterPathKind {
    /// Transmitted path: ray refracts (Snell's law); cursor continues straight.
    Transmitting,
    /// Reflected path: ray reflects (law of reflection); cursor is deflected.
    Reflecting,
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

/// Tilt and decenter input spec for one surface.
///
/// This is the *input* passed to [`SequentialModel::from_surfaces`] and used
/// internally when converting [`SurfaceSpec`]s into a model. It is distinct
/// from the computed [`Placement`], which records the surface's resolved
/// position and orientation in the global frame.
///
/// The three fields have distinct roles:
/// - `rotation` — nominal cursor-redirecting tilt (e.g., 45° for a fold
///   mirror); a first-order design parameter, not a perturbation.
/// - `rotation_offset` — additional surface-only tilt that never redirects the
///   cursor; a true perturbation in the classical tilt/decenter sense.
/// - `decenter` — vertex offset from the nominal cursor position, in
///   cursor-frame (R, U, F); a true decenter in the classical sense.
///
/// [`SequentialModel::from_surfaces`]:
///     crate::core::sequential_model::SequentialModel::from_surfaces
/// [`Placement`]: crate::core::sequential_model::placement::Placement
#[derive(Debug, Clone)]
pub struct PlacementSpec {
    /// Nominal surface tilt; redirects the cursor for reflecting surfaces.
    pub rotation: Rotation3D,
    /// Additional surface-only rotation; never redirects the cursor.
    pub rotation_offset: Rotation3D,
    /// Vertex offset from the nominal cursor position, in cursor-frame (R, U,
    /// F).
    pub decenter: Vec3,
}

impl PlacementSpec {
    /// Returns a `PlacementSpec` with no tilt, no rotation offset, and no
    /// decenter.
    pub fn none() -> Self {
        Self {
            rotation: Rotation3D::None,
            rotation_offset: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    /// Returns a `PlacementSpec` with the given nominal rotation and no
    /// decenter or rotation offset.
    pub fn from_rotation(rotation: Rotation3D) -> Self {
        Self {
            rotation,
            rotation_offset: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

/// Specifies a surface in a sequential optical system.
///
/// Rotations specify transformations from the cursor reference frame to the
/// surface local reference frame.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SurfaceSpec {
    Conic {
        semi_diameter: Float,
        radius_of_curvature: Float,
        conic_constant: Float,
        surf_kind: BoundaryKind,
        rotation: Rotation3D,
        #[cfg_attr(feature = "serde", serde(default = "default_zero_vec3"))]
        decenter: Vec3,
        #[cfg_attr(feature = "serde", serde(default = "default_rotation3d_none"))]
        rotation_offset: Rotation3D,
    },
    Sphere {
        semi_diameter: Float,
        radius_of_curvature: Float,
        surf_kind: BoundaryKind,
        rotation: Rotation3D,
        #[cfg_attr(feature = "serde", serde(default = "default_zero_vec3"))]
        decenter: Vec3,
        #[cfg_attr(feature = "serde", serde(default = "default_rotation3d_none"))]
        rotation_offset: Rotation3D,
    },
    /// An idealized thin lens with a focal length independent of the
    /// surrounding media.
    ThinLens {
        semi_diameter: Float,
        focal_length: Float,
        rotation: Rotation3D,
        #[cfg_attr(feature = "serde", serde(default = "default_zero_vec3"))]
        decenter: Vec3,
        #[cfg_attr(feature = "serde", serde(default = "default_rotation3d_none"))]
        rotation_offset: Rotation3D,
    },
    /// A user-defined surface type registered with a [`SurfaceRegistry`].
    ///
    /// `type_id` must match a key registered via
    /// [`SurfaceRegistry::register`]. `params` is forwarded verbatim to the
    /// registered constructor.
    ///
    /// [`SurfaceRegistry`]: crate::core::surfaces::SurfaceRegistry
    /// [`SurfaceRegistry::register`]: crate::core::surfaces::SurfaceRegistry::register
    #[cfg(feature = "serde")]
    Custom {
        type_id: String,
        params: serde_json::Value,
        rotation: Rotation3D,
    },
    Image {
        rotation: Rotation3D,
        #[cfg_attr(feature = "serde", serde(default = "default_zero_vec3"))]
        decenter: Vec3,
        #[cfg_attr(feature = "serde", serde(default = "default_rotation3d_none"))]
        rotation_offset: Rotation3D,
    },
    Object,
    Probe {
        rotation: Rotation3D,
        #[cfg_attr(feature = "serde", serde(default = "default_zero_vec3"))]
        decenter: Vec3,
        #[cfg_attr(feature = "serde", serde(default = "default_rotation3d_none"))]
        rotation_offset: Rotation3D,
    },
    Iris {
        semi_diameter: Float,
        rotation: Rotation3D,
        #[cfg_attr(feature = "serde", serde(default = "default_zero_vec3"))]
        decenter: Vec3,
        #[cfg_attr(feature = "serde", serde(default = "default_rotation3d_none"))]
        rotation_offset: Rotation3D,
    },
    /// A flat beam-splitting surface nominally tilted like a mirror.
    ///
    /// The surface is purely geometric. Arm selection (transmitting vs.
    /// reflecting) is declared per path via
    /// [`PathSpec::beam_splitter_arms`].
    ///
    /// [`PathSpec::beam_splitter_arms`]: crate::specs::paths::PathSpec
    BeamSplitter {
        semi_diameter: Float,
        rotation: Rotation3D,
        #[cfg_attr(feature = "serde", serde(default = "default_zero_vec3"))]
        decenter: Vec3,
        #[cfg_attr(feature = "serde", serde(default = "default_rotation3d_none"))]
        rotation_offset: Rotation3D,
    },
}

#[cfg(feature = "serde")]
fn default_zero_vec3() -> Vec3 {
    Vec3::new(0.0, 0.0, 0.0)
}

#[cfg(feature = "serde")]
fn default_rotation3d_none() -> Rotation3D {
    Rotation3D::None
}

impl SurfaceSpec {
    /// Nominal surface tilt; redirects the cursor for reflecting surfaces.
    pub fn rotation(&self) -> Rotation3D {
        match self {
            SurfaceSpec::Conic { rotation, .. }
            | SurfaceSpec::Sphere { rotation, .. }
            | SurfaceSpec::ThinLens { rotation, .. }
            | SurfaceSpec::Image { rotation, .. }
            | SurfaceSpec::Probe { rotation, .. }
            | SurfaceSpec::Iris { rotation, .. }
            | SurfaceSpec::BeamSplitter { rotation, .. } => rotation.clone(),
            SurfaceSpec::Object => Rotation3D::None,
            #[cfg(feature = "serde")]
            SurfaceSpec::Custom { rotation, .. } => rotation.clone(),
        }
    }

    /// Additional surface-only rotation; never redirects the cursor.
    pub fn rotation_offset(&self) -> Rotation3D {
        match self {
            SurfaceSpec::Conic {
                rotation_offset, ..
            }
            | SurfaceSpec::Sphere {
                rotation_offset, ..
            }
            | SurfaceSpec::ThinLens {
                rotation_offset, ..
            }
            | SurfaceSpec::Image {
                rotation_offset, ..
            }
            | SurfaceSpec::Probe {
                rotation_offset, ..
            }
            | SurfaceSpec::Iris {
                rotation_offset, ..
            }
            | SurfaceSpec::BeamSplitter {
                rotation_offset, ..
            } => rotation_offset.clone(),
            SurfaceSpec::Object => Rotation3D::None,
            #[cfg(feature = "serde")]
            SurfaceSpec::Custom { .. } => Rotation3D::None,
        }
    }

    /// Vertex offset from the nominal cursor position, in cursor-frame (R, U,
    /// F).
    pub fn decenter(&self) -> Vec3 {
        match self {
            SurfaceSpec::Conic { decenter, .. }
            | SurfaceSpec::Sphere { decenter, .. }
            | SurfaceSpec::ThinLens { decenter, .. }
            | SurfaceSpec::Image { decenter, .. }
            | SurfaceSpec::Probe { decenter, .. }
            | SurfaceSpec::Iris { decenter, .. }
            | SurfaceSpec::BeamSplitter { decenter, .. } => *decenter,
            SurfaceSpec::Object => Vec3::new(0.0, 0.0, 0.0),
            #[cfg(feature = "serde")]
            SurfaceSpec::Custom { .. } => Vec3::new(0.0, 0.0, 0.0),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // Step 3: SurfaceSpec::BeamSplitter can be constructed without path_kind.
    #[test]
    fn beam_splitter_spec_has_no_path_kind() {
        let _ = SurfaceSpec::BeamSplitter {
            semi_diameter: 10.0,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
    }

    // AT-9: serialize a Sphere with non-zero decenter and rotation_offset, then
    // deserialize; assert the round-tripped values are preserved.
    #[cfg(feature = "serde")]
    #[test]
    fn at9_serde_round_trip_preserves_decenter_and_rotation_offset() {
        use crate::core::math::linalg::rotations::EulerAngles;

        let phi = 0.1_f64;
        let spec = SurfaceSpec::Sphere {
            semi_diameter: 12.5,
            radius_of_curvature: 25.8,
            surf_kind: BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.5, -0.3, 0.0),
            rotation_offset: Rotation3D::IntrinsicPassiveRUF(EulerAngles(phi, 0.0, 0.0)),
        };

        let json = serde_json::to_string(&spec).expect("serialize");
        let back: SurfaceSpec = serde_json::from_str(&json).expect("deserialize");

        let d = back.decenter();
        assert!((d.x() - 0.5).abs() < 1e-15, "decenter x: {}", d.x());
        assert!((d.y() - (-0.3)).abs() < 1e-15, "decenter y: {}", d.y());
        assert!(d.z().abs() < 1e-15, "decenter z: {}", d.z());

        match back.rotation_offset() {
            Rotation3D::IntrinsicPassiveRUF(EulerAngles(a, b, c)) => {
                assert!((a - phi).abs() < 1e-15, "rotation_offset theta: {a}");
                assert!(b.abs() < 1e-15, "rotation_offset psi: {b}");
                assert!(c.abs() < 1e-15, "rotation_offset phi: {c}");
            }
            other => panic!("unexpected rotation_offset variant: {:?}", other),
        }
    }

    // ThinLens: serialize then deserialize a ThinLens spec with non-zero
    // decenter and rotation_offset; assert the round-tripped values match,
    // and that focal_length survives the round trip.
    #[cfg(feature = "serde")]
    #[test]
    fn thin_lens_serde_round_trip_preserves_focal_length_and_decenter() {
        use crate::core::math::linalg::rotations::EulerAngles;

        let phi = 0.2_f64;
        let spec = SurfaceSpec::ThinLens {
            semi_diameter: 25.0,
            focal_length: 100.0,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.1, 0.2, 0.0),
            rotation_offset: Rotation3D::IntrinsicPassiveRUF(EulerAngles(phi, 0.0, 0.0)),
        };

        let json = serde_json::to_string(&spec).expect("serialize");
        let back: SurfaceSpec = serde_json::from_str(&json).expect("deserialize");

        match back {
            SurfaceSpec::ThinLens {
                focal_length,
                semi_diameter,
                ..
            } => {
                assert!((focal_length - 100.0).abs() < 1e-15);
                assert!((semi_diameter - 25.0).abs() < 1e-15);
            }
            other => panic!("unexpected spec variant: {:?}", other),
        }

        let d = back.decenter();
        assert!((d.x() - 0.1).abs() < 1e-15, "decenter x: {}", d.x());
        assert!((d.y() - 0.2).abs() < 1e-15, "decenter y: {}", d.y());
    }

    // AT-10: deserializing a JSON string that omits decenter and rotation_offset
    // applies the correct defaults (zero vector and None).
    #[cfg(feature = "serde")]
    #[test]
    fn at10_serde_default_decenter_and_rotation_offset() {
        let json = r#"{
            "Sphere": {
                "semi_diameter": 10.0,
                "radius_of_curvature": 50.0,
                "surf_kind": "Refracting",
                "rotation": "None"
            }
        }"#;

        let spec: SurfaceSpec = serde_json::from_str(json).expect("deserialize");

        let d = spec.decenter();
        assert!(
            d.x().abs() < 1e-15 && d.y().abs() < 1e-15 && d.z().abs() < 1e-15,
            "default decenter should be zero: {:?}",
            d
        );

        assert!(
            matches!(spec.rotation_offset(), Rotation3D::None),
            "default rotation_offset should be None"
        );
    }
}
