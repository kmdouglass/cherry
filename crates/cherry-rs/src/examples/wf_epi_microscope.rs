//! Combined widefield epifluorescence microscope: excitation and emission
//! paths sharing an objective and dichroic beam splitter.
//!
//! Path 0 (excitation): source → tube lens (f = 40 mm) → beam splitter
//! (reflecting arm, +45°) → objective (f = 3.3333 mm, aperture stop) →
//! sample.
//!
//! Path 1 (emission): sample (linked, reversed, to the excitation Image) →
//! objective (shared) → beam splitter (shared, transmitting arm) → fold
//! mirror (−45°) → tube lens (f = 200 mm) → camera.
//!
//! The emission path's Object is co-located with the excitation path's
//! Image (the sample plane); its cursor is derived from the excitation
//! Image cursor with R unchanged and U, F negated (`Reversed`). The gaps
//! from the emission Object to the objective and from the objective to the
//! beam splitter are inferred from the excitation path's saved gap specs,
//! since both paths traverse the same physical space.
use std::rc::Rc;

use crate::{
    BeamSplitterPathKind, BoundaryKind, EulerAngles, GapSpec, LinkedObjectOrientation, PathSpec,
    PathSurfaceRef, RefractiveIndexSpec, Rotation3D, SequentialModel, SurfaceSpec, Vec3,
    core::{Float, sequential_model::builder::SequentialModelBuilder},
};

pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    n_oil: Rc<dyn RefractiveIndexSpec>,
    excitation_wavelengths: &[f64],
    emission_wavelengths: &[f64],
) -> SequentialModel {
    // Excitation path gaps.
    let gap_0 = GapSpec {
        thickness: 40.0,
        refractive_index: n_air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 100.0,
        refractive_index: n_air.clone(),
    };
    let gap_2 = GapSpec {
        thickness: 50.0,
        refractive_index: n_air.clone(),
    };
    let gap_3 = GapSpec {
        thickness: 5.0,
        refractive_index: n_oil,
    };

    // Emission path gaps (user-supplied only; Object→Objective and
    // Objective→BeamSplitter are inferred from the excitation path).
    let gap_bs_mirror = GapSpec {
        thickness: 50.0,
        refractive_index: n_air.clone(),
    };
    let gap_mirror_tube = GapSpec {
        thickness: 50.0,
        refractive_index: n_air.clone(),
    };
    let gap_tube_camera = GapSpec {
        thickness: 200.0,
        refractive_index: n_air,
    };

    let bs_rotation_exc =
        Rotation3D::IntrinsicPassiveRUF(EulerAngles((45.0_f64 as Float).to_radians(), 0.0, 0.0));
    let mirror_rotation =
        Rotation3D::IntrinsicPassiveRUF(EulerAngles((-45.0_f64 as Float).to_radians(), 0.0, 0.0));

    let path_excitation = PathSpec {
        surface_refs: vec![
            PathSurfaceRef::New(SurfaceSpec::Object),
            PathSurfaceRef::New(SurfaceSpec::ThinLens {
                semi_diameter: 25.0,
                focal_length: 40.0,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(SurfaceSpec::BeamSplitter {
                semi_diameter: 36.0,
                rotation: bs_rotation_exc,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::New(SurfaceSpec::ThinLens {
                semi_diameter: 4.25,
                focal_length: 3.3333,
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
        gaps: vec![gap_0, gap_1, gap_2, gap_3],
        beam_splitter_arms: vec![BeamSplitterPathKind::Reflecting],
        stop_surface: Some(3),
        wavelengths: excitation_wavelengths.to_vec(),
    };

    let path_emission = PathSpec {
        surface_refs: vec![
            PathSurfaceRef::ObjectLinkedTo {
                path: 0,
                orientation: LinkedObjectOrientation::Reversed,
            },
            PathSurfaceRef::Shared(3), // Objective
            PathSurfaceRef::Shared(2), // BeamSplitter
            PathSurfaceRef::New(SurfaceSpec::Conic {
                semi_diameter: 10.0,
                radius_of_curvature: Float::INFINITY,
                conic_constant: 0.0,
                surf_kind: BoundaryKind::Reflecting,
                rotation: mirror_rotation,
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
        gaps: vec![gap_bs_mirror, gap_mirror_tube, gap_tube_camera],
        beam_splitter_arms: vec![BeamSplitterPathKind::Transmitting],
        stop_surface: Some(3),
        wavelengths: emission_wavelengths.to_vec(),
    };

    SequentialModelBuilder::new()
        .paths(vec![path_excitation, path_emission])
        .build()
        .expect("wf_epi_microscope model builds")
        .model
}
