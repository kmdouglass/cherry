//! A Michelson interferometer: a reference example for building complex
//! multipath systems.
//!
//! ```text
//!                     M2 (arm 2)
//!                      │
//!                      │ arm_2
//!                      │
//!  source ──── iris ───╳──────────── M1 (arm 1)
//!    (∞)          50   BS (+45°)  arm_1
//!                      │ 50
//!                     ═══  lens, f = 100
//!                      │ 100
//!                     ─┴─ detector
//! ```
//!
//! Each arm is one path. Both paths show two multipath idioms:
//!
//! - **Revisits.** An arm crosses the beam splitter twice: once on the way out
//!   to its mirror and once on the way back. The second crossing is a `Shared`
//!   reference to a surface the *same* path already introduced, with the
//!   opposite arm declared in `beam_splitter_arms` (path 0: transmit out,
//!   reflect back; path 1: reflect out, transmit back). Only direction-
//!   independent surfaces (flat surfaces, thin lenses) may be revisited within
//!   one path; the builder rejects curved ones.
//! - **Recombination.** Path 1 shares every surface with path 0 except its own
//!   mirror, ending on `Shared` references to path 0's lens and detector, so
//!   both arms land on one physical Image.
//!
//! Gaps for `Shared` steps are never inferred here: each path restates them,
//! including the return leg after each mirror, and they must agree with the
//! shared geometry.
//!
//! Cherry is a geometric ray tracer; there is no interference model. The
//! example illustrates topology, not fringes.
use std::rc::Rc;

use crate::{
    BeamSplitterPathKind, BoundaryKind, EulerAngles, GapSpec, PathSpec, PathSurfaceRef,
    RefractiveIndexSpec, Rotation3D, SequentialModel, SurfaceSpec, Vec3,
    core::{Float, sequential_model::builder::SequentialModelBuilder},
};

/// Distance from the iris (aperture stop) to the beam splitter.
pub const IRIS_TO_BS: Float = 50.0;
/// Distance from the beam splitter to the detector lens.
pub const BS_TO_LENS: Float = 50.0;
/// Detector lens focal length.
pub const LENS_F: Float = 100.0;
/// Distance from the detector lens to the detector (its focal plane).
pub const LENS_TO_DETECTOR: Float = 100.0;
/// Semi-diameter of the beam splitter, mirrors, and lens (1-inch optics).
pub const SEMI_DIAMETER: Float = 12.7;
/// Semi-diameter of the iris, which is the aperture stop of both arms.
pub const IRIS_SEMI_DIAMETER: Float = 2.0;

/// Build the two-path Michelson interferometer.
///
/// - Path 0 (arm 1): BS transmits to M1 (+z) at distance `arm_1`; on return the
///   BS reflects toward the detector (−y).
/// - Path 1 (arm 2): BS reflects to M2 (+y) at distance `arm_2`; on return the
///   BS transmits toward the detector.
///
/// Store indices: 0 Object, 1 Iris, 2 BS, 3 M1, 4 Lens, 5 Detector, 6 M2.
pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[Float],
    arm_1: Float,
    arm_2: Float,
) -> SequentialModel {
    const OBJECT: usize = 0;
    const IRIS: usize = 1;
    const BS: usize = 2;
    const LENS: usize = 4;
    const DETECTOR: usize = 5;

    let gap = |thickness: Float| GapSpec {
        thickness,
        refractive_index: n_air.clone(),
    };
    let flat_mirror = || SurfaceSpec::Conic {
        semi_diameter: SEMI_DIAMETER,
        radius_of_curvature: Float::INFINITY,
        conic_constant: 0.0,
        surf_kind: BoundaryKind::Reflecting,
        rotation: Rotation3D::None,
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };

    let path_arm_1 = PathSpec {
        surface_refs: vec![
            PathSurfaceRef::New(SurfaceSpec::Object),
            PathSurfaceRef::New(SurfaceSpec::Iris {
                semi_diameter: IRIS_SEMI_DIAMETER,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(SurfaceSpec::BeamSplitter {
                semi_diameter: SEMI_DIAMETER,
                rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(
                    (45.0 as Float).to_radians(),
                    0.0,
                    0.0,
                )),
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(flat_mirror()), // M1
            PathSurfaceRef::Shared(BS),         // return pass: revisit
            PathSurfaceRef::New(SurfaceSpec::ThinLens {
                semi_diameter: SEMI_DIAMETER,
                focal_length: LENS_F,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
        ],
        gaps: vec![
            gap(Float::INFINITY),
            gap(IRIS_TO_BS),
            gap(arm_1), // BS → M1
            gap(arm_1), // M1 → BS
            gap(BS_TO_LENS),
            gap(LENS_TO_DETECTOR),
        ],
        beam_splitter_arms: vec![
            BeamSplitterPathKind::Transmitting,
            BeamSplitterPathKind::Reflecting,
        ],
        stop_surface: Some(IRIS),
        wavelengths: wavelengths.to_vec(),
    };

    let path_arm_2 = PathSpec {
        surface_refs: vec![
            PathSurfaceRef::Shared(OBJECT),
            PathSurfaceRef::Shared(IRIS),
            PathSurfaceRef::Shared(BS),
            PathSurfaceRef::New(flat_mirror()), // M2
            PathSurfaceRef::Shared(BS),         // return pass: revisit
            PathSurfaceRef::Shared(LENS),
            PathSurfaceRef::Shared(DETECTOR), // recombination
        ],
        gaps: vec![
            gap(Float::INFINITY),
            gap(IRIS_TO_BS),
            gap(arm_2), // BS → M2
            gap(arm_2), // M2 → BS
            gap(BS_TO_LENS),
            gap(LENS_TO_DETECTOR),
        ],
        beam_splitter_arms: vec![
            BeamSplitterPathKind::Reflecting,
            BeamSplitterPathKind::Transmitting,
        ],
        stop_surface: Some(IRIS),
        wavelengths: wavelengths.to_vec(),
    };

    SequentialModelBuilder::new()
        .paths(vec![path_arm_1, path_arm_2])
        .build()
        .expect("michelson_interferometer model builds")
        .model
}
