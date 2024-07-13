mod core;
mod specs;
mod views;

// API
pub mod examples;
pub use core::sequential_model::SequentialModel;
pub use specs::{
    aperture::ApertureSpec,
    fields::FieldSpec,
    gaps::{GapSpec, ImagSpec, RealSpec, RefractiveIndexSpec},
    surfaces::SurfaceSpec,
    surfaces::SurfaceType,
};
pub use views::paraxial::ParaxialView;
