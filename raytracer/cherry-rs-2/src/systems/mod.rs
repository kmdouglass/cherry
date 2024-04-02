use crate::specs::{fields::FieldSpec, gaps::GapSpec};

#[derive(Debug)]
pub struct SeqSystem {
    fields: Vec<FieldSpec>,
    gaps: Vec<GapSpec>,
}

/// Builds a sequential optical system from user specs.
#[derive(Debug)]
pub struct SeqSysBuilder {
    fields: Option<Vec<FieldSpec>>,
    gaps: Option<Vec<GapSpec>>,
}

impl SeqSysBuilder {
    /// Creates a new sequential optical system builder.
    pub fn new() -> Self {
        Self {
            fields: None,
            gaps: None,
        }
    }

    /// Sets the fields of the optical system.
    pub fn fields(mut self, fields: Vec<FieldSpec>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Sets the gaps of the optical system.
    pub fn gaps(mut self, gaps: Vec<GapSpec>) -> Self {
        self.gaps = Some(gaps);
        self
    }

    /// Builds the sequential optical system.
    pub fn build(self) -> SeqSystem {
        SeqSystem {
            fields: self.fields.unwrap(),
            gaps: self.gaps.unwrap(),
        }
    }
}
