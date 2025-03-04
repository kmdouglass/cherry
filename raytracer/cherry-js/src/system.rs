use std::collections::HashSet;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use cherry_rs::{
    components_view, n, ray_trace_3d_view_v2, ApertureSpec, Component, CutawayView, FieldSpec,
    GapSpec, Material, ParaxialView, ParaxialViewDescription, PupilSampling, RefractiveIndexSpec,
    SequentialModel, SurfaceSpec, TraceResultsCollection,
};

const BACKGROUND_REFRACTIVE_INDEX: f64 = 1.0;

/// Handles the refractive index gap specs from the JS side.
///
/// These will be converted to `GapSpec` instances, which contain trait objects that cannot be
/// serialized/deserialized.
#[derive(Debug, Serialize, Deserialize)]
pub struct GapSpecConstantN {
    pub thickness: f64,
    pub refractive_index: f64,
}

/// Handles the material gap specs from the JS side.
///
/// These will be converted to `GapSpec` instances, which contain trait objects that cannot be
/// serialized/deserialized.
#[derive(Debug, Serialize, Deserialize)]
pub struct GapSpecMaterial {
    pub thickness: f64,
    pub material: Material,
}

#[derive(Debug)]
pub struct System {
    sequential_model: SequentialModel,
    components_view: HashSet<Component>,
    cutaway_view: CutawayView,
    paraxial_view: ParaxialView,

    // Cache specs for calculations later
    aperture_spec: ApertureSpec,
    field_specs: Vec<FieldSpec>,
}

#[derive(Debug)]
pub struct SystemBuilder {
    aperture_spec: Option<ApertureSpec>,
    field_specs: Vec<FieldSpec>,
    gap_specs: Vec<GapSpec>,
    surface_specs: Vec<SurfaceSpec>,
    wavelengths: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct SystemDescription {
    pub components_view: HashSet<Component>,
    pub cutaway_view: CutawayView,
    pub paraxial_view: ParaxialViewDescription,
}

impl From<GapSpecConstantN> for GapSpec {
    fn from(gap: GapSpecConstantN) -> Self {
        GapSpec {
            thickness: gap.thickness,
            refractive_index: n!(gap.refractive_index),
        }
    }
}

impl From<GapSpecMaterial> for GapSpec {
    fn from(gap: GapSpecMaterial) -> Self {
        let material: Rc<dyn RefractiveIndexSpec> = Rc::new(gap.material);

        GapSpec {
            thickness: gap.thickness,
            refractive_index: material,
        }
    }
}

impl System {
    fn new(
        aperture_spec: &ApertureSpec,
        field_specs: &[FieldSpec],
        gap_specs: &[GapSpec],
        surface_specs: &[SurfaceSpec],
        wavelengths: &[f64],
    ) -> Result<Self> {
        let sequential_model = SequentialModel::new(gap_specs, surface_specs, wavelengths)?;

        let components_view = components_view(&sequential_model, n!(BACKGROUND_REFRACTIVE_INDEX))?;
        let cutaway_view = CutawayView::new(&sequential_model, 20);
        let paraxial_view = ParaxialView::new(&sequential_model, field_specs, false)?;

        Ok(Self {
            sequential_model,
            components_view,
            cutaway_view,
            paraxial_view,
            aperture_spec: *aperture_spec,
            field_specs: field_specs.to_vec(),
        })
    }

    pub fn describe(&self) -> SystemDescription {
        SystemDescription {
            components_view: self.components_view.clone(),
            cutaway_view: self.cutaway_view.clone(),
            paraxial_view: self.paraxial_view.describe(),
        }
    }

    pub fn trace(&self) -> Result<TraceResultsCollection> {
        ray_trace_3d_view_v2(
            &self.aperture_spec,
            &self.field_specs,
            &self.sequential_model,
            &self.paraxial_view,
            None,
        )
    }

    pub fn trace_chief_and_marginal_rays(&self) -> Result<TraceResultsCollection> {
        ray_trace_3d_view_v2(
            &self.aperture_spec,
            &self.field_specs,
            &self.sequential_model,
            &self.paraxial_view,
            Some(PupilSampling::ChiefAndMarginalRays),
        )
    }
}

impl SystemBuilder {
    pub fn new() -> Self {
        Self {
            aperture_spec: None,
            field_specs: Vec::new(),
            gap_specs: Vec::new(),
            surface_specs: Vec::new(),
            wavelengths: Vec::new(),
        }
    }

    pub fn aperture_spec(&mut self, aperture_spec: ApertureSpec) -> &mut Self {
        self.aperture_spec = Some(aperture_spec);
        self
    }

    pub fn field_specs(&mut self, field_specs: Vec<FieldSpec>) -> &mut Self {
        self.field_specs = field_specs;
        self
    }

    pub fn gap_specs(&mut self, gaps: Vec<GapSpec>) -> &mut Self {
        self.gap_specs = gaps;
        self
    }

    pub fn surface_specs(&mut self, surface_specs: Vec<SurfaceSpec>) -> &mut Self {
        self.surface_specs = surface_specs;
        self
    }

    pub fn wavelengths(&mut self, wavelengths: Vec<f64>) -> &mut Self {
        self.wavelengths = wavelengths;
        self
    }

    pub fn build(&self) -> Result<System> {
        let aperture_spec = self
            .aperture_spec
            .ok_or(anyhow!("The system aperture must be specified."))?;
        let model = System::new(
            &aperture_spec,
            &self.field_specs,
            &self.gap_specs,
            &self.surface_specs,
            &self.wavelengths,
        )?;

        Ok(model)
    }
}
