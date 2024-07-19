/// Performs a 3D ray trace on the system.
use crate::{
    core::{
        sequential_model::{Axis, SequentialModel, SequentialSubModel, Step, SubModelID, Surface},
        Float,
    },
    specs::{aperture::ApertureSpec, fields::FieldSpec, surfaces::SurfaceType},
};

mod rays;

#[derive(Debug)]
pub struct RayTrace3DView {}

impl RayTrace3DView {
    pub fn new(aperture_spec: ApertureSpec, field_specs: &[FieldSpec]) -> Self {
        Self {}
    }
}
