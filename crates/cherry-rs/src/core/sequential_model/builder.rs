/// Builder for immutable SequentialModels.
use anyhow::{Result, anyhow};

#[cfg(feature = "serde")]
use crate::core::surfaces::SurfaceRegistry;
use crate::specs::{gaps::GapSpec, surfaces::SurfaceSpec};

use super::SequentialModel;
use super::solves::Solve;

/// Specs-based construction of [`SequentialModel`]s with optional extensions.
///
/// Use this when you need solves or a custom surface registry. For simple
/// construction without either, prefer [`SequentialModel::from_surface_specs`]
/// directly.
pub struct SequentialModelBuilder {
    gap_specs: Option<Vec<GapSpec>>,
    surface_specs: Option<Vec<SurfaceSpec>>,
    stop_surface: Option<usize>,
    wavelengths: Option<Vec<f64>>,
    solves: Vec<Box<dyn Solve>>,
    #[cfg(feature = "serde")]
    registry: Option<SurfaceRegistry>,
}

impl Default for SequentialModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SequentialModelBuilder {
    /// Creates a new [`SequentialModelBuilder`].
    pub fn new() -> Self {
        Self {
            gap_specs: None,
            surface_specs: None,
            stop_surface: None,
            wavelengths: None,
            solves: Vec::new(),
            #[cfg(feature = "serde")]
            registry: None,
        }
    }

    pub fn build(self) -> Result<SequentialModel> {
        self.validate()?;

        // Destructure all fields before any partial moves.
        let mut gap_specs = self.gap_specs.unwrap();
        let mut surface_specs = self.surface_specs.unwrap();
        let wavelengths = self.wavelengths.unwrap();
        let stop_surface = self.stop_surface;
        let solves = self.solves;
        #[cfg(feature = "serde")]
        let registry = self.registry;

        let build = |gap_specs: &[GapSpec], surface_specs: &[SurfaceSpec]| {
            #[cfg(feature = "serde")]
            return SequentialModel::from_surface_specs_with_registry(
                gap_specs,
                surface_specs,
                &wavelengths,
                stop_surface,
                registry.as_ref(),
            );
            #[cfg(not(feature = "serde"))]
            SequentialModel::from_surface_specs(
                gap_specs,
                surface_specs,
                &wavelengths,
                stop_surface,
            )
        };

        let mut model = build(&gap_specs, &surface_specs)?;

        for solve in solves {
            solve.apply(&model, &mut gap_specs, &mut surface_specs)?;
            model = build(&gap_specs, &surface_specs)?;
        }

        Ok(model)
    }

    pub fn gap_specs(mut self, gap_specs: Vec<GapSpec>) -> Self {
        self.gap_specs = Some(gap_specs);
        self
    }

    pub fn surface_specs(mut self, surface_specs: Vec<SurfaceSpec>) -> Self {
        self.surface_specs = Some(surface_specs);
        self
    }

    pub fn stop_surface(mut self, stop_surface: usize) -> Self {
        self.stop_surface = Some(stop_surface);
        self
    }

    pub fn wavelengths(mut self, wavelengths: Vec<f64>) -> Self {
        self.wavelengths = Some(wavelengths);
        self
    }

    pub fn solves(mut self, solves: Vec<Box<dyn Solve>>) -> Self {
        self.solves = solves;
        self
    }

    /// Sets the [`SurfaceRegistry`] used to resolve [`SurfaceSpec::Custom`]
    /// variants. Required when the system contains custom surfaces.
    #[cfg(feature = "serde")]
    pub fn registry(mut self, registry: SurfaceRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    fn validate(&self) -> Result<()> {
        if self.gap_specs.is_none() {
            return Err(anyhow!("Gap specs must be set"));
        }
        if self.surface_specs.is_none() {
            return Err(anyhow!("Surface specs must be set"));
        }
        if self.wavelengths.is_none() {
            return Err(anyhow!("Wavelengths must be set"));
        } else if self.wavelengths.as_ref().unwrap().is_empty() {
            return Err(anyhow!("Wavelengths cannot be empty"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::math::linalg::rotations::Rotation3D,
        core::sequential_model::SequentialSubModel,
        n,
        specs::gaps::GapSpec,
        specs::surfaces::{BoundaryType, SurfaceSpec},
    };

    /// A minimal two-surface system: Object → gap → Image.
    fn minimal_gaps() -> Vec<GapSpec> {
        vec![GapSpec {
            thickness: f64::INFINITY,
            refractive_index: n!(1.0),
        }]
    }

    fn minimal_surfaces() -> Vec<SurfaceSpec> {
        vec![
            SurfaceSpec::Object,
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
            },
        ]
    }

    /// A three-surface system with one physical surface eligible as stop.
    fn lens_gaps() -> Vec<GapSpec> {
        vec![
            GapSpec {
                thickness: f64::INFINITY,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
        ]
    }

    fn lens_surfaces() -> Vec<SurfaceSpec> {
        vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 12.5,
                radius_of_curvature: 25.8,
                surf_type: BoundaryType::Refracting,
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
            },
        ]
    }

    // --- Solve helpers ---

    /// Sets `gap_specs[gap_index].thickness` to `value` unconditionally.
    struct SetThickness {
        gap_index: usize,
        value: f64,
    }

    impl Solve for SetThickness {
        fn apply(
            &self,
            _model: &SequentialModel,
            gap_specs: &mut Vec<GapSpec>,
            _surface_specs: &mut Vec<SurfaceSpec>,
        ) -> Result<()> {
            gap_specs[self.gap_index].thickness = self.value;
            Ok(())
        }
    }

    // --- Validation tests ---

    #[test]
    fn build_fails_without_gap_specs() {
        let result = SequentialModelBuilder::new()
            .surface_specs(minimal_surfaces())
            .wavelengths(vec![0.587])
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn build_fails_without_surface_specs() {
        let result = SequentialModelBuilder::new()
            .gap_specs(minimal_gaps())
            .wavelengths(vec![0.587])
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn build_fails_without_wavelengths() {
        let result = SequentialModelBuilder::new()
            .gap_specs(minimal_gaps())
            .surface_specs(minimal_surfaces())
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn build_fails_with_empty_wavelengths() {
        let result = SequentialModelBuilder::new()
            .gap_specs(minimal_gaps())
            .surface_specs(minimal_surfaces())
            .wavelengths(vec![])
            .build();
        assert!(result.is_err());
    }

    // --- Successful build tests ---

    #[test]
    fn build_succeeds_with_required_fields() {
        let result = SequentialModelBuilder::new()
            .gap_specs(minimal_gaps())
            .surface_specs(minimal_surfaces())
            .wavelengths(vec![0.587])
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn stop_surface_is_passed_through() {
        let model = SequentialModelBuilder::new()
            .gap_specs(lens_gaps())
            .surface_specs(lens_surfaces())
            .wavelengths(vec![0.587])
            .stop_surface(1)
            .build()
            .expect("build should succeed");
        assert_eq!(model.stop_surface(), Some(1));
    }

    // --- Solve loop tests ---

    #[test]
    fn solve_modifies_gap_thickness() {
        let model = SequentialModelBuilder::new()
            .gap_specs(lens_gaps())
            .surface_specs(lens_surfaces())
            .wavelengths(vec![0.587])
            .solves(vec![Box::new(SetThickness {
                gap_index: 1,
                value: 99.0,
            })])
            .build()
            .expect("build should succeed");

        // Gap index 1 in the submodel corresponds to gap_specs[1].
        let thickness = model.submodel(0).unwrap().gaps()[1].thickness;
        assert_eq!(thickness, 99.0);
    }

    #[test]
    fn solves_are_applied_in_order() {
        // Two solves targeting the same gap; the second value should win.
        let model = SequentialModelBuilder::new()
            .gap_specs(lens_gaps())
            .surface_specs(lens_surfaces())
            .wavelengths(vec![0.587])
            .solves(vec![
                Box::new(SetThickness {
                    gap_index: 1,
                    value: 42.0,
                }),
                Box::new(SetThickness {
                    gap_index: 1,
                    value: 77.0,
                }),
            ])
            .build()
            .expect("build should succeed");

        let thickness = model.submodel(0).unwrap().gaps()[1].thickness;
        assert_eq!(thickness, 77.0);
    }
}
