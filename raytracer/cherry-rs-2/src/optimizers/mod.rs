/// Prototype optimizers.
use anyhow::Result;

use crate::systems::SeqSys;

pub(crate) trait Optimizer {
    fn optimize(&self, system: &mut SeqSys) -> Result<()>;
}

struct DummyOptimizer;

impl Optimizer for DummyOptimizer {
    fn optimize(&self, system: &mut SeqSys) -> Result<()> {
        Ok(())
    }
}
