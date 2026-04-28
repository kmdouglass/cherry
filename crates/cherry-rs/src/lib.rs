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
//! - [CrossSectionView](fn@cross_section_view) - A 2D cross section through the
//!   system.
//! - [ComponentsView](fn@components_view) - A view of the components of the
//!   system. Used for grouping surfaces into lenses.
//!
//! # Quick Start
//! ```rust
//! use cherry_rs::{
//!     n, ray_trace_3d_view, trace_ray_bundle, ApertureSpec, FieldSpec, GapSpec, ImagePlane,
//!     ParaxialView, Pupil, SamplingConfig, RefractiveIndexSpec, Rotation3D,
//!     SequentialModel, SurfaceSpec, BoundaryType,
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
//!     SurfaceSpec::Sphere {
//!         semi_diameter: 12.5,
//!         radius_of_curvature: 25.8,
//!         surf_type: BoundaryType::Refracting,
//!         rotation: Rotation3D::None,
//!     },
//!     SurfaceSpec::Sphere {
//!         semi_diameter: 12.5,
//!         radius_of_curvature: f64::INFINITY,
//!         surf_type: BoundaryType::Refracting,
//!         rotation: Rotation3D::None,
//!     },
//!     SurfaceSpec::Image { rotation: Rotation3D::None },
//! ];
//!
//! // Define a set of wavelengths to model.
//! let wavelengths: Vec<f64> = vec![0.567];
//!
//! // Create a sequential model from the gaps, surfaces, and wavelengths.
//! let sequential_model = SequentialModel::from_surface_specs(&gaps, &surfaces, &wavelengths, None).unwrap();
//!
//! // Define a user-defined system aperture.
//! let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 5.0 };
//!
//! // Analyze the system at two different field points.
//! let field_specs = vec![
//!     FieldSpec::Angle { chi: 0.0, phi: 90.0 },
//!     FieldSpec::Angle { chi: 5.0, phi: 90.0 },
//! ];
//!
//! // Compute the paraxial view of the system.
//! let paraxial_view = ParaxialView::new(&sequential_model, &field_specs, false).unwrap();
//!
//! // Compute the effective focal length of the lens for each submodel.
//! for sub_view in paraxial_view.iter() {
//!     let result = sub_view.effective_focal_length();
//!
//!     println!(
//!         "wavelength_id={} tangential_vec_id={} EFL={}",
//!         sub_view.wavelength_id(), sub_view.tangential_vec_id(), result
//!     );
//! }
//!
//! // Compute a 3D ray trace of the system, sampling the pupil with a square
//! // grid with a spacing of 0.1 in normalized pupil coordinates.
//! let results_collection = ray_trace_3d_view(
//!     &aperture_spec, &field_specs,
//!     &sequential_model,
//!     &paraxial_view,
//!     SamplingConfig { n_fan_rays: 9, full_pupil_spacing: 0.1 },
//! ).unwrap();
//!
//! // Get all results for the second (5 degree) field point.
//! let results = results_collection.get_by_field_id(1);
//! println!("Results for 5 degree field point: {:?}", results);
//! ```

mod core;
mod materials;
mod specs;
mod views;

#[cfg(feature = "gui")]
pub mod gui;

// API
pub mod examples;
#[cfg(feature = "serde")]
pub use core::surfaces::{SurfaceConstructor, SurfaceRegistry};
pub use core::{
    math::linalg::rotations::{EulerAngles, Rotation3D},
    math::vec3::Vec3,
    placement::Placement,
    ray::Ray,
    sequential_model::{SequentialModel, SequentialSubModel, Step},
    surfaces::{Conic, Image, Iris, Object, Probe, Sphere, Surface, SurfaceKind},
};
pub use specs::{
    aperture::ApertureSpec,
    fields::{FieldSpec, PupilSampling},
    gaps::{ConstantRefractiveIndex, GapSpec, RefractiveIndexSpec},
    surfaces::{BoundaryType, Mask, SurfaceSpec},
};
pub use views::{
    components::{Component, components_view},
    cross_section::{
        Bounds2D, CrossSectionView, DrawElement, FlatPlaneKind, PlaneGeometry, cross_section_view,
    },
    paraxial::{
        ImagePlane, ParaxialRay, ParaxialRayBundle, ParaxialSubView, ParaxialSubViewDescription,
        ParaxialView, ParaxialViewDescription, Pupil,
    },
    ray_trace_3d::{
        RayBundle, SamplingConfig, TraceResults, TraceResultsCollection, ray_trace_3d_view,
        trace_ray_bundle,
    },
};

// Re-exports from dependencies
#[cfg(feature = "ri-info")]
pub use lib_ria::Material;
