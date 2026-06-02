//! A flat 45° beam splitter between object and image planes.
//!
//! The beam splitter sits at the origin, tilted 45° about the R axis. Use
//! `transmitting_model` for the path that continues along +z, or
//! `reflecting_model` for the path that is deflected 90° into +y.
use std::rc::Rc;

use crate::{
    BeamSplitterPathKind, EulerAngles, GapSpec, RefractiveIndexSpec, Rotation3D, SequentialModel,
    SurfaceSpec, Vec3, core::Float,
};

/// Transmitted path: cursor continues along +z; image plane is along +z.
pub fn transmitting_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[Float],
) -> SequentialModel {
    build(n_air, wavelengths, BeamSplitterPathKind::Transmitting)
}

/// Reflected path: cursor deflects 90° into +y; image plane is along +y.
pub fn reflecting_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[Float],
) -> SequentialModel {
    build(n_air, wavelengths, BeamSplitterPathKind::Reflecting)
}

fn build(
    n_air: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[Float],
    path_kind: BeamSplitterPathKind,
) -> SequentialModel {
    let gap_0 = GapSpec {
        thickness: Float::INFINITY,
        refractive_index: n_air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 100.0,
        refractive_index: n_air,
    };
    let gaps = vec![gap_0, gap_1];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::BeamSplitter {
        semi_diameter: 10.0,
        path_kind,
        rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(
            (-45 as Float).to_radians(),
            0.0,
            0.0,
        )),
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let surf_2 = SurfaceSpec::Image {
        rotation: Rotation3D::None,
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let surfaces = vec![surf_0, surf_1, surf_2];

    SequentialModel::from_surface_specs(&gaps, &surfaces, wavelengths, None).unwrap()
}
