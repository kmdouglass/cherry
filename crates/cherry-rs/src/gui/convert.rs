use std::rc::Rc;

use anyhow::{Context, Result, bail};

use crate::{
    ApertureSpec, BoundaryKind, ConstantRefractiveIndex, EulerAngles, FNumberSolve, FieldSpec,
    GapSpec, MarginalRaySolve, RefractiveIndexSpec, Rotation3D, Solve, SurfaceSpec, Vec3,
};

use super::model::{FieldMode, SolveSpec, SurfaceKind, SurfaceRow, SurfaceVariant, SystemSpecs};

/// Parsed core specs ready for model construction.
pub struct ParsedSpecs {
    pub surfaces: Vec<SurfaceSpec>,
    pub gaps: Vec<GapSpec>,
    pub fields: Vec<FieldSpec>,
    pub aperture: ApertureSpec,
    pub wavelengths: Vec<f64>,
    pub background: std::rc::Rc<dyn RefractiveIndexSpec>,
    pub solves: Vec<Box<dyn Solve>>,
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

fn parse_decenter(row: &SurfaceRow, i: usize) -> Result<Vec3> {
    let r = parse_float(&row.decenter_r).with_context(|| format!("surface {i}: decenter R"))?;
    let u = parse_float(&row.decenter_u).with_context(|| format!("surface {i}: decenter U"))?;
    let f = parse_float(&row.decenter_f).with_context(|| format!("surface {i}: decenter F"))?;
    Ok(Vec3::new(r, u, f))
}

fn parse_rotation_offset(row: &SurfaceRow, i: usize) -> Result<Rotation3D> {
    let theta = parse_float(&row.rot_offset_theta)
        .with_context(|| format!("surface {i}: rot_offset theta"))?;
    let psi =
        parse_float(&row.rot_offset_psi).with_context(|| format!("surface {i}: rot_offset psi"))?;
    let phi =
        parse_float(&row.rot_offset_phi).with_context(|| format!("surface {i}: rot_offset phi"))?;
    if theta == 0.0 && psi == 0.0 && phi == 0.0 {
        Ok(Rotation3D::None)
    } else {
        Ok(Rotation3D::IntrinsicPassiveRUF(EulerAngles(
            theta.to_radians(),
            psi.to_radians(),
            phi.to_radians(),
        )))
    }
}

/// Convert GUI `SystemSpecs` into core library specs.
///
/// When ri-info is enabled, pass the materials map so material keys can be
/// resolved to `RefractiveIndexSpec` implementations.
#[cfg(feature = "ri-info")]
pub fn convert_specs(specs: &SystemSpecs, materials: &MaterialsMap) -> Result<ParsedSpecs> {
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
                let surf_kind = match row.surface_kind {
                    SurfaceKind::Refracting => BoundaryKind::Refracting,
                    SurfaceKind::Reflecting => BoundaryKind::Reflecting,
                };
                let rotation = if matches!(surf_kind, BoundaryKind::Reflecting) {
                    let theta_deg =
                        parse_float(&row.theta).with_context(|| format!("surface {i}: theta"))?;
                    let psi_deg =
                        parse_float(&row.psi).with_context(|| format!("surface {i}: psi"))?;
                    if theta_deg == 0.0 && psi_deg == 0.0 {
                        Rotation3D::None
                    } else {
                        Rotation3D::IntrinsicPassiveRUF(EulerAngles(
                            theta_deg.to_radians(),
                            psi_deg.to_radians(),
                            0.0,
                        ))
                    }
                } else {
                    Rotation3D::None
                };
                SurfaceSpec::Conic {
                    semi_diameter,
                    radius_of_curvature: roc,
                    conic_constant: conic,
                    surf_kind,
                    rotation,
                    decenter: parse_decenter(row, i)?,
                    rotation_offset: parse_rotation_offset(row, i)?,
                }
            }
            SurfaceVariant::Sphere => {
                let semi_diameter = parse_float(&row.semi_diameter)
                    .with_context(|| format!("surface {i}: semi-diameter"))?;
                let roc = parse_float(&row.radius_of_curvature)
                    .with_context(|| format!("surface {i}: radius of curvature"))?;
                let surf_kind = match row.surface_kind {
                    SurfaceKind::Refracting => BoundaryKind::Refracting,
                    SurfaceKind::Reflecting => BoundaryKind::Reflecting,
                };
                let rotation = if matches!(surf_kind, BoundaryKind::Reflecting) {
                    let theta_deg =
                        parse_float(&row.theta).with_context(|| format!("surface {i}: theta"))?;
                    let psi_deg =
                        parse_float(&row.psi).with_context(|| format!("surface {i}: psi"))?;
                    if theta_deg == 0.0 && psi_deg == 0.0 {
                        Rotation3D::None
                    } else {
                        Rotation3D::IntrinsicPassiveRUF(EulerAngles(
                            theta_deg.to_radians(),
                            psi_deg.to_radians(),
                            0.0,
                        ))
                    }
                } else {
                    Rotation3D::None
                };
                SurfaceSpec::Sphere {
                    semi_diameter,
                    radius_of_curvature: roc,
                    surf_kind,
                    rotation,
                    decenter: parse_decenter(row, i)?,
                    rotation_offset: parse_rotation_offset(row, i)?,
                }
            }
            SurfaceVariant::Iris => {
                let semi_diameter = parse_float(&row.semi_diameter)
                    .with_context(|| format!("surface {i}: semi-diameter"))?;
                SurfaceSpec::Iris {
                    semi_diameter,
                    rotation: Rotation3D::None,
                    decenter: parse_decenter(row, i)?,
                    rotation_offset: parse_rotation_offset(row, i)?,
                }
            }
            SurfaceVariant::Probe => SurfaceSpec::Probe {
                rotation: Rotation3D::None,
                decenter: parse_decenter(row, i)?,
                rotation_offset: parse_rotation_offset(row, i)?,
            },
            SurfaceVariant::Image => SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: parse_decenter(row, i)?,
                rotation_offset: parse_rotation_offset(row, i)?,
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
        let field = match specs.field_mode {
            FieldMode::Angle => {
                let chi = parse_float(&frow.chi).with_context(|| format!("field {i}: chi"))?;
                let phi = parse_float(&frow.phi).with_context(|| format!("field {i}: phi"))?;
                FieldSpec::Angle { chi, phi }
            }
            FieldMode::PointSource => {
                let y = parse_float(&frow.chi).with_context(|| format!("field {i}: y"))?;
                let x = parse_float(&frow.x).with_context(|| format!("field {i}: x"))?;
                FieldSpec::PointSource { x, y }
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

    let background = resolve_background(
        specs,
        #[cfg(feature = "ri-info")]
        materials,
    )?;

    let solves: Vec<Box<dyn Solve>> = specs
        .solves
        .iter()
        .map(|s| -> Box<dyn Solve> {
            match s {
                SolveSpec::MarginalRayHeight {
                    gap_index,
                    target_height,
                    wavelength_id,
                } => Box::new(MarginalRaySolve::new(
                    *gap_index,
                    *target_height,
                    *wavelength_id,
                )),
                SolveSpec::FNumber {
                    surface_index,
                    target_fno,
                    wavelength_id,
                } => Box::new(FNumberSolve::new(
                    *surface_index,
                    *target_fno,
                    *wavelength_id,
                )),
            }
        })
        .collect();

    Ok(ParsedSpecs {
        surfaces,
        gaps,
        fields,
        aperture,
        wavelengths,
        background,
        solves,
    })
}

/// Resolve the background refractive index from SystemSpecs.
fn resolve_background(
    specs: &SystemSpecs,
    #[cfg(feature = "ri-info")] materials: Option<&MaterialsMap>,
) -> Result<Rc<dyn RefractiveIndexSpec>> {
    #[cfg(feature = "ri-info")]
    if specs.use_materials
        && let Some(key) = &specs.background_material_key
    {
        let materials = materials.ok_or_else(|| anyhow::anyhow!("Material store not loaded"))?;
        let mat = materials
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("background material '{key}' not found in database"))?;
        return Ok(Rc::clone(mat) as Rc<dyn RefractiveIndexSpec>);
    }

    let n = parse_float(&specs.background_n).context("background refractive index")?;
    Ok(Rc::new(ConstantRefractiveIndex::new(n, 0.0)))
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
    if use_materials && let Some(key) = &row.material_key {
        let materials = materials.ok_or_else(|| anyhow::anyhow!("Material store not loaded"))?;
        let mat = materials.get(key).ok_or_else(|| {
            anyhow::anyhow!("surface {surface_idx}: material '{key}' not found in database")
        })?;
        return Ok(Rc::clone(mat) as Rc<dyn RefractiveIndexSpec>);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::{SolveSpec, SurfaceRow, SystemSpecs};

    fn convert(specs: &SystemSpecs) -> ParsedSpecs {
        #[cfg(not(feature = "ri-info"))]
        return convert_specs(specs).expect("convert should succeed");
        #[cfg(feature = "ri-info")]
        return convert_specs(specs, &Default::default()).expect("convert should succeed");
    }

    #[test]
    fn empty_solves_converts_ok() {
        let specs = SystemSpecs::default();
        let parsed = convert(&specs);
        assert!(parsed.solves.is_empty());
    }

    #[test]
    fn marginal_ray_height_spec_converts_to_solve() {
        let mut specs = SystemSpecs {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                SurfaceRow::new_sphere("12.5", "25.8", "5.3", "1.515"),
                SurfaceRow::new_sphere("12.5", "Infinity", "46.6", "1.0"),
                SurfaceRow::new_image(),
            ],
            solves: vec![SolveSpec::MarginalRayHeight {
                gap_index: 2,
                target_height: 0.0,
                wavelength_id: 0,
            }],
            ..Default::default()
        };
        specs.wavelengths = vec!["0.567".into()];
        let parsed = convert(&specs);
        assert_eq!(parsed.solves.len(), 1);
        assert_eq!(parsed.solves[0].surface_index(), 2);
    }

    #[test]
    fn fno_spec_converts_to_solve() {
        let mut specs = SystemSpecs {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                SurfaceRow::new_sphere("12.5", "25.8", "5.3", "1.515"),
                SurfaceRow::new_sphere("12.5", "Infinity", "46.6", "1.0"),
                SurfaceRow::new_image(),
            ],
            solves: vec![SolveSpec::FNumber {
                surface_index: 1,
                target_fno: 4.0,
                wavelength_id: 0,
            }],
            ..Default::default()
        };
        specs.wavelengths = vec!["0.567".into()];
        let parsed = convert(&specs);
        assert_eq!(parsed.solves.len(), 1);
        assert_eq!(parsed.solves[0].surface_index(), 1);
    }

    fn lens_specs() -> SystemSpecs {
        SystemSpecs {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                SurfaceRow::new_sphere("12.5", "25.8", "5.3", "1.515"),
                SurfaceRow::new_sphere("12.5", "Infinity", "46.6", "1.0"),
                SurfaceRow::new_image(),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn convert_non_zero_decenter_r_populates_sphere_decenter() {
        use approx::assert_abs_diff_eq;
        let mut specs = lens_specs();
        specs.surfaces[1].decenter_r = "2.5".into();
        let parsed = convert(&specs);
        match &parsed.surfaces[1] {
            crate::SurfaceSpec::Sphere { decenter, .. } => {
                assert_abs_diff_eq!(decenter.x(), 2.5, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
            }
            other => panic!("expected Sphere, got {:?}", other),
        }
    }

    #[test]
    fn convert_non_zero_rotation_offset_theta_populates_sphere() {
        use approx::assert_abs_diff_eq;
        let mut specs = lens_specs();
        specs.surfaces[1].rot_offset_theta = "30.0".into();
        let parsed = convert(&specs);
        match &parsed.surfaces[1] {
            crate::SurfaceSpec::Sphere {
                rotation_offset, ..
            } => match rotation_offset {
                crate::Rotation3D::IntrinsicPassiveRUF(crate::EulerAngles(a, b, c)) => {
                    assert_abs_diff_eq!(*a, 30.0_f64.to_radians(), epsilon = 1e-10);
                    assert_abs_diff_eq!(*b, 0.0, epsilon = 1e-10);
                    assert_abs_diff_eq!(*c, 0.0, epsilon = 1e-10);
                }
                other => panic!("expected IntrinsicPassiveRUF, got {:?}", other),
            },
            other => panic!("expected Sphere, got {:?}", other),
        }
    }

    #[test]
    fn convert_zero_displacement_gives_none_and_zero_vec() {
        use approx::assert_abs_diff_eq;
        let specs = lens_specs();
        let parsed = convert(&specs);
        match &parsed.surfaces[1] {
            crate::SurfaceSpec::Sphere {
                decenter,
                rotation_offset,
                ..
            } => {
                assert!(
                    matches!(rotation_offset, crate::Rotation3D::None),
                    "all-zero rot_offset should give Rotation3D::None"
                );
                assert_abs_diff_eq!(decenter.x(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
            }
            other => panic!("expected Sphere, got {:?}", other),
        }
    }
}
