/// Description of an optical system.
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::math::vec3::Vec3;
use crate::ray_tracing::{
    ApertureSpec, Component, FieldSpec, Gap, Surface, SurfaceSpec, SystemModel,
};

const NUM_SAMPLES_PER_SURFACE: usize = 20;

type Diameters = HashMap<usize, f32>;
type SurfaceSamples = HashMap<usize, Vec<Vec3>>;
type SurfaceTypes = HashMap<usize, String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemDescription {
    inputs: Inputs,
    component_model: ComponentModelDescr,
    surface_model: SurfaceModelDescr,
}

impl SystemDescription {
    pub fn new(system: &SystemModel) -> Self {
        let surfaces = system.surf_model.surfaces();
        let component_model = ComponentModelDescr::new(system.comp_model.components());
        let surface_model = SurfaceModelDescr::new(surfaces, NUM_SAMPLES_PER_SURFACE);

        let inputs = Inputs {
            surface_specs: system.surface_specs().to_owned(),
            gap_specs: system.gap_specs().to_owned(),
            aperture_spec: system.aperture_spec().to_owned(),
            field_specs: system.field_specs().to_owned(),
        };

        Self {
            inputs,
            component_model: component_model,
            surface_model,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inputs {
    surface_specs: Vec<SurfaceSpec>,
    gap_specs: Vec<Gap>,
    aperture_spec: ApertureSpec,
    field_specs: Vec<FieldSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComponentModelDescr {
    components: HashSet<Component>,
}

impl ComponentModelDescr {
    fn new(components: &HashSet<Component>) -> Self {
        Self {
            components: components.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SurfaceModelDescr {
    diameters: Diameters,
    surface_samples: SurfaceSamples,
    surface_types: SurfaceTypes,
}

impl SurfaceModelDescr {
    fn new(surfaces: &[Surface], num_samples_per_surf: usize) -> Self {
        let mut diameters = HashMap::new();
        let mut surface_samples = HashMap::new();
        let mut surface_types = HashMap::new();

        for (idx, surface) in surfaces.iter().enumerate() {
            let samples = surface.sample_yz(num_samples_per_surf);
            surface_samples.insert(idx, samples);

            diameters.insert(idx, surface.diam());

            let surface_type = match surface {
                Surface::ObjectPlane(_) => "ObjectPlane".to_string(),
                Surface::ImagePlane(_) => "ImagePlane".to_string(),
                Surface::RefractingCircularConic(_) => "RefractingCircularConic".to_string(),
                Surface::RefractingCircularFlat(_) => "RefractingCircularFlat".to_string(),
                Surface::Stop(_) => "Stop".to_string(),
            };
            surface_types.insert(idx, surface_type);
        }

        Self {
            diameters: diameters,
            surface_samples: surface_samples,
            surface_types: surface_types,
        }
    }
}
