use std::rc::Rc;

use anyhow::{Context, Result, bail};

use crate::{
    ApertureSpec, BoundaryKind, ConstantRefractiveIndex, EulerAngles, FNumberSolve, FieldSpec,
    GapSpec, MarginalRaySolve, RefractiveIndexSpec, Rotation3D, SequentialModel, Solve,
    SurfaceSpec, Vec3,
    core::math::linalg::mat3x3::Mat3x3,
    views::components::{Component, components_view},
};

use super::model::{FieldMode, SolveSpec, SurfaceKind, SurfaceVariant, SystemSpecs};

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
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
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
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }
            }
            SurfaceVariant::Iris => {
                let semi_diameter = parse_float(&row.semi_diameter)
                    .with_context(|| format!("surface {i}: semi-diameter"))?;
                SurfaceSpec::Iris {
                    semi_diameter,
                    rotation: Rotation3D::None,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }
            }
            SurfaceVariant::Probe => SurfaceSpec::Probe {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceVariant::Image => SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
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

    // --- Pass 2: apply group transformations ---
    if !specs.lens_groups.is_empty() {
        apply_group_transforms(
            &mut surfaces,
            &gaps,
            &wavelengths,
            background.clone(),
            &specs.lens_groups,
        )?;
    }

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

/// Two-pass group transformation: compute per-surface `decenter` and
/// `rotation_offset` from `LensGroupSpec` entries and write them back into the
/// `surfaces` vec.
///
/// Pass 1 (done by the caller): nominal `SurfaceSpec`s with zero decenter and
/// `Rotation3D::None`.  This function forms pass 2.
fn apply_group_transforms(
    surfaces: &mut [SurfaceSpec],
    gaps: &[GapSpec],
    wavelengths: &[f64],
    background: Rc<dyn RefractiveIndexSpec>,
    lens_groups: &[super::model::LensGroupSpec],
) -> Result<()> {
    // Build a nominal SequentialModel from the zero-displacement specs to get
    // per-surface placements (pos_i and cursor rotation matrix C_i).
    let nominal = match SequentialModel::from_surface_specs(gaps, surfaces, wavelengths, None) {
        Ok(m) => m,
        Err(_) => return Ok(()), // nominal model failed; skip transforms silently
    };
    let placements = nominal.placements();

    // Derive the component map so we can map component_first_surfs → surf_idxs.
    let components = components_view(&nominal, background).unwrap_or_default();

    for group in lens_groups {
        // Collect the full surface index list for this group from the component map.
        let mut all_surfs: Vec<usize> = Vec::new();
        for &first_surf in &group.component_first_surfs {
            if let Some(comp) = components
                .iter()
                .find(|c| component_first_idx(c) == first_surf)
            {
                match comp {
                    Component::Element { surf_idxs } => all_surfs.extend(surf_idxs),
                    Component::Iris { stop_idx } => all_surfs.push(*stop_idx),
                    Component::Mirror { surf_idx } => all_surfs.push(*surf_idx),
                    Component::UnpairedSurface { surf_idx } => all_surfs.push(*surf_idx),
                }
            }
        }
        all_surfs.sort_unstable();
        all_surfs.dedup();

        if all_surfs.is_empty() {
            continue;
        }

        // The first surface in the group is the pivot / coordinate-frame origin.
        let s1 = *all_surfs.first().unwrap();
        let p = placements[s1].position; // pivot vertex, global frame
        let c_s1 = placements[s1].cursor_rotation_matrix; // passive global→cursor at s1

        // Convert group rotation (degrees, cursor frame at s1) to passive matrix.
        let [theta_deg, psi_deg, phi_deg] = group.rotation;
        let r_cursor_passive = Rotation3D::IntrinsicPassiveRUF(EulerAngles(
            theta_deg.to_radians(),
            psi_deg.to_radians(),
            phi_deg.to_radians(),
        ))
        .rotation_matrix();

        // R_group: passive rotation in global frame.
        // R_group = C_{s1}^T · R_cursor_passive · C_{s1}
        let c_s1_t = c_s1.transpose();
        let r_group = c_s1_t * r_cursor_passive * c_s1;

        // d_global: group decenter converted from cursor frame of s1 to global frame.
        let [dr, du, df] = group.decenter;
        let d_user = Vec3::new(dr, du, df);
        let d_global = c_s1_t * d_user;

        for &i in &all_surfs {
            if i >= placements.len() {
                continue;
            }
            let v_i = placements[i].position; // nominal vertex, global frame
            let c_i = placements[i].cursor_rotation_matrix;

            // Rotate about pivot (active = r_group^T), then translate.
            let rotated = r_group.transpose() * (v_i - p);
            let v_i_prime = p + rotated + d_global;

            // Per-surface decenter in cursor frame i.
            let decenter_i = c_i * (v_i_prime - v_i);

            // Per-surface rotation_offset in cursor frame i (passive).
            let rot_off_mat = c_i * r_group * c_i.transpose();
            let rotation_offset_i = mat3x3_to_rotation3d(rot_off_mat);

            set_surface_displacement(surfaces, i, decenter_i, rotation_offset_i);
        }
    }
    Ok(())
}

/// Return the first (lowest) surface index of a component.
fn component_first_idx(c: &Component) -> usize {
    match c {
        Component::Element { surf_idxs } => *surf_idxs.first().unwrap_or(&usize::MAX),
        Component::Iris { stop_idx } => *stop_idx,
        Component::Mirror { surf_idx } => *surf_idx,
        Component::UnpairedSurface { surf_idx } => *surf_idx,
    }
}

/// Convert a 3×3 passive rotation matrix to `Rotation3D` via Euler-angle
/// extraction. Uses the IntrinsicPassiveRUF (R→U→F, ZYX) decomposition.
///
/// Formula: ψ = asin(−e[0][2]), θ = atan2(e[1][2], e[2][2]), φ = atan2(e[0][1],
/// e[0][0]).
fn mat3x3_to_rotation3d(m: Mat3x3) -> Rotation3D {
    let e = m.e;
    // Check if the matrix is effectively identity.
    let identity = Mat3x3::identity();
    if m.approx_eq(&identity, 1e-12) {
        return Rotation3D::None;
    }
    let psi = (-e[0][2]).asin();
    let theta = e[1][2].atan2(e[2][2]);
    let phi = e[0][1].atan2(e[0][0]);
    Rotation3D::IntrinsicPassiveRUF(EulerAngles(theta, psi, phi))
}

/// Write computed `decenter` and `rotation_offset` into the `SurfaceSpec` at
/// index `i`.  Object and Image surfaces are skipped.
fn set_surface_displacement(
    surfaces: &mut [SurfaceSpec],
    i: usize,
    decenter: Vec3,
    rotation_offset: Rotation3D,
) {
    match &mut surfaces[i] {
        SurfaceSpec::Conic {
            decenter: d,
            rotation_offset: ro,
            ..
        }
        | SurfaceSpec::Sphere {
            decenter: d,
            rotation_offset: ro,
            ..
        }
        | SurfaceSpec::Iris {
            decenter: d,
            rotation_offset: ro,
            ..
        }
        | SurfaceSpec::Probe {
            decenter: d,
            rotation_offset: ro,
            ..
        }
        | SurfaceSpec::Image {
            decenter: d,
            rotation_offset: ro,
            ..
        } => {
            *d = decenter;
            *ro = rotation_offset;
        }
        SurfaceSpec::Object | SurfaceSpec::Custom { .. } => {}
    }
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

    // Thin singlet: Object[0] → Sphere[1] (F=0) → Sphere[2] (F=thickness) →
    // Image[3]. Using a thin lens (5 mm thick) in air so surface 2 is at F=5.
    fn thin_singlet_specs(thickness: &str) -> SystemSpecs {
        SystemSpecs {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                SurfaceRow::new_sphere("12.5", "50.0", thickness, "1.515"),
                SurfaceRow::new_sphere("12.5", "Infinity", "100.0", "1.0"),
                SurfaceRow::new_image(),
            ],
            aperture_semi_diameter: "12.5".into(),
            wavelengths: vec!["0.5876".into()],
            ..Default::default()
        }
    }

    // VT-XFMR-4: zero group params → all surfaces have zero decenter and
    // Rotation3D::None. Groups are ignored when they are all-zero.
    #[test]
    fn group_with_zero_params_leaves_specs_unchanged() {
        use crate::gui::model::LensGroupSpec;
        let mut specs = thin_singlet_specs("5.0");
        let mut g = LensGroupSpec::new("Lens");
        g.component_first_surfs = vec![1];
        specs.lens_groups = vec![g];

        let parsed = convert(&specs);
        for surf in &parsed.surfaces {
            match surf {
                crate::SurfaceSpec::Sphere {
                    decenter,
                    rotation_offset,
                    ..
                }
                | crate::SurfaceSpec::Conic {
                    decenter,
                    rotation_offset,
                    ..
                } => {
                    approx::assert_abs_diff_eq!(decenter.x(), 0.0, epsilon = 1e-10);
                    approx::assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                    approx::assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
                    assert!(matches!(rotation_offset, crate::Rotation3D::None));
                }
                _ => {}
            }
        }
    }

    // VT-XFMR-1: pure R decenter on a straight singlet — both surfaces shift
    // by the same (1, 0, 0) mm in the cursor frame (= global frame for unfolded).
    #[test]
    fn group_pure_r_decenter_shifts_all_surfaces() {
        use crate::gui::model::LensGroupSpec;
        use approx::assert_abs_diff_eq;
        let mut specs = thin_singlet_specs("5.0");
        let mut g = LensGroupSpec::new("Lens");
        g.component_first_surfs = vec![1]; // element at surfaces [1, 2]
        g.decenter = [1.0, 0.0, 0.0]; // R = 1 mm
        g.rotation = [0.0, 0.0, 0.0];
        specs.lens_groups = vec![g];

        let parsed = convert(&specs);
        // Both lens surfaces (indices 1 and 2) should have decenter = (1, 0, 0).
        for i in [1, 2] {
            match &parsed.surfaces[i] {
                crate::SurfaceSpec::Sphere { decenter, .. } => {
                    assert_abs_diff_eq!(decenter.x(), 1.0, epsilon = 1e-10);
                    assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                    assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
                }
                other => panic!("expected Sphere at index {i}, got {:?}", other),
            }
        }
        // Object and Image should be unaffected.
        match &parsed.surfaces[0] {
            crate::SurfaceSpec::Object => {}
            other => panic!("expected Object, got {:?}", other),
        }
    }

    // VT-XFMR-2: pure ψ=1° rotation (about U-axis, tilts in RF plane) on a
    // straight singlet. Surface 1 is at the pivot (F=0); surface 2 is at F=5 mm.
    // Expected: surface 1 has zero decenter; surface 2's R-displacement is
    // 5·sin(1°) and its F-displacement is −5·(1−cos(1°)).
    #[test]
    fn group_pure_rotation_tilts_surfaces_about_pivot() {
        use crate::gui::model::LensGroupSpec;
        use approx::assert_abs_diff_eq;
        let mut specs = thin_singlet_specs("5.0");
        let mut g = LensGroupSpec::new("Lens");
        g.component_first_surfs = vec![1];
        g.decenter = [0.0, 0.0, 0.0];
        g.rotation = [0.0, 1.0, 0.0]; // ψ = 1° (U-axis rotation)
        specs.lens_groups = vec![g];

        let parsed = convert(&specs);
        // Surface 1 is the pivot: zero decenter.
        match &parsed.surfaces[1] {
            crate::SurfaceSpec::Sphere { decenter, .. } => {
                assert_abs_diff_eq!(decenter.x(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
            }
            other => panic!("expected Sphere, got {:?}", other),
        }
        // Surface 2 is at F=5 from the pivot: R-displacement = 5·sin(1°),
        // F-displacement = −5·(1−cos(1°)).
        let psi_rad = 1_f64.to_radians();
        let expected_r = 5.0 * psi_rad.sin();
        let expected_f = -5.0 * (1.0 - psi_rad.cos());
        match &parsed.surfaces[2] {
            crate::SurfaceSpec::Sphere { decenter, .. } => {
                assert_abs_diff_eq!(decenter.x(), expected_r, epsilon = 1e-8);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), expected_f, epsilon = 1e-8);
            }
            other => panic!("expected Sphere, got {:?}", other),
        }
    }

    // VT-XFMR-3: ψ=1° rotation AND R=1 mm decenter simultaneously.
    // Surface 1 decenter = (1, 0, 0); surface 2 decenter = (1 + 5·sin(1°), 0,
    // −5·(1−cos(1°))).
    #[test]
    fn group_rotation_and_decenter_compose_correctly() {
        use crate::gui::model::LensGroupSpec;
        use approx::assert_abs_diff_eq;
        let mut specs = thin_singlet_specs("5.0");
        let mut g = LensGroupSpec::new("Lens");
        g.component_first_surfs = vec![1];
        g.decenter = [1.0, 0.0, 0.0];
        g.rotation = [0.0, 1.0, 0.0]; // ψ = 1°
        specs.lens_groups = vec![g];

        let parsed = convert(&specs);
        let psi_rad = 1_f64.to_radians();
        // Surface 1 (pivot): decenter = (1, 0, 0).
        match &parsed.surfaces[1] {
            crate::SurfaceSpec::Sphere { decenter, .. } => {
                assert_abs_diff_eq!(decenter.x(), 1.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
            }
            other => panic!("expected Sphere, got {:?}", other),
        }
        // Surface 2: R = 1 + 5·sin(1°), F = −5·(1−cos(1°)).
        match &parsed.surfaces[2] {
            crate::SurfaceSpec::Sphere { decenter, .. } => {
                assert_abs_diff_eq!(decenter.x(), 1.0 + 5.0 * psi_rad.sin(), epsilon = 1e-8);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), -5.0 * (1.0 - psi_rad.cos()), epsilon = 1e-8);
            }
            other => panic!("expected Sphere, got {:?}", other),
        }
    }

    // VT-XFMR-5: two elements in one merged group; rotation about first surface
    // vertex. Element A: surfs 1–2 (F=0 to F=5). Element B: surfs 3–4 (F=10 to
    // F=15). Gap between elements: air gap (background). Apply ψ=2° rotation.
    // Surface 3's |v_3'−v_3| should equal 10·sin(2°) ≈ 0.349 mm.
    #[test]
    fn group_rotation_multi_element_rotates_about_first_surface() {
        use crate::gui::model::LensGroupSpec;
        use approx::assert_abs_diff_eq;

        // Two singlets separated by 5 mm of air.
        // surfs: Object[0], Sphere[1](F=0), Sphere[2](F=5), Sphere[3](F=10),
        // Sphere[4](F=15), Image[5] gaps: inf_air, glass(5mm), air(5mm),
        // glass(5mm), air(100mm)
        let specs = SystemSpecs {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                SurfaceRow::new_sphere("12.5", "50.0", "5.0", "1.515"), // glass 5mm
                SurfaceRow::new_sphere("12.5", "Infinity", "5.0", "1.0"), // air 5mm
                SurfaceRow::new_sphere("12.5", "50.0", "5.0", "1.515"), // glass 5mm
                SurfaceRow::new_sphere("12.5", "Infinity", "100.0", "1.0"), // air 100mm
                SurfaceRow::new_image(),
            ],
            aperture_semi_diameter: "12.5".into(),
            wavelengths: vec!["0.5876".into()],
            lens_groups: vec![{
                let mut g = LensGroupSpec::new("Both");
                g.component_first_surfs = vec![1, 3]; // element A (1–2) and element B (3–4)
                g.decenter = [0.0, 0.0, 0.0];
                g.rotation = [0.0, 2.0, 0.0]; // ψ = 2°
                g
            }],
            ..Default::default()
        };

        let parsed = convert(&specs);
        let psi_rad = 2_f64.to_radians();

        // Surface 1 is pivot: zero decenter.
        match &parsed.surfaces[1] {
            crate::SurfaceSpec::Sphere { decenter, .. } => {
                assert_abs_diff_eq!(decenter.x(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
            }
            other => panic!("expected Sphere, got {:?}", other),
        }
        // Surface 3 is at F=10 from pivot: R-displacement ≈ 10·sin(2°).
        let expected_r3 = 10.0 * psi_rad.sin();
        match &parsed.surfaces[3] {
            crate::SurfaceSpec::Sphere { decenter, .. } => {
                assert_abs_diff_eq!(decenter.x(), expected_r3, epsilon = 1e-6);
            }
            other => panic!("expected Sphere, got {:?}", other),
        }
    }
}
