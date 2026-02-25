use std::rc::Rc;

use anyhow::{Context, Result, bail};

use crate::{
    ApertureSpec, ConstantRefractiveIndex, FieldSpec, GapSpec, PupilSampling, RefractiveIndexSpec,
    Rotation3D, SurfaceSpec, SurfaceType,
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

/// Materials map type used when the ri-info feature is enabled.
#[cfg(feature = "ri-info")]
pub type MaterialsMap = std::collections::HashMap<String, Rc<lib_ria::Material>>;

/// Convert GUI `SystemSpecs` into core library specs.
///
/// When ri-info is enabled, pass the materials map so material keys can be
/// resolved to `RefractiveIndexSpec` implementations.
#[cfg(feature = "ri-info")]
pub fn convert_specs(
    specs: &SystemSpecs,
    materials: &MaterialsMap,
) -> Result<ParsedSpecs> {
    convert_specs_inner(specs, Some(materials))
}

#[cfg(not(feature = "ri-info"))]
pub fn convert_specs(specs: &SystemSpecs) -> Result<ParsedSpecs> {
    convert_specs_inner(specs)
}

fn convert_specs_inner(
    specs: &SystemSpecs,
    #[cfg(feature = "ri-info")] materials: Option<&MaterialsMap>,
) -> Result<ParsedSpecs> {
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
            let ri = resolve_refractive_index(
                i,
                row,
                specs.use_materials,
                #[cfg(feature = "ri-info")]
                materials,
            )?;
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

/// Resolve the refractive index for a gap, using either a material key or a
/// constant n value.
fn resolve_refractive_index(
    surface_idx: usize,
    row: &super::model::SurfaceRow,
    use_materials: bool,
    #[cfg(feature = "ri-info")] materials: Option<&MaterialsMap>,
) -> Result<Rc<dyn RefractiveIndexSpec>> {
    #[cfg(feature = "ri-info")]
    if use_materials {
        if let Some(key) = &row.material_key {
            let materials =
                materials.ok_or_else(|| anyhow::anyhow!("Material store not loaded"))?;
            let mat = materials
                .get(key)
                .ok_or_else(|| anyhow::anyhow!("surface {surface_idx}: material '{key}' not found in database"))?;
            return Ok(Rc::clone(mat) as Rc<dyn RefractiveIndexSpec>);
        }
    }

    #[cfg(not(feature = "ri-info"))]
    if use_materials {
        bail!("Material mode requires the ri-info feature");
    }

    // Fall back to constant n.
    let n = parse_float(&row.refractive_index)
        .with_context(|| format!("surface {surface_idx}: refractive index"))?;
    Ok(Rc::new(ConstantRefractiveIndex::new(n, 0.0)))
}
