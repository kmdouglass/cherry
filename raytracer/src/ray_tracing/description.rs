/// Description of an optical system.
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::math::vec3::Vec3;
use crate::ray_tracing::{Component, Surface, SystemModel};

const NUM_SAMPLES_PER_SURFACE: usize = 20;

type SurfaceSamples = HashMap<usize, Vec<Vec3>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemDescription {
    component_model: ComponentModelDescr,
    sequential_model: SequentialModelDescr,
}

impl SystemDescription {
    pub fn new(system: &SystemModel) -> Self {
        let surfaces = system.seq_model.surfaces();
        let component_model = ComponentModelDescr::new(system.comp_model.components());
        let sequential_model = SequentialModelDescr::new(surfaces, NUM_SAMPLES_PER_SURFACE);

        Self {
            component_model: component_model,
            sequential_model: sequential_model,
        }
    }
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
struct SequentialModelDescr {
    surface_samples: SurfaceSamples,
}

impl SequentialModelDescr {
    fn new(surfaces: &[Surface], num_samples_per_surf: usize) -> Self {
        let mut surface_samples = HashMap::new();
        for (idx, surface) in surfaces.iter().enumerate() {
            let samples = surface.sample_yz(num_samples_per_surf);
            surface_samples.insert(idx, samples);
        }

        Self {
            surface_samples: surface_samples,
        }
    }
}
