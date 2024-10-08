use serde::{Deserialize, Serialize};

use crate::core::Float;

/// Specifies a gap in a sequential optical system model.
#[derive(Serialize, Deserialize, Debug)]
pub struct GapSpec {
    pub thickness: Float,
    pub refractive_index: RefractiveIndexSpec,
}

/// Specifies the refractive index of the material constituting a gap.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RefractiveIndexSpec {
    pub real: RealSpec,
    pub imag: Option<ImagSpec>,
}

/// Specifies the real part of a refractive index.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RealSpec {
    Constant(Float),
    TabulatedN {
        data: Vec<[Float; 2]>,
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

/// Specifies the imaginary part of a refractive index.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ImagSpec {
    Constant(Float),
    TabulatedK { data: Vec<[Float; 2]> },
}

impl GapSpec {
    pub fn from_thickness_and_real_refractive_index(thickness: Float, n: Float) -> Self {
        let refractive_index = RefractiveIndexSpec {
            real: RealSpec::Constant(n),
            imag: None,
        };
        Self {
            thickness,
            refractive_index,
        }
    }
}

impl RefractiveIndexSpec {
    pub fn depends_on_wavelength(&self) -> bool {
        !self.is_constant()
    }

    pub fn is_constant(&self) -> bool {
        let is_real_part_const = matches!(&self.real, RealSpec::Constant(_));
        let is_imag_part_const = matches!(&self.imag, Some(ImagSpec::Constant(_)) | None);

        is_real_part_const && is_imag_part_const
    }

    pub fn from_real_refractive_index(n: Float) -> Self {
        Self {
            real: RealSpec::Constant(n),
            imag: None,
        }
    }
}
