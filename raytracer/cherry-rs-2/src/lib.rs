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
pub use views::{
    cutaway::cutaway_view,
    paraxial::{ParaxialSubViewDescription, ParaxialView, ParaxialViewDescription, Pupil},
    ray_trace_3d::{ray_trace_3d_view, Ray, TraceResults},
};
