# GUI Design

The GUI is composed of

- a main window that contains all panels
- various panels for data entry, a.k.a. specs
- various panels for results, i.e. paraxial quantities, cross-section views, etc.

## File Menu

- File
  - Save
  - Open...
  - Export results
- Results
  - Summary
- Examples

## Inputs

### Specs

The specs window has three (sometimes four) panels. There is one table to each panel.

- Surfaces
  - Two modes: Specify refractive indexes directly, or select a material from the list of loaded optical materials (see below)
- Fields
- Aperture
- Wavelength (optional)
  - Only visible when the surfaces table is in `material select` mode

### Materials

This is an interface to a RefractiveIndex.INFO database dump.

There are drop downs for Book, Shelf, and Page (highest to lowest) levels of the database hierarchy.

Updating one drop down changes the choices available in the lower hierarchies. A button moves the selected material into a list of "Selected Materials". Only selected materials are available in the surface spec table in material mode.

## Implementation

- The new GUI is implemented in egui: <https://github.com/emilk/egui>
- It works in pure Wasm using Rust --> no need for two languages
- Build desktop and browser apps

## Old GUI

The old GUI was a React frontend that called into the cherry-rs core as necessary.

- Located in `www/js`