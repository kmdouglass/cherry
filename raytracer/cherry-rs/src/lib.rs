//! Cherry is a library for sequential optical system design.
//!
//! The core structure of sequential optical design is the
//! [SequentialModel](struct@SequentialModel) which is a set of submodels
//! containing surfaces and gaps between surfaces. Each
//! [SequentialSubModel](trait@SequentialSubModel) corresponds to a unique set
//! of system parameters, i.e. a wavelength and a transverse axis. A submodel
//! provides an interface to iterate over the surfaces and gaps in the system.
//!
//! Inputs to the system are provided by specs, of which there are several
//! types:
//!
//! - [SurfaceSpec](enum@SurfaceSpec) - Describes a surface in the system for
//!   which surface sag or paraxial ray trace matrices can be calculated.
//! - [GapSpec](struct@GapSpec) - Describes a gap between surfaces in the
//!   system. Refractive index data is located here.
//! - [ApertureSpec](enum@ApertureSpec) - Describes the aperture of the system.
//!   This may differ from any pupils that can be derived directly from the
//!   surfaces and gaps.
//! - [FieldSpec](enum@FieldSpec) - Describes the field points of the system.
//! - [RefractiveIndexSpec](trait@RefractiveIndexSpec) - Describes the
//!   refractive index of a gap. This is a trait so that different material
//!   databases may be implemented.
//! - Wavelength - Describes a single wavelength to model.
//!
//! The outputs of the system are provided by views, such as:
//!
//! - [ParaxialView](struct@ParaxialView) - A paraxial view of the system.
//!   Contains information such as focal length, principal planes, etc.
//! - [RayTrace3DView](fn@ray_trace_3d_view) - A 3D ray trace view of the
//!   system.
//! - [CutawayView](struct@CutawayView) - A cutaway view of the system. Used
//!   primarily for drawing the system.
//! - [ComponentsView](fn@components_view) - A view of the components of the
//!   system. Used for grouping surfaces into lenses.
//!
//! # Quick Start
//! ```rust
//! use cherry_rs::{
//!     n, ray_trace_3d_view, ApertureSpec, FieldSpec, GapSpec, ImagePlane, ParaxialView, Pupil, PupilSampling, RefractiveIndexSpec,
//!     SequentialModel, SurfaceSpec, SurfaceType,
//! };
//!
//! // Create a convexplano lens with an object at infinity.
//! let air = n!(1.0);
//! let nbk7 = n!(1.515);
//!
//! // Define a set of gaps between surfaces.
//! let gaps = vec![
//!     GapSpec {
//!         thickness: f64::INFINITY,
//!         refractive_index: air.clone(),
//!     },
//!     GapSpec {
//!         thickness: 5.3,
//!         refractive_index: nbk7,
//!     },
//!     GapSpec {
//!        thickness: 46.6,
//!        refractive_index: air,
//!     },
//! ];
//!
//! // Define a set of surfaces in the system.
//! let surfaces = vec![
//!     SurfaceSpec::Object,
//!     SurfaceSpec::Conic {
//!         semi_diameter: 12.5,
//!         radius_of_curvature: 25.8,
//!         conic_constant: 0.0,
//!         surf_type: SurfaceType::Refracting,
//!     },
//!     SurfaceSpec::Conic {
//!         semi_diameter: 12.5,
//!         radius_of_curvature: f64::INFINITY,
//!         conic_constant: 0.0,
//!         surf_type: SurfaceType::Refracting,
//!     },
//!     SurfaceSpec::Image,
//! ];
//!
//! // Define a set of wavelengths to model.
//! let wavelengths: Vec<f64> = vec![0.567];
//!
//! // Create a sequential model from the gaps, surfaces, and wavelengths.
//! let sequential_model = SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap();
//!
//! // Define a user-defined system aperture.
//! let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 5.0 };
//!
//! // Analyze the system at two different field points, sampling the pupil
//! // with a square grid with a spacing of 0.1 in normalized pupil coordinates.
//! let field_specs = vec![
//!     FieldSpec::Angle {
//!         angle: 0.0,
//!         pupil_sampling: PupilSampling::SquareGrid { spacing: 0.1 },
//!     },
//!     FieldSpec::Angle {
//!         angle: 5.0,
//!         pupil_sampling: PupilSampling::SquareGrid { spacing: 0.1 },
//!     },
//! ];
//!
//! // Compute the paraxial view of the system.
//! let paraxial_view = ParaxialView::new(&sequential_model, &field_specs, false).unwrap();
//!
//! // Compute the effective focal length of the lens for each submodel.
//! for (sub_model_id, _) in sequential_model.submodels() {
//!     let sub_view = paraxial_view.subviews().get(sub_model_id).unwrap();
//!     let result = sub_view.effective_focal_length();
//!
//!     println!("Submodel ID: {:?}, Effective focal length: {}", sub_model_id, result);
//! }
//!
//! // Compute a 3D ray trace of the system.
//! let rays = ray_trace_3d_view(
//!     &aperture_spec, &field_specs,
//!     &sequential_model,
//!     &paraxial_view,
//!     None,
//! ).unwrap();
//! ```

mod core;
mod materials;
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
    gaps::{ConstantRefractiveIndex, GapSpec, RefractiveIndexSpec},
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

// Re-exports from dependencies
#[cfg(feature = "ri-info")]
pub use lib_ria::Material;
