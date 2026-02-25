use std::rc::Rc;

use anyhow::{Context, Result, bail};

use crate::{
    ApertureSpec, ConstantRefractiveIndex, FieldSpec, GapSpec, PupilSampling, Rotation3D,
    SurfaceSpec, SurfaceType,
};

use super::model::{FieldMode, SurfaceKind, SurfaceVariant, SystemSpecs};

/// Parsed core specs ready for model construction.
pub struct ParsedSpecs {
    pub surfaces: Vec<SurfaceSpec>,
    pub gaps: Vec<GapSpec>,
    pub fields: Vec<FieldSpec>,
    pub aperture: ApertureSpec,
    pub wavelengths: Vec<f64>,
}

/// Parse a string as f64, treating "Infinity" / "infinity" / "inf" as
/// `f64::INFINITY`.
fn parse_float(s: &str) -> Result<f64> {
    let trimmed = s.trim();
    match trimmed.to_lowercase().as_str() {
        "infinity" | "inf" => Ok(f64::INFINITY),
        "-infinity" | "-inf" => Ok(f64::NEG_INFINITY),
        _ => trimmed
            .parse::<f64>()
            .with_context(|| format!("cannot parse '{trimmed}' as a number")),
    }
}

/// Convert GUI `SystemSpecs` into core library specs.
pub fn convert_specs(specs: &SystemSpecs) -> Result<ParsedSpecs> {
    // --- Surfaces & Gaps ---
    let num_surfaces = specs.surfaces.len();
    if num_surfaces < 2 {
        bail!("need at least an Object and Image surface");
    }

    let mut surfaces = Vec::with_capacity(num_surfaces);
    let mut gaps = Vec::with_capacity(num_surfaces - 1);

    for (i, row) in specs.surfaces.iter().enumerate() {
        let surface = match row.variant {
            SurfaceVariant::Object => SurfaceSpec::Object,
            SurfaceVariant::Conic => {
                let semi_diameter = parse_float(&row.semi_diameter)
                    .with_context(|| format!("surface {i}: semi-diameter"))?;
                let roc = parse_float(&row.radius_of_curvature)
                    .with_context(|| format!("surface {i}: radius of curvature"))?;
                let conic = parse_float(&row.conic_constant)
                    .with_context(|| format!("surface {i}: conic constant"))?;
                let surf_type = match row.surface_kind {
                    SurfaceKind::Refracting => SurfaceType::Refracting,
                    SurfaceKind::Reflecting => SurfaceType::Reflecting,
                };
                SurfaceSpec::Conic {
                    semi_diameter,
                    radius_of_curvature: roc,
                    conic_constant: conic,
                    surf_type,
                    rotation: Rotation3D::None,
                }
            }
            SurfaceVariant::Stop => {
                let semi_diameter = parse_float(&row.semi_diameter)
                    .with_context(|| format!("surface {i}: semi-diameter"))?;
                SurfaceSpec::Stop {
                    semi_diameter,
                    rotation: Rotation3D::None,
                }
            }
            SurfaceVariant::Probe => SurfaceSpec::Probe {
                rotation: Rotation3D::None,
            },
            SurfaceVariant::Image => SurfaceSpec::Image {
                rotation: Rotation3D::None,
            },
        };
        surfaces.push(surface);

        // Every surface except the last has a gap after it.
        if i < num_surfaces - 1 {
            let thickness =
                parse_float(&row.thickness).with_context(|| format!("surface {i}: thickness"))?;
            let n = parse_float(&row.refractive_index)
                .with_context(|| format!("surface {i}: refractive index"))?;
            let ri: Rc<dyn crate::RefractiveIndexSpec> =
                Rc::new(ConstantRefractiveIndex::new(n, 0.0));
            gaps.push(GapSpec {
                thickness,
                refractive_index: ri,
            });
        }
    }

    // --- Fields ---
    if specs.fields.is_empty() {
        bail!("need at least one field point");
    }

    let mut fields = Vec::with_capacity(specs.fields.len());
    for (i, frow) in specs.fields.iter().enumerate() {
        let spacing = parse_float(&frow.pupil_spacing)
            .with_context(|| format!("field {i}: pupil spacing"))?;
        let pupil_sampling = PupilSampling::SquareGrid { spacing };

        let field = match specs.field_mode {
            FieldMode::Angle => {
                let angle =
                    parse_float(&frow.value).with_context(|| format!("field {i}: angle"))?;
                FieldSpec::Angle {
                    angle,
                    pupil_sampling,
                }
            }
            FieldMode::PointSource => {
                let y = parse_float(&frow.value).with_context(|| format!("field {i}: y"))?;
                let x = parse_float(&frow.x).with_context(|| format!("field {i}: x"))?;
                FieldSpec::PointSource {
                    x,
                    y,
                    pupil_sampling,
                }
            }
        };
        fields.push(field);
    }

    // --- Aperture ---
    let aperture_sd = parse_float(&specs.aperture_semi_diameter).context("aperture")?;
    let aperture = ApertureSpec::EntrancePupil {
        semi_diameter: aperture_sd,
    };

    // --- Wavelengths ---
    if specs.wavelengths.is_empty() {
        bail!("need at least one wavelength");
    }
    let mut wavelengths = Vec::with_capacity(specs.wavelengths.len());
    for (i, w) in specs.wavelengths.iter().enumerate() {
        let wl = parse_float(w).with_context(|| format!("wavelength {i}"))?;
        wavelengths.push(wl);
    }

    Ok(ParsedSpecs {
        surfaces,
        gaps,
        fields,
        aperture,
        wavelengths,
    })
}
