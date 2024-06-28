mod core;
mod specs;
mod systems;
mod views;

// API
pub mod examples;
pub use specs::{aperture::ApertureSpec, fields::FieldSpec, gaps::{GapSpec, ImagSpec, RealSpec, RefractiveIndexSpec}, surfaces::SurfaceSpec, surfaces::SurfaceType};
pub use systems::System;
pub use views::View;
