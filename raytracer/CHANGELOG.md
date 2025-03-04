# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A `ri-info` feature for loading materials data from the RefractiveIndex.info database.
- A `primary_axial_color` method to `ParaxialView` for computing the axial primary color aberration of a lens.
- An `axes` method on `SequentialModel` to return the set of axes that the system is modeled over.
- `RayBundle`, `TraceResultsCollection` were added as part of refactoring the `ray_trace_3d_view`.

### Changed

- `RefractiveIndexSpec` is now a trait which supports getting refractive index data from any generic materials database.
- The examples, including the associated tests, were moved into a Cargo-specific `examples` folder.
- `ray_trace_3d_view` now returns a `TraceResultsCollection` of modified `TraceResults` structs. This allows for better access to a given set of values for (field_id, wavelength_id, Axis). `Ray` was also modified and now contains only position and direction information.

### Fixed

- Fixed an import error in the `n` macro.

## [0.2.0] 2024-12-09

### Added

- A macro `n` for easily creating real `RefractiveIndexSpec`s.
- A Petzval lens example.
- The following paraxial properties are now computed in the paraxial view: back focal distance, front focal distance, effective focal length, exit pupil, back principal plane, front principal plane, the chief ray, and the paraxial image plane.

### Fixed

- The aperture stop calculation of the paraxial view now correctly finds the minimum semi-diameter to pseudo-marginal ray height ratio by first taking the absolute value of the ratios. 

[Unreleased]: https://github.com/kmdouglass/cherry/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kmdouglass/cherry/releases/tag/v0.2.0
[0.1.0]: https://github.com/kmdouglass/cherry/releases/tag/v0.1.0
