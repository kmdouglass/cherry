# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A macro `n` for easily creating real `RefractiveIndexSpec`s.
- A Petzval lens example.
- The following paraxial properties are now computed in the paraxial view: back focal distance, effective focal length.

### Fixed

- The aperture stop calculation of the paraxial view now correctly finds the minimum semi-diameter to pseudo-marginal ray height ratio by first taking the absolute value of the ratios. 

[Unreleased]: https://github.com/kmdouglass/cherry/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kmdouglass/cherry/releases/tag/v0.1.0
