mod core;
mod specs;
mod views;

// API
pub mod examples;
pub use core::sequential_model::{SequentialModel, SequentialSubModel, SubModelID};
pub use specs::{
    aperture::ApertureSpec,
    fields::{FieldSpec, PupilSampling},
    gaps::{GapSpec, ImagSpec, RealSpec, RefractiveIndexSpec},
    surfaces::SurfaceSpec,
    surfaces::SurfaceType,
};
pub use views::{paraxial::{ParaxialView, Pupil}, ray_trace_3d::{Ray, ray_trace_3d_view, TraceResults}};
