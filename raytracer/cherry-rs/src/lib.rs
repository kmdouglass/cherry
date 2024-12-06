mod core;
mod specs;
mod views;

// API
pub mod examples;
pub use core::{
    math::vec3::Vec3,
    sequential_model::{Axis, SequentialModel, SequentialSubModel, Step, SubModelID},
};
pub use specs::{
    aperture::ApertureSpec,
    fields::{FieldSpec, PupilSampling},
    gaps::{GapSpec, ImagSpec, RealSpec, RefractiveIndexSpec},
    surfaces::SurfaceSpec,
    surfaces::SurfaceType,
};
pub use views::{
    components::{components_view, Component},
    cutaway::CutawayView,
    paraxial::{
        ImagePlane, ParaxialSubView, ParaxialSubViewDescription, ParaxialView,
        ParaxialViewDescription, Pupil,
    },
    ray_trace_3d::{ray_trace_3d_view, Ray, TraceResults},
};
