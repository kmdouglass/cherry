//! A flat 45° beam splitter: object → BS → two image planes.
//!
//! Build with `two_path_model(n_air, wavelengths, gap_transmitted,
//! gap_reflected)`. Path 0 is the transmitted arm (+z); path 1 is the reflected
//! arm (+y after a –45° tilt about R).
use std::rc::Rc;

use crate::{
    BeamSplitterPathKind, EulerAngles, GapSpec, PathSpec, PathSurfaceRef, RefractiveIndexSpec,
    Rotation3D, SequentialModel, SurfaceSpec, Vec3,
    core::{Float, sequential_model::builder::SequentialModelBuilder},
};

/// Build a two-path beam splitter model.
///
/// - Path 0: transmitted arm — image along +z at distance `gap_transmitted`
///   from the BS.
/// - Path 1: reflected arm — image along +y at distance `gap_reflected` from
///   the BS (45° BS deflects 90° into +y).
pub fn two_path_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[Float],
    gap_transmitted: Float,
    gap_reflected: Float,
) -> SequentialModel {
    let bs_rotation =
        Rotation3D::IntrinsicPassiveRUF(EulerAngles((-45_f64 as Float).to_radians(), 0.0, 0.0));

    let img = || SurfaceSpec::Image {
        rotation: Rotation3D::None,
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let gap_inf = || GapSpec {
        thickness: Float::INFINITY,
        refractive_index: n_air.clone(),
    };

    // Path 0: transmitted arm.
    // Store indices: Object=0, BS=1, Image_T=2
    let path_t = PathSpec {
        surface_refs: vec![
            PathSurfaceRef::New(SurfaceSpec::Object),
            PathSurfaceRef::New(SurfaceSpec::BeamSplitter {
                semi_diameter: 10.0,
                rotation: bs_rotation,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(img()),
        ],
        gaps: vec![
            gap_inf(),
            GapSpec {
                thickness: gap_transmitted,
                refractive_index: n_air.clone(),
            },
        ],
        beam_splitter_arms: vec![BeamSplitterPathKind::Transmitting],
        stop_surface: None,
        wavelengths: wavelengths.to_vec(),
    };

    // Path 1: reflected arm.
    // Shared(0)=Object, Shared(1)=BS (same physical component), New=Image_R=3
    let path_r = PathSpec {
        surface_refs: vec![
            PathSurfaceRef::Shared(0),
            PathSurfaceRef::Shared(1),
            PathSurfaceRef::New(img()),
        ],
        gaps: vec![
            gap_inf(),
            GapSpec {
                thickness: gap_reflected,
                refractive_index: n_air,
            },
        ],
        beam_splitter_arms: vec![BeamSplitterPathKind::Reflecting],
        stop_surface: None,
        wavelengths: wavelengths.to_vec(),
    };

    SequentialModelBuilder::new()
        .paths(vec![path_t, path_r])
        .build()
        .expect("beam splitter model builds")
        .model
}
