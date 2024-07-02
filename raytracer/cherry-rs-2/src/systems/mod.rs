use std::collections::HashMap;

use anyhow::Result;

use crate::core::{sequential_model::SequentialModel, Float};
use crate::specs::{
    aperture::ApertureSpec, fields::FieldSpec, gaps::GapSpec, surfaces::SurfaceSpec,
};
use crate::views::View;

pub struct System {
    sequential_model: SequentialModel,
    views: HashMap<String, Box<dyn View>>,
}

impl System {
    pub fn new(
        aperture_spec: ApertureSpec,
        field_specs: Vec<FieldSpec>,
        gap_specs: Vec<GapSpec>,
        surface_specs: Vec<SurfaceSpec>,
        wavelengths: Vec<Float>,
        views: Vec<Box<dyn View>>,
    ) -> Result<Self> {
        let sequential_model = SequentialModel::new(
            aperture_spec,
            field_specs,
            gap_specs,
            surface_specs,
            wavelengths,
        )?;

        let mut views = Self::resolve_views(views);

        Ok(Self {
            sequential_model,
            views: views,
        })
    }

    // This can be removed once the paraxial view tests are moved to the system.
    pub fn sequential_model(&self) -> &SequentialModel {
        &self.sequential_model
    }

    /// Resolve the views and their dependencies into a single HashMap.
    fn resolve_views(requested_views: Vec<Box<dyn View>>) -> HashMap<String, Box<dyn View>> {
        let mut views: HashMap<String, Box<dyn View>> = HashMap::new();

        for view in requested_views {
            // Get the dependencies first; otherwise we lose the reference to the view upon insertion into the HashMap.
            let mut dependencies = view.dependencies();

            views.entry(view.name().to_owned()).or_insert(view);
            for dependency in dependencies.drain(..) {
                views
                    .entry(dependency.name().to_owned())
                    .or_insert(dependency);
            }
        }

        views
    }
}
