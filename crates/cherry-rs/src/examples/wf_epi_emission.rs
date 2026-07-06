//! A reduced widefield epifluorescence emission path: sample → objective →
//! dichroic beam splitter (transmission) → flat fold mirror → tube lens →
//! camera.
//!
//! Emission light originates in a sample medium (n = 1.5), passes through the
//! objective (f = 3.3333 mm, aperture stop), transmits through a dichroic beam
//! splitter tilted at –45°, reflects off a flat fold mirror (also at –45°),
//! passes through a tube lens (f = 200 mm), and reaches the image plane.
use std::rc::Rc;

use crate::{
    BeamSplitterPathKind, BoundaryKind, EulerAngles, GapSpec, PathSpec, PathSurfaceRef,
    RefractiveIndexSpec, Rotation3D, SequentialModel, SurfaceSpec, Vec3,
    core::{Float, sequential_model::builder::SequentialModelBuilder},
};

pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    n_sample: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[f64],
) -> SequentialModel {
    let gap_0 = GapSpec {
        thickness: 5.0,
        refractive_index: n_sample,
    };
    let gap_1 = GapSpec {
        thickness: 50.0,
        refractive_index: n_air.clone(),
    };
    let gap_2 = GapSpec {
        thickness: 50.0,
        refractive_index: n_air.clone(),
    };
    let gap_3 = GapSpec {
        thickness: 50.0,
        refractive_index: n_air.clone(),
    };
    let gap_4 = GapSpec {
        thickness: 200.0,
        refractive_index: n_air,
    };

    let tilt_neg45 =
        Rotation3D::IntrinsicPassiveRUF(EulerAngles((-45.0_f64 as Float).to_radians(), 0.0, 0.0));

    let path = PathSpec {
        surface_refs: vec![
            PathSurfaceRef::New(SurfaceSpec::Object),
            PathSurfaceRef::New(SurfaceSpec::ThinLens {
                semi_diameter: 4.25,
                focal_length: 3.3333,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(SurfaceSpec::BeamSplitter {
                semi_diameter: 36.5,
                rotation: tilt_neg45,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(SurfaceSpec::Conic {
                semi_diameter: 10.0,
                radius_of_curvature: Float::INFINITY,
                conic_constant: 0.0,
                surf_kind: BoundaryKind::Reflecting,
                rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(
                    (-45.0_f64 as Float).to_radians(),
                    0.0,
                    0.0,
                )),
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(SurfaceSpec::ThinLens {
                semi_diameter: 20.0,
                focal_length: 200.0,
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
        gaps: vec![gap_0, gap_1, gap_2, gap_3, gap_4],
        beam_splitter_arms: vec![BeamSplitterPathKind::Transmitting],
        stop_surface: Some(1),
        wavelengths: wavelengths.to_vec(),
    };

    SequentialModelBuilder::new()
        .paths(vec![path])
        .build()
        .expect("wf_epi_emission model builds")
        .model
}
