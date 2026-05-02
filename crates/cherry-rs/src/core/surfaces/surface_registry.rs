use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::core::surfaces::Surface;

/// A function that constructs a [`Surface`] from a JSON parameter blob.
pub type SurfaceConstructor = fn(&Value) -> Result<Box<dyn Surface>>;

/// A registry that maps surface type identifiers to constructor functions.
///
/// Used with [`SequentialModelBuilder`] to build models containing
/// [`SurfaceSpec::Custom`] variants defined outside the cherry-rs crate.
///
/// [`SequentialModelBuilder`]: crate::core::sequential_model::builder::SequentialModelBuilder
/// [`SurfaceSpec::Custom`]: crate::specs::surfaces::SurfaceSpec::Custom
#[derive(Clone)]
pub struct SurfaceRegistry {
    constructors: HashMap<String, SurfaceConstructor>,
}

impl SurfaceRegistry {
    pub fn new() -> Self {
        Self {
            constructors: HashMap::new(),
        }
    }

    /// Register a constructor for the given `type_id`.
    ///
    /// If a constructor was already registered for this `type_id`, it is
    /// replaced.
    pub fn register(&mut self, type_id: &str, constructor: SurfaceConstructor) {
        self.constructors.insert(type_id.to_string(), constructor);
    }

    /// Build a [`Surface`] for the given `type_id` using `params`.
    pub fn build(&self, type_id: &str, params: &Value) -> Result<Box<dyn Surface>> {
        let constructor = self
            .constructors
            .get(type_id)
            .ok_or_else(|| anyhow!("no surface constructor registered for type '{type_id}'"))?;
        constructor(params)
    }
}

impl Default for SurfaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
