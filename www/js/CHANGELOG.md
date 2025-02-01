# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Materials dispersion data is now downloaded and added to the browser's IndexedDB database when the application starts.
- Users may now specify materials data to use in a lens design, and this data will be used to automatically compute refractive indexes at each wavelength.

## [0.2.0] 2024-12-09

### Added

- A `Results > Summary` window to display the system properties.

### Fixed

- You can no longer change tabs in the data entry section of the GUI when there is an invalid value for a cell. This prevents you from getting locked out of the cell.

[Unreleased]: https://github.com/kmdouglass/cherry/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kmdouglass/cherry/releases/tag/v0.2.0
[0.1.0]: https://github.com/kmdouglass/cherry/releases/tag/v0.1.0
