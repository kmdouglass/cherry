use std::collections::HashMap;

use anyhow::{anyhow, Result};

use cherry_rs_2::{
    ray_trace_3d_view, ApertureSpec, FieldSpec, GapSpec, ParaxialView, SequentialModel, SubModelID,
    SurfaceSpec, TraceResults,
};

#[derive(Debug)]
pub struct System {
    sequential_model: SequentialModel,
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

impl System {
    fn new(
        aperture_spec: &ApertureSpec,
        field_specs: &[FieldSpec],
        gap_specs: &[GapSpec],
        surface_specs: &[SurfaceSpec],
        wavelengths: &[f64],
    ) -> Result<Self> {
        let sequential_model = SequentialModel::new(gap_specs, surface_specs, wavelengths)?;

        let paraxial_view = ParaxialView::new(&sequential_model, false);

        Ok(Self {
            sequential_model,
            paraxial_view,
            aperture_spec: aperture_spec.clone(),
            field_specs: field_specs.to_vec(),
        })
    }

    pub fn trace(&self) -> HashMap<SubModelID, TraceResults> {
        ray_trace_3d_view(
            &self.aperture_spec,
            &self.field_specs,
            &self.sequential_model,
            &self.paraxial_view,
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