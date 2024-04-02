use serde::{Deserialize, Serialize};

use crate::core::Float;

#[derive(Serialize, Deserialize, Debug)]
pub enum RefractiveIndexSpec {
    N { n: DataSpec },
    NAndKSeparate { n: DataSpec, k: DataSpec },
    NAndKTogether { nk: DataSpec },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DataSpec {
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
