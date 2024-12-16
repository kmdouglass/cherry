//! A material's refractive index.
use anyhow::{anyhow, Result};

use crate::core::{math::Complex, Float};
use crate::specs::gaps::{ImagSpec, RealSpec, RefractiveIndexSpec};

#[derive(Debug, Clone, Copy)]
pub struct RefractiveIndex {
    eta: Complex<Float>,
}

impl RefractiveIndex {
    pub(crate) fn new(n: Float, k: Float) -> Self {
        Self {
            eta: Complex { real: n, imag: k },
        }
    }

    pub(crate) fn n(&self) -> Float {
        self.eta.real
    }

    pub(crate) fn k(&self) -> Float {
        self.eta.imag
    }

    /// Creates a RefractiveIndex instance from a RefractiveIndexSpec.
    ///
    /// A wavelength is required to compute the refractive index from the spec
    /// if the refractive index is specified as a function of wavelength.
    /// Otherwise, the real part of the refractive index is provided by the user
    /// as a constant value.
    pub(crate) fn try_from_spec(
        spec: &RefractiveIndexSpec,
        wavelength: Option<Float>,
    ) -> Result<Self> {
        if wavelength.is_none() && spec.depends_on_wavelength() {
            return Err(anyhow!(
                "The refractive index must be a constant when no wavelength is provided."
            ));
        }

        // If no wavelength is provided, set it to 0. It won't be used anyway.
        let wavelength = if let Some(wavelength) = wavelength {
            wavelength
        } else {
            0.0
        };

        let n: Float = match &spec.real {
            RealSpec::Constant(n) => *n,
            RealSpec::Formula1 {
                wavelength_range,
                c,
            } => {
                // Sellmeier (preferred)
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }
                let mut sum = 0.0;
                for i in (1..c.len()).step_by(2) {
                    sum += c[i] * wavelength.powi(2) / (wavelength.powi(2) - c[i + 1].powi(2));
                }
                (1.0 + c[0] + sum).sqrt()
            }
            RealSpec::Formula2 {
                wavelength_range,
                c,
            } => {
                // Sellmeier-2
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }
                let mut sum = 0.0;
                for i in (1..c.len()).step_by(2) {
                    sum += c[i] * wavelength.powi(2) / (wavelength.powi(2) - c[i + 1]);
                }
                (1.0 + c[0] + sum).sqrt()
            }
            RealSpec::Formula3 {
                wavelength_range,
                c,
            } => {
                // Polynomial
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }

                let mut sum = 0.0;
                for i in (1..c.len()).step_by(2) {
                    sum += c[i] * wavelength.powf(c[i + 1]);
                }
                (c[0] + sum).sqrt()
            }
            RealSpec::Formula4 {
                wavelength_range,
                c,
            } => {
                // RefractiveIndex.INFO
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }

                let mut sum = 0.0;
                for i in (1..c.len()).step_by(4) {
                    // Formula 4 is kind of wild.
                    if i <= 9 {
                        sum += c[i] * wavelength.powf(c[i + 1])
                            / (wavelength.powi(2) - c[i + 2].powf(c[i + 3]));
                    } else {
                        sum += c[i] * wavelength.powf(c[i + 1]);
                    }
                }
                (c[0] + sum).sqrt()
            }
            RealSpec::Formula5 {
                wavelength_range,
                c,
            } => {
                // Cauchy
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }

                let mut sum = 0.0;
                for i in (1..c.len()).step_by(2) {
                    sum += c[i] * wavelength.powf(c[i + 1]);
                }
                c[0] + sum
            }
            RealSpec::Formula6 {
                wavelength_range,
                c,
            } => {
                // Gases
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }

                let mut sum = 0.0;
                for i in (1..c.len()).step_by(2) {
                    sum += c[i] / (c[i + 1] - wavelength.powi(-2));
                }
                1.0 + c[0] + sum
            }
            RealSpec::Formula7 {
                wavelength_range,
                c,
            } => {
                // Herzberger
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }
                let mut sum = 0.0;
                for i in (3..c.len()).step_by(2) {
                    sum += c[i] * wavelength.powi(i as i32 - 1);
                }
                c[0] + c[1] / (wavelength.powi(2) - 0.028)
                    + c[2] / (wavelength.powi(2) - 0.028).powi(2)
                    + sum
            }
            RealSpec::Formula8 {
                wavelength_range,
                c,
            } => {
                // Retro
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }

                let sum = c[0]
                    + c[1] * wavelength.powi(2) / (wavelength.powi(2) - c[2])
                    + c[3] * wavelength.powi(2);
                println!("sum: {}", sum);
                ((2.0 * sum + 1.0) / (1.0 - sum)).sqrt()
            }
            RealSpec::Formula9 {
                wavelength_range,
                c,
            } => {
                // Exotic
                if wavelength < wavelength_range[0] || wavelength > wavelength_range[1] {
                    return Err(anyhow!(
                        "The wavelength is outside the range of the real spec."
                    ));
                }

                (c[0]
                    + c[1] / (wavelength.powi(2) - c[2])
                    + c[3] * (wavelength - c[4]) / ((wavelength - c[4]).powi(2) + c[5]))
                    .sqrt()
            }
            _ => {
                return Err(anyhow!(
                    "Tabulated real refractive indexes are not implemented."
                ));
            }
        };

        let k = match spec.imag {
            Some(ImagSpec::Constant(k)) => k,
            None => 0.0,
            _ => {
                return Err(anyhow!(
                    "Non-constant imaginary parts of refractive indexes are not implemented."
                ))
            }
        };

        Ok(Self::new(n, k))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_refractive_index_try_from_spec() {
        let spec = RefractiveIndexSpec {
            real: RealSpec::Constant(1.5),
            imag: None,
        };

        let n = RefractiveIndex::try_from_spec(&spec, None).unwrap();
        assert_eq!(n.eta.real, 1.5);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_1() {
        // Water Ice at 150 K from refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula1 {
                wavelength_range: [0.210, 0.757],
                c: vec![0.0, 0.496, 0.071, 0.190, 0.134],
            },
            imag: None,
        };
        let n = RefractiveIndex::try_from_spec(&spec, Some(0.5876)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 1.3053, epsilon = 1e-4);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_2() {
        // N-BK7 from refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula2 {
                wavelength_range: [0.3, 2.5],
                c: vec![
                    0.0,
                    1.03961212,
                    0.00600069867,
                    0.231792344,
                    0.0200179144,
                    1.01046945,
                    103.560653,
                ],
            },
            imag: None,
        };

        let n = RefractiveIndex::try_from_spec(&spec, Some(0.5876)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 1.51680, epsilon = 1e-5);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_3() {
        // Ohara BAH10 from refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula3 {
                wavelength_range: [0.365, 0.9],
                c: vec![
                    2.730459,
                    -0.01063385,
                    2.0,
                    0.01942756,
                    -2.0,
                    0.0008209873,
                    -4.0,
                    -5.210457e-05,
                    -6.0,
                    4.447534e-06,
                    -8.0,
                ],
            },
            imag: None,
        };
        let n = RefractiveIndex::try_from_spec(&spec, Some(0.5876)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 1.6700, epsilon = 1e-4);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_4() {
        // CH4N20 Urea from refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula4 {
                wavelength_range: [0.3, 1.06],
                c: vec![2.1823, 0.0125, 0.0, 0.0300, 1.0, 0.0, 0.0, 0.0, 1.0],
            },
            imag: None,
        };
        let n = RefractiveIndex::try_from_spec(&spec, Some(0.5876)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 1.4906, epsilon = 1e-4);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_5() {
        // BK7 matching liquid from refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula5 {
                wavelength_range: [0.31, 1.55],
                c: vec![1.502787, 455872.4E-8, -2.0, 9.844856E-5, -4.0],
            },
            imag: None,
        };
        let n = RefractiveIndex::try_from_spec(&spec, Some(0.5876)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 1.5168, epsilon = 1e-4);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_6() {
        // H2 (Peck) in main shelf from refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula6 {
                wavelength_range: [0.168, 1.6945],
                c: vec![0.0, 0.0148956, 180.7, 0.0049037, 92.0],
            },
            imag: None,
        };
        let n = RefractiveIndex::try_from_spec(&spec, Some(0.5876)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 1.00013881, epsilon = 1e-8);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_7() {
        // Si (Edwards) in main shelf of refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula7 {
                wavelength_range: [2.4373, 25.0],
                c: vec![3.41983, 0.159906, -0.123109, 1.26878E-6, -1.95104E-9],
            },
            imag: None,
        };
        let n = RefractiveIndex::try_from_spec(&spec, Some(2.4373)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 3.4434, epsilon = 1e-4);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_8() {
        // TlCl (Schroter) in main shelf of refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula8 {
                wavelength_range: [0.43, 0.66],
                c: vec![0.47856, 0.07858, 0.08277, -0.00881],
            },
            imag: None,
        };
        let n = RefractiveIndex::try_from_spec(&spec, Some(0.5876)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 2.2636, epsilon = 1e-4);
        assert_eq!(n.eta.imag, 0.0);
    }

    #[test]
    fn test_try_from_spec_formula_9() {
        // CH4N2O Urea (Rosker-e) from refractiveindex.info
        let spec = RefractiveIndexSpec {
            real: RealSpec::Formula9 {
                wavelength_range: [0.3, 1.06],
                c: vec![2.51527, 0.0240, 0.0300, 0.020, 1.52, 0.8771],
            },
            imag: None,
        };
        let n = RefractiveIndex::try_from_spec(&spec, Some(0.5876)).unwrap();
        assert_abs_diff_eq!(n.eta.real, 1.6065, epsilon = 1e-4);
        assert_eq!(n.eta.imag, 0.0);
    }
}
