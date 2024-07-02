use std::collections::HashMap;

use anyhow::Result;

use crate::core::{sequential_model::SequentialModel, Float};
use crate::specs::{
    aperture::ApertureSpec, fields::FieldSpec, gaps::GapSpec, surfaces::SurfaceSpec,
};
use crate::views::{View, ViewType, VIEW_INIT_ORDER};

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
        views: Vec<ViewType>,
    ) -> Result<Self> {
        let sequential_model = SequentialModel::new(
            aperture_spec,
            field_specs,
            gap_specs,
            surface_specs,
            wavelengths,
        )?;

        let views = Self::initialize_views(views);

        Ok(Self {
            sequential_model,
            views: views,
        })
    }

    // This can be removed once the paraxial view tests are moved to the system.
    pub fn sequential_model(&self) -> &SequentialModel {
        &self.sequential_model
    }

    fn initialize_views(views: Vec<ViewType>) -> HashMap<String, Box<dyn View>> {
        let mut views_map: HashMap<String, Box<dyn View>> = HashMap::new();

        for view_type in VIEW_INIT_ORDER.iter() {
            if views.contains(view_type) {
                match view_type {
                    ViewType::Paraxial => {
                        let view = crate::views::paraxial::ParaxialView::new();
                        views_map.insert("Paraxial".to_string(), Box::new(view));
                    }
                }
            }
        }

        views_map
    }
}
