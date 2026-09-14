use std::rc::Rc;

use anyhow::{Context, Result, anyhow, bail};

use crate::{
    ApertureSpec, BeamSplitterPathKind, BoundaryKind, ConstantRefractiveIndex, EulerAngles,
    FNumberSolve, FieldSpec, GapSpec, LinkedObjectOrientation, MarginalRaySolve, PathSpec,
    PathSurfaceRef, RefractiveIndexSpec, Rotation3D, SequentialModel, Solve, SurfaceSpec, Vec3,
    core::math::linalg::mat3x3::Mat3x3,
    views::components::{Component, components_view},
};

use super::model::{
    BeamSplitterPathKindRow, BoundaryVariant, FieldMode, LinkedObjectOrientationRow, PathRow,
    SolveSpec, StoreIndexTable, SurfaceRefRow, SurfaceRow, SurfaceVariant, SystemSpecs,
};

/// Parsed core specs ready for model construction, one entry per path except
/// where noted.
pub struct ParsedSpecs {
    pub path_specs: Vec<PathSpec>,
    pub field_specs_by_path: Vec<Vec<FieldSpec>>,
    pub aperture_specs_by_path: Vec<ApertureSpec>,
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

impl LinkedObjectOrientationRow {
    fn to_lib(self) -> LinkedObjectOrientation {
        match self {
            LinkedObjectOrientationRow::SameDirection => LinkedObjectOrientation::SameDirection,
            LinkedObjectOrientationRow::Reversed => LinkedObjectOrientation::Reversed,
        }
    }
}

impl BeamSplitterPathKindRow {
    fn to_lib(self) -> BeamSplitterPathKind {
        match self {
            BeamSplitterPathKindRow::Transmitting => BeamSplitterPathKind::Transmitting,
            BeamSplitterPathKindRow::Reflecting => BeamSplitterPathKind::Reflecting,
        }
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
    if specs.paths.is_empty() {
        bail!("system must have at least one path");
    }

    let background = resolve_background(
        specs,
        #[cfg(feature = "ri-info")]
        materials,
    )?;

    let table = specs.store_index_table();

    // --- Pass 1: nominal per-path surface_refs/gaps, zero displacement. ---
    let mut path_specs = Vec::with_capacity(specs.paths.len());
    let mut field_specs_by_path = Vec::with_capacity(specs.paths.len());
    let mut aperture_specs_by_path = Vec::with_capacity(specs.paths.len());

    for (path_idx, path) in specs.paths.iter().enumerate() {
        if path.surface_refs.len() < 2 {
            bail!("path {path_idx}: need at least an Object and Image surface");
        }

        let surface_refs = path
            .surface_refs
            .iter()
            .enumerate()
            .map(|(step, r)| convert_surface_ref(path_idx, step, r, &table))
            .collect::<Result<Vec<_>>>()?;

        let gaps = build_gaps_for_path(
            path_idx,
            path,
            specs.use_materials,
            #[cfg(feature = "ri-info")]
            materials,
        )?;

        let beam_splitter_arms = path.beam_splitter_arms.iter().map(|k| k.to_lib()).collect();

        if path.wavelengths.is_empty() {
            bail!("path {path_idx}: need at least one wavelength");
        }
        let wavelengths = path
            .wavelengths
            .iter()
            .enumerate()
            .map(|(i, w)| {
                parse_float(w).with_context(|| format!("path {path_idx}: wavelength {i}"))
            })
            .collect::<Result<Vec<_>>>()?;

        path_specs.push(PathSpec {
            surface_refs,
            gaps,
            beam_splitter_arms,
            stop_surface: path.stop_surface,
            wavelengths,
        });

        // --- Fields ---
        if path.fields.is_empty() {
            bail!("path {path_idx}: need at least one field point");
        }
        let mut fields = Vec::with_capacity(path.fields.len());
        for (i, frow) in path.fields.iter().enumerate() {
            let field = match path.field_mode {
                FieldMode::Angle => {
                    let chi = parse_float(&frow.chi)
                        .with_context(|| format!("path {path_idx}: field {i}: chi"))?;
                    let phi = parse_float(&frow.phi)
                        .with_context(|| format!("path {path_idx}: field {i}: phi"))?;
                    FieldSpec::Angle { chi, phi }
                }
                FieldMode::PointSource => {
                    let y = parse_float(&frow.chi)
                        .with_context(|| format!("path {path_idx}: field {i}: y"))?;
                    let x = parse_float(&frow.x)
                        .with_context(|| format!("path {path_idx}: field {i}: x"))?;
                    FieldSpec::PointSource { x, y }
                }
            };
            fields.push(field);
        }
        field_specs_by_path.push(fields);

        // --- Aperture ---
        let aperture_sd = parse_float(&path.aperture_semi_diameter)
            .with_context(|| format!("path {path_idx}: aperture"))?;
        aperture_specs_by_path.push(ApertureSpec::EntrancePupil {
            semi_diameter: aperture_sd,
        });
    }

    let solves: Vec<Box<dyn Solve>> = specs
        .solves
        .iter()
        .map(|s| -> Box<dyn Solve> {
            match s {
                SolveSpec::MarginalRayHeight {
                    gap_index,
                    target_height,
                    wavelength_id,
                    path_id,
                } => Box::new(
                    MarginalRaySolve::new(*gap_index, *target_height, *wavelength_id)
                        .with_path_id(*path_id),
                ),
                SolveSpec::FNumber {
                    surface_index,
                    target_fno,
                    wavelength_id,
                    path_id,
                } => Box::new(
                    FNumberSolve::new(*surface_index, *target_fno, *wavelength_id)
                        .with_path_id(*path_id),
                ),
            }
        })
        .collect();

    // --- Pass 2: apply lens-group transformations. Single-path only — lens
    // grouping has no multipath story (a group's pivot/placement geometry is
    // computed from a single flat nominal model, which only exists for a
    // single-path system); multipath systems skip this pass entirely.
    if specs.paths.len() == 1 && !specs.lens_groups.is_empty() {
        apply_group_transforms(&mut path_specs[0], background.clone(), &specs.lens_groups)?;
    }

    Ok(ParsedSpecs {
        path_specs,
        field_specs_by_path,
        aperture_specs_by_path,
        background,
        solves,
    })
}

fn convert_surface_ref(
    path_idx: usize,
    step: usize,
    r: &SurfaceRefRow,
    table: &StoreIndexTable,
) -> Result<PathSurfaceRef> {
    match r {
        SurfaceRefRow::New(row) => Ok(PathSurfaceRef::New(surface_spec_from_row(
            path_idx, step, row,
        )?)),
        SurfaceRefRow::Shared { target, .. } => {
            let store_index = table.store_index_of(*target).ok_or_else(|| {
                anyhow!("path {path_idx} step {step}: Shared row targets an unknown row")
            })?;
            Ok(PathSurfaceRef::Shared(store_index))
        }
        SurfaceRefRow::ObjectLinkedTo {
            path, orientation, ..
        } => Ok(PathSurfaceRef::ObjectLinkedTo {
            path: *path,
            orientation: orientation.to_lib(),
        }),
    }
}

fn surface_spec_from_row(path_idx: usize, step: usize, row: &SurfaceRow) -> Result<SurfaceSpec> {
    let ctx = |field: &str| format!("path {path_idx}, surface {step}: {field}");
    let spec = match row.variant {
        SurfaceVariant::Object => SurfaceSpec::Object,
        SurfaceVariant::Conic => {
            let semi_diameter =
                parse_float(&row.semi_diameter).with_context(|| ctx("semi-diameter"))?;
            let roc = parse_float(&row.radius_of_curvature)
                .with_context(|| ctx("radius of curvature"))?;
            let conic = parse_float(&row.conic_constant).with_context(|| ctx("conic constant"))?;
            let surf_kind = match row.boundary_variant {
                BoundaryVariant::Refracting => BoundaryKind::Refracting,
                BoundaryVariant::Reflecting => BoundaryKind::Reflecting,
            };
            let rotation = tilt_rotation(row, matches!(surf_kind, BoundaryKind::Reflecting))?;
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
            let semi_diameter =
                parse_float(&row.semi_diameter).with_context(|| ctx("semi-diameter"))?;
            let roc = parse_float(&row.radius_of_curvature)
                .with_context(|| ctx("radius of curvature"))?;
            let surf_kind = match row.boundary_variant {
                BoundaryVariant::Refracting => BoundaryKind::Refracting,
                BoundaryVariant::Reflecting => BoundaryKind::Reflecting,
            };
            let rotation = tilt_rotation(row, matches!(surf_kind, BoundaryKind::Reflecting))?;
            SurfaceSpec::Sphere {
                semi_diameter,
                radius_of_curvature: roc,
                surf_kind,
                rotation,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }
        }
        SurfaceVariant::ThinLens => {
            let semi_diameter =
                parse_float(&row.semi_diameter).with_context(|| ctx("semi-diameter"))?;
            let focal_length =
                parse_float(&row.focal_length).with_context(|| ctx("focal length"))?;
            SurfaceSpec::ThinLens {
                semi_diameter,
                focal_length,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }
        }
        SurfaceVariant::BeamSplitter => {
            let semi_diameter =
                parse_float(&row.semi_diameter).with_context(|| ctx("semi-diameter"))?;
            let rotation = tilt_rotation(row, true)?;
            SurfaceSpec::BeamSplitter {
                semi_diameter,
                rotation,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }
        }
        SurfaceVariant::Iris => {
            let semi_diameter =
                parse_float(&row.semi_diameter).with_context(|| ctx("semi-diameter"))?;
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
    Ok(spec)
}

/// θ/ψ nominal-tilt rotation, shared by reflecting Conic/Sphere surfaces and
/// BeamSplitter surfaces (always tiltable, regardless of a chosen boundary
/// kind).
fn tilt_rotation(row: &SurfaceRow, tiltable: bool) -> Result<Rotation3D> {
    if !tiltable {
        return Ok(Rotation3D::None);
    }
    let theta_deg = parse_float(&row.theta).context("theta")?;
    let psi_deg = parse_float(&row.psi).context("psi")?;
    if theta_deg == 0.0 && psi_deg == 0.0 {
        Ok(Rotation3D::None)
    } else {
        Ok(Rotation3D::IntrinsicPassiveRUF(EulerAngles(
            theta_deg.to_radians(),
            psi_deg.to_radians(),
            0.0,
        )))
    }
}

/// Build the `GapSpec`s for one path, skipping the leading run of `Shared`
/// steps immediately after a `Reversed`-oriented `ObjectLinkedTo` first row
/// (those gaps are auto-inferred by the library — mirrors
/// `count_leading_shared`, `core/sequential_model/mod.rs`).
fn build_gaps_for_path(
    path_idx: usize,
    path: &PathRow,
    use_materials: bool,
    #[cfg(feature = "ri-info")] materials: Option<&MaterialsMap>,
) -> Result<Vec<GapSpec>> {
    let n = path.surface_refs.len();
    let n_leading_shared = path.n_leading_shared();

    let mut gaps = Vec::with_capacity(n.saturating_sub(1).saturating_sub(n_leading_shared));
    for step in 0..n - 1 {
        if step < n_leading_shared {
            continue;
        }
        let ctx = |field: &str| format!("path {path_idx}, gap after step {step}: {field}");
        let (thickness_str, ri) = match &path.surface_refs[step] {
            SurfaceRefRow::New(row) => {
                let ri = resolve_row_refractive_index(
                    &row.refractive_index,
                    row.material_key.as_deref(),
                    use_materials,
                    #[cfg(feature = "ri-info")]
                    materials,
                )
                .with_context(|| ctx("refractive index"))?;
                (row.thickness.clone(), ri)
            }
            SurfaceRefRow::Shared { gap_after, .. }
            | SurfaceRefRow::ObjectLinkedTo { gap_after, .. } => {
                let ri = resolve_row_refractive_index(
                    &gap_after.refractive_index,
                    gap_after.material_key.as_deref(),
                    use_materials,
                    #[cfg(feature = "ri-info")]
                    materials,
                )
                .with_context(|| ctx("refractive index"))?;
                (gap_after.thickness.clone(), ri)
            }
        };
        let thickness = parse_float(&thickness_str).with_context(|| ctx("thickness"))?;
        gaps.push(GapSpec {
            thickness,
            refractive_index: ri,
        });
    }
    Ok(gaps)
}

fn resolve_row_refractive_index(
    refractive_index: &str,
    material_key: Option<&str>,
    use_materials: bool,
    #[cfg(feature = "ri-info")] materials: Option<&MaterialsMap>,
) -> Result<Rc<dyn RefractiveIndexSpec>> {
    #[cfg(feature = "ri-info")]
    if use_materials && let Some(key) = material_key {
        let materials = materials.ok_or_else(|| anyhow::anyhow!("Material store not loaded"))?;
        let mat = materials
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("material '{key}' not found in database"))?;
        return Ok(Rc::clone(mat) as Rc<dyn RefractiveIndexSpec>);
    }

    #[cfg(not(feature = "ri-info"))]
    if use_materials {
        bail!("Material mode requires the ri-info feature");
    }
    #[cfg(not(feature = "ri-info"))]
    let _ = material_key;

    let n = parse_float(refractive_index)?;
    Ok(Rc::new(ConstantRefractiveIndex::new(n, 0.0)))
}

/// Two-pass group transformation: compute per-surface `decenter` and
/// `rotation_offset` from `LensGroupSpec` entries and write them back into
/// path 0's `New`-row surfaces. Single-path only (called only when
/// `specs.paths.len() == 1`, so every row is guaranteed `New`).
fn apply_group_transforms(
    path_spec: &mut PathSpec,
    background: Rc<dyn RefractiveIndexSpec>,
    lens_groups: &[super::model::LensGroupSpec],
) -> Result<()> {
    if !path_spec
        .surface_refs
        .iter()
        .all(|r| matches!(r, PathSurfaceRef::New(_)))
    {
        return Ok(()); // defensive: not all New (shouldn't happen for path 0)
    }
    let mut surfaces: Vec<SurfaceSpec> = std::mem::take(&mut path_spec.surface_refs)
        .into_iter()
        .map(|r| match r {
            PathSurfaceRef::New(s) => s,
            _ => unreachable!("checked above"),
        })
        .collect();
    let gaps = &path_spec.gaps;
    let wavelengths = &path_spec.wavelengths;

    // Build a nominal SequentialModel from the zero-displacement specs to get
    // per-surface placements (pos_i and cursor rotation matrix C_i).
    let nominal = match SequentialModel::from_surface_specs(gaps, &surfaces, wavelengths, None) {
        Ok(m) => m,
        Err(_) => return Ok(()), // nominal model failed; skip transforms silently
    };
    let placements = nominal.placements();

    // Derive the component map so we can map component_first_surfs → surf_idxs.
    let components = components_view(&nominal, background).unwrap_or_default();

    for group in lens_groups {
        // Collect the full surface index list for this group from the component map.
        // `nominal` is always single-path here, so there is exactly one
        // PathComponent per component and no path filtering is needed.
        let mut all_surfs: Vec<usize> = Vec::new();
        for &first_surf in &group.component_first_surfs {
            if let Some(pc) = components
                .iter()
                .find(|pc| component_first_idx(&pc.component) == first_surf)
            {
                match &pc.component {
                    Component::Element { surf_idxs } => all_surfs.extend(surf_idxs),
                    Component::Iris { stop_idx } => all_surfs.push(*stop_idx),
                    Component::Mirror { surf_idx }
                    | Component::ThinLens { surf_idx }
                    | Component::BeamSplitter { surf_idx }
                    | Component::UnpairedSurface { surf_idx } => all_surfs.push(*surf_idx),
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
        let c_s1 = placements[s1].rotation_matrix; // passive global→surface local at s1

        // Convert group rotation (degrees, surface-local frame at s1) to passive
        // matrix.
        let [theta_deg, psi_deg, phi_deg] = group.rotation;
        let r_cursor_passive = Rotation3D::IntrinsicPassiveRUF(EulerAngles(
            theta_deg.to_radians(),
            psi_deg.to_radians(),
            phi_deg.to_radians(),
        ))
        .rotation_matrix();

        // R_group: passive rotation in global frame.
        // R_group = C_{s1}^T · R_local_passive · C_{s1}
        let c_s1_t = c_s1.transpose();
        let r_group = c_s1_t * r_cursor_passive * c_s1;

        // d_global: group decenter converted from surface-local frame of s1 to global
        // frame.
        let [dr, du, df] = group.decenter;
        let d_user = Vec3::new(dr, du, df);
        let d_global = c_s1_t * d_user;

        for &i in &all_surfs {
            if i >= placements.len() {
                continue;
            }
            let v_i = placements[i].position; // nominal vertex, global frame
            let c_i = placements[i].rotation_matrix;

            // Rotate about pivot (active = r_group^T), then translate.
            let rotated = r_group.transpose() * (v_i - p);
            let v_i_prime = p + rotated + d_global;

            // Per-surface decenter in surface-local frame i.
            let decenter_i = c_i * (v_i_prime - v_i);

            // Per-surface rotation_offset in surface-local frame i (passive).
            let rot_off_mat = c_i * r_group * c_i.transpose();
            let rotation_offset_i = mat3x3_to_rotation3d(rot_off_mat);

            set_surface_displacement(&mut surfaces, i, decenter_i, rotation_offset_i);
        }
    }

    path_spec.surface_refs = surfaces.drain(..).map(PathSurfaceRef::New).collect();
    Ok(())
}

/// Return the first (lowest) surface index of a component.
fn component_first_idx(c: &Component) -> usize {
    match c {
        Component::Element { surf_idxs } => *surf_idxs.first().unwrap_or(&usize::MAX),
        Component::Iris { stop_idx } => *stop_idx,
        Component::Mirror { surf_idx }
        | Component::ThinLens { surf_idx }
        | Component::BeamSplitter { surf_idx }
        | Component::UnpairedSurface { surf_idx } => *surf_idx,
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
        | SurfaceSpec::ThinLens {
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
        }
        | SurfaceSpec::BeamSplitter {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::{PathRow, SolveSpec, SurfaceRow, SystemSpecs};

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
        assert_eq!(parsed.path_specs.len(), 1);
    }

    #[test]
    fn marginal_ray_height_spec_converts_to_solve() {
        let mut specs = SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("12.5", "25.8", "5.3", "1.515"),
            SurfaceRow::new_sphere("12.5", "Infinity", "46.6", "1.0"),
            SurfaceRow::new_image(),
        ]));
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 2,
            target_height: 0.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        let parsed = convert(&specs);
        assert_eq!(parsed.solves.len(), 1);
        assert_eq!(parsed.solves[0].surface_index(), 2);
        assert_eq!(parsed.solves[0].path_id(), 0);
    }

    #[test]
    fn fno_spec_converts_to_solve() {
        let mut specs = SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("12.5", "25.8", "5.3", "1.515"),
            SurfaceRow::new_sphere("12.5", "Infinity", "46.6", "1.0"),
            SurfaceRow::new_image(),
        ]));
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 1,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        let parsed = convert(&specs);
        assert_eq!(parsed.solves.len(), 1);
        assert_eq!(parsed.solves[0].surface_index(), 1);
    }

    // Thin singlet: Object[0] → Sphere[1] (F=0) → Sphere[2] (F=thickness) →
    // Image[3]. Using a thin lens (5 mm thick) in air so surface 2 is at F=5.
    fn thin_singlet_specs(thickness: &str) -> SystemSpecs {
        let mut path = PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("12.5", "50.0", thickness, "1.515"),
            SurfaceRow::new_sphere("12.5", "Infinity", "100.0", "1.0"),
            SurfaceRow::new_image(),
        ]);
        path.aperture_semi_diameter = "12.5".into();
        path.wavelengths = vec!["0.5876".into()];
        SystemSpecs::new_single(path)
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
        for surf_ref in &parsed.path_specs[0].surface_refs {
            let PathSurfaceRef::New(surf) = surf_ref else {
                continue;
            };
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
        let refs = &parsed.path_specs[0].surface_refs;
        // Both lens surfaces (indices 1 and 2) should have decenter = (1, 0, 0).
        for i in [1, 2] {
            let PathSurfaceRef::New(surf) = &refs[i] else {
                panic!("expected New at index {i}");
            };
            match surf {
                crate::SurfaceSpec::Sphere { decenter, .. } => {
                    assert_abs_diff_eq!(decenter.x(), 1.0, epsilon = 1e-10);
                    assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                    assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
                }
                other => panic!("expected Sphere at index {i}, got {:?}", other),
            }
        }
        // Object and Image should be unaffected.
        match &refs[0] {
            PathSurfaceRef::New(crate::SurfaceSpec::Object) => {}
            _other => panic!("expected Object, got a different PathSurfaceRef variant"),
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
        let refs = &parsed.path_specs[0].surface_refs;
        // Surface 1 is the pivot: zero decenter.
        match &refs[1] {
            PathSurfaceRef::New(crate::SurfaceSpec::Sphere { decenter, .. }) => {
                assert_abs_diff_eq!(decenter.x(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
            }
            _other => panic!("expected Sphere, got a different PathSurfaceRef variant"),
        }
        // Surface 2 is at F=5 from the pivot: R-displacement = 5·sin(1°),
        // F-displacement = −5·(1−cos(1°)).
        let psi_rad = 1_f64.to_radians();
        let expected_r = 5.0 * psi_rad.sin();
        let expected_f = -5.0 * (1.0 - psi_rad.cos());
        match &refs[2] {
            PathSurfaceRef::New(crate::SurfaceSpec::Sphere { decenter, .. }) => {
                assert_abs_diff_eq!(decenter.x(), expected_r, epsilon = 1e-8);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), expected_f, epsilon = 1e-8);
            }
            _other => panic!("expected Sphere, got a different PathSurfaceRef variant"),
        }
    }

    // VT-XFMR-3: ψ=1° rotation AND R=1 mm decenter simultaneously.
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
        let refs = &parsed.path_specs[0].surface_refs;
        let psi_rad = 1_f64.to_radians();
        // Surface 1 (pivot): decenter = (1, 0, 0).
        match &refs[1] {
            PathSurfaceRef::New(crate::SurfaceSpec::Sphere { decenter, .. }) => {
                assert_abs_diff_eq!(decenter.x(), 1.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
            }
            _other => panic!("expected Sphere, got a different PathSurfaceRef variant"),
        }
        // Surface 2: R = 1 + 5·sin(1°), F = −5·(1−cos(1°)).
        match &refs[2] {
            PathSurfaceRef::New(crate::SurfaceSpec::Sphere { decenter, .. }) => {
                assert_abs_diff_eq!(decenter.x(), 1.0 + 5.0 * psi_rad.sin(), epsilon = 1e-8);
                assert_abs_diff_eq!(decenter.y(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), -5.0 * (1.0 - psi_rad.cos()), epsilon = 1e-8);
            }
            _other => panic!("expected Sphere, got a different PathSurfaceRef variant"),
        }
    }

    // VT-XFMR-5: two elements in one merged group; rotation about first surface
    // vertex.
    #[test]
    fn group_rotation_multi_element_rotates_about_first_surface() {
        use crate::gui::model::LensGroupSpec;
        use approx::assert_abs_diff_eq;

        let mut path = PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("12.5", "50.0", "5.0", "1.515"), // glass 5mm
            SurfaceRow::new_sphere("12.5", "Infinity", "5.0", "1.0"), // air 5mm
            SurfaceRow::new_sphere("12.5", "50.0", "5.0", "1.515"), // glass 5mm
            SurfaceRow::new_sphere("12.5", "Infinity", "100.0", "1.0"), // air 100mm
            SurfaceRow::new_image(),
        ]);
        path.aperture_semi_diameter = "12.5".into();
        path.wavelengths = vec!["0.5876".into()];
        let mut g = LensGroupSpec::new("Both");
        g.component_first_surfs = vec![1, 3]; // element A (1–2) and element B (3–4)
        g.decenter = [0.0, 0.0, 0.0];
        g.rotation = [0.0, 2.0, 0.0]; // ψ = 2°
        let mut specs = SystemSpecs::new_single(path);
        specs.lens_groups = vec![g];

        let parsed = convert(&specs);
        let refs = &parsed.path_specs[0].surface_refs;
        let psi_rad = 2_f64.to_radians();

        // Surface 1 is pivot: zero decenter.
        match &refs[1] {
            PathSurfaceRef::New(crate::SurfaceSpec::Sphere { decenter, .. }) => {
                assert_abs_diff_eq!(decenter.x(), 0.0, epsilon = 1e-10);
                assert_abs_diff_eq!(decenter.z(), 0.0, epsilon = 1e-10);
            }
            _other => panic!("expected Sphere, got a different PathSurfaceRef variant"),
        }
        // Surface 3 is at F=10 from pivot: R-displacement ≈ 10·sin(2°).
        let expected_r3 = 10.0 * psi_rad.sin();
        match &refs[3] {
            PathSurfaceRef::New(crate::SurfaceSpec::Sphere { decenter, .. }) => {
                assert_abs_diff_eq!(decenter.x(), expected_r3, epsilon = 1e-6);
            }
            _other => panic!("expected Sphere, got a different PathSurfaceRef variant"),
        }
    }

    /// NFR-2 / VT-MODEL-2: `wf_epi_microscope`'s exact topology —
    /// out-of-order `Shared` references, `ObjectLinkedTo` with `Reversed`
    /// orientation, and per-path beam-splitter arms — round-trips through
    /// `convert_paths` into a model equivalent to the hand-built example.
    #[test]
    fn wf_epi_microscope_topology_round_trips() {
        use crate::gui::model::{
            BeamSplitterPathKindRow, FieldRow, LinkedObjectOrientationRow, RowId, SurfaceRefRow,
        };
        use crate::{ApertureSpec, ParaxialView, SequentialModelBuilder};

        let mut specs = SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_thin_lens("25.0", "40.0", "100.0", "1.0"),
            SurfaceRow::new_beam_splitter("36.0", "50.0", "1.0"),
            SurfaceRow::new_thin_lens("4.25", "3.3333", "5.0", "1.5"),
            SurfaceRow::new_image(),
        ]));
        specs.paths[0].wavelengths = vec!["0.488".into()];
        specs.paths[0].stop_surface = Some(3);
        specs.paths[0].beam_splitter_arms = vec![BeamSplitterPathKindRow::Reflecting];
        // BS at 45 degrees.
        if let SurfaceRefRow::New(row) = &mut specs.paths[0].surface_refs[2] {
            row.theta = "45".into();
        }

        let linked_id = specs.mint_row_id();
        let mirror_id = specs.mint_row_id();
        let tube_id = specs.mint_row_id();
        let img_id = specs.mint_row_id();
        specs.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                SurfaceRefRow::ObjectLinkedTo {
                    id: linked_id,
                    path: 0,
                    orientation: LinkedObjectOrientationRow::Reversed,
                    gap_after: crate::gui::model::GapRow::default(),
                },
                SurfaceRefRow::Shared {
                    target: RowId(3),
                    gap_after: crate::gui::model::GapRow::default(),
                }, // Objective
                SurfaceRefRow::Shared {
                    target: RowId(2),
                    gap_after: crate::gui::model::GapRow::new("50.0", "1.0"),
                }, // BeamSplitter
                SurfaceRefRow::New({
                    let mut m = SurfaceRow::new_conic("10.0", "Infinity", "0", "50.0", "1.0");
                    m.boundary_variant = crate::gui::model::BoundaryVariant::Reflecting;
                    m.theta = "-45".into();
                    m.id = mirror_id;
                    m
                }),
                SurfaceRefRow::New(
                    SurfaceRow::new_thin_lens("20.0", "200.0", "200.0", "1.0").with_id(tube_id),
                ),
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(img_id)),
            ],
            stop_surface: Some(3),
            fields: vec![FieldRow {
                chi: "0.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            }],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "1.5".into(),
            wavelengths: vec!["0.520".into()],
            beam_splitter_arms: vec![BeamSplitterPathKindRow::Transmitting],
        });

        let parsed = convert(&specs);
        assert_eq!(parsed.path_specs.len(), 2);
        // Path 1's Shared refs resolve out of order (3 then 2), matching
        // wf_epi_microscope's own topology exactly.
        assert!(matches!(
            parsed.path_specs[1].surface_refs[1],
            PathSurfaceRef::Shared(3)
        ));
        assert!(matches!(
            parsed.path_specs[1].surface_refs[2],
            PathSurfaceRef::Shared(2)
        ));

        let seq = SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .solves(parsed.solves)
            .build()
            .expect("model should build")
            .model;
        assert_eq!(seq.path_count(), 2);
        assert!(seq.path_surface_indices(0).contains(&3));
        assert!(seq.path_surface_indices(1).contains(&3));
        assert!(seq.path_surface_indices(1).contains(&2));

        let pv = ParaxialView::new(&seq, &parsed.field_specs_by_path, false)
            .expect("paraxial view should build");
        assert!(
            pv.iter().any(|sv| sv.path_id() == 0),
            "path 0 should have a subview"
        );
        assert!(
            pv.iter().any(|sv| sv.path_id() == 1),
            "path 1 should have a subview"
        );
        let _ = ApertureSpec::EntrancePupil { semi_diameter: 1.5 };
    }
}
