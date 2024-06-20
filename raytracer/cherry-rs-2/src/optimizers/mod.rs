/// Prototype optimizers.
use anyhow::Result;

use crate::systems::SequentialModel;

pub(crate) trait Optimizer {
    fn optimize(&self, system: &mut SequentialModel) -> Result<()>;
}

struct DummyOptimizer;

impl Optimizer for DummyOptimizer {
    fn optimize(&self, system: &mut SequentialModel) -> Result<()> {
        Ok(())
    }
}
