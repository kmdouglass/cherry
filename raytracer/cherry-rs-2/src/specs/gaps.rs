use serde::{Deserialize, Serialize};

use crate::core::Float;

#[derive(Serialize, Deserialize, Debug)]
pub struct GapSpec {
    pub thickness: Float,
    pub refractive_index: RefractiveIndexSpec,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RefractiveIndexSpec {
    N { n: RIDataSpec },
    NAndKSeparate { n: RIDataSpec, k: RIDataSpec },
    NAndKTogether { nk: RIDataSpec },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RIDataSpec {
    Constant(Float),
    TabulatedN {
        data: Vec<[Float; 2]>,
    },
    TabulatedK {
        data: Vec<[Float; 2]>,
    },
    TabulatedNK {
        data: Vec<[Float; 3]>,
    },
    Formula1 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
    Formula2 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
    Formula3 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
    Formula4 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
    Formula5 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
    Formula6 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
    Formula7 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
    Formula8 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
    Formula9 {
        wavelength_range: [Float; 2],
        coefficients: Vec<Float>,
    },
}
