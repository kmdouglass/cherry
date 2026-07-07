//! A reduced version of the widefield epifluorescence microscope excitation
//! path: an object, a thin lens, a dichroic beam splitter (reflection), and a
//! second thin lens that is the aperture stop.
//!
//! The beam splitter is oriented at +45° and is used in the reflected arm;
//! excitation light is folded 90° downward toward the sample.
//!
//! By hand: the stop (second thin lens) sits 150 mm (100 mm + 50 mm) after
//! the first thin lens (f = 40 mm). Treating the stop as a real object for
//! the first thin lens and solving 1/s' = 1/f - 1/s with s = 150 mm gives
//! s' = 600 / 11 = 54.5455 mm, with the image (entrance pupil) on the
//! object-space side of the first thin lens, i.e. at location -54.5455 mm
//! relative to it.
use std::rc::Rc;

use crate::{
    BeamSplitterPathKind, EulerAngles, GapSpec, PathSpec, PathSurfaceRef, RefractiveIndexSpec,
    Rotation3D, SequentialModel, SurfaceSpec, Vec3,
    core::{
        Float,
        sequential_model::{builder::SequentialModelBuilder, solves::MarginalRaySolve},
    },
};

pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    n_oil: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[f64],
) -> SequentialModel {
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
        refractive_index: n_air,
    };
    // Placeholder thickness; solved below to place the image plane at the
    // paraxial focus (MarginalRaySolve targeting gap 3, target_height = 0.0).
    let gap_3 = GapSpec {
        thickness: 1.0,
        refractive_index: n_oil,
    };

    let path = PathSpec {
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
                rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(
                    (45.0_f64 as Float).to_radians(),
                    0.0,
                    0.0,
                )),
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
        wavelengths: wavelengths.to_vec(),
    };

    SequentialModelBuilder::new()
        .paths(vec![path])
        .solves(vec![Box::new(MarginalRaySolve::new(3, 0.0, 0))])
        .build()
        .expect("wf_epi_excitation model builds")
        .model
}
