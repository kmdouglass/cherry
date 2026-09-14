use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Which table parameter a solve controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolveParameter {
    Thickness,
    RadiusOfCurvature,
}

impl fmt::Display for SolveParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Thickness => write!(f, "Thickness"),
            Self::RadiusOfCurvature => write!(f, "Curvature"),
        }
    }
}

/// Serializable description of one active solve on the system.
///
/// `SolveSpec`s are stored flat on `SystemSpecs`, not nested inside
/// `PathRow` (FR-MODEL-5) — a `Curvature`-kind solve's target store index may
/// be owned by a different path than the one whose ray trace evaluates it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SolveSpec {
    /// `gap_index` is relative to `path_id`'s own gap list — gaps are never
    /// shared between paths, so no cross-path resolution is needed here
    /// (FR-SOLVE-2).
    MarginalRayHeight {
        gap_index: usize,
        target_height: f64,
        wavelength_id: usize,
        #[serde(default)]
        path_id: usize,
    },
    /// `surface_index` is a global **store index** (FR-SOLVE-3), which may be
    /// owned by a path other than `path_id`.
    FNumber {
        surface_index: usize,
        target_fno: f64,
        wavelength_id: usize,
        #[serde(default)]
        path_id: usize,
    },
}

impl SolveSpec {
    pub(crate) fn surface_index(&self) -> usize {
        match self {
            Self::MarginalRayHeight { gap_index, .. } => *gap_index,
            Self::FNumber { surface_index, .. } => *surface_index,
        }
    }

    pub(crate) fn set_surface_index(&mut self, idx: usize) {
        match self {
            Self::MarginalRayHeight { gap_index, .. } => *gap_index = idx,
            Self::FNumber { surface_index, .. } => *surface_index = idx,
        }
    }

    pub(crate) fn path_id(&self) -> usize {
        match self {
            Self::MarginalRayHeight { path_id, .. } => *path_id,
            Self::FNumber { path_id, .. } => *path_id,
        }
    }

    pub(crate) fn set_path_id(&mut self, id: usize) {
        match self {
            Self::MarginalRayHeight { path_id, .. } => *path_id = id,
            Self::FNumber { path_id, .. } => *path_id = id,
        }
    }

    pub(crate) fn parameter(&self) -> SolveParameter {
        match self {
            Self::MarginalRayHeight { .. } => SolveParameter::Thickness,
            Self::FNumber { .. } => SolveParameter::RadiusOfCurvature,
        }
    }

    pub(crate) fn is_paraxial(&self) -> bool {
        match self {
            Self::MarginalRayHeight { .. } | Self::FNumber { .. } => true,
        }
    }
}

/// Transient UI state for the solve-configuration popup. Lives in `model.rs`
/// so both the window and panel can reference it without a circular dependency.
///
/// `surface_index` means a **step index** relative to `path_id`'s own
/// surfaces (`Thickness` parameter) or a global **store index**
/// (`RadiusOfCurvature` parameter) — mirroring `SolveSpec`'s own two
/// index spaces (FR-SOLVE-2/3).
#[derive(Clone, Debug)]
pub struct SolvePopupState {
    pub surface_index: usize,
    pub path_id: usize,
    pub parameter: SolveParameter,
    pub type_str: String,
    pub target: String,
    pub wavelength_id: usize,
    pub error: Option<String>,
}

impl SolvePopupState {
    pub fn open(
        surface_index: usize,
        path_id: usize,
        parameter: SolveParameter,
        existing: Option<&SolveSpec>,
    ) -> Self {
        match existing {
            None => Self {
                surface_index,
                path_id,
                parameter,
                type_str: "None".to_owned(),
                target: String::new(),
                wavelength_id: 0,
                error: None,
            },
            Some(SolveSpec::MarginalRayHeight {
                target_height,
                wavelength_id,
                path_id: existing_path_id,
                ..
            }) => Self {
                surface_index,
                path_id: *existing_path_id,
                parameter,
                type_str: "Marginal ray height".to_owned(),
                target: target_height.to_string(),
                wavelength_id: *wavelength_id,
                error: None,
            },
            Some(SolveSpec::FNumber {
                target_fno,
                wavelength_id,
                path_id: existing_path_id,
                ..
            }) => Self {
                surface_index,
                path_id: *existing_path_id,
                parameter,
                type_str: "F/#".to_owned(),
                target: target_fno.to_string(),
                wavelength_id: *wavelength_id,
                error: None,
            },
        }
    }

    /// Returns true if the currently selected solve type is a paraxial solve.
    pub fn is_paraxial(&self) -> bool {
        match self.type_str.as_str() {
            "Marginal ray height" => SolveSpec::MarginalRayHeight {
                gap_index: 0,
                target_height: 0.0,
                wavelength_id: 0,
                path_id: 0,
            }
            .is_paraxial(),
            "F/#" => SolveSpec::FNumber {
                surface_index: 0,
                target_fno: 1.0,
                wavelength_id: 0,
                path_id: 0,
            }
            .is_paraxial(),
            _ => false,
        }
    }
}

/// Which surface variant this row represents.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SurfaceVariant {
    Object,
    Sphere,
    Conic,
    ThinLens,
    BeamSplitter,
    Iris,
    Probe,
    Image,
}

impl SurfaceVariant {
    /// Variants available for user selection (excludes Object and Image which
    /// are fixed).
    pub const SELECTABLE: &[SurfaceVariant] = &[
        SurfaceVariant::Sphere,
        SurfaceVariant::Conic,
        SurfaceVariant::ThinLens,
        SurfaceVariant::BeamSplitter,
        SurfaceVariant::Iris,
        SurfaceVariant::Probe,
    ];
}

impl std::fmt::Display for SurfaceVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfaceVariant::Object => write!(f, "Object"),
            SurfaceVariant::Sphere => write!(f, "Sphere"),
            SurfaceVariant::Conic => write!(f, "Conic"),
            SurfaceVariant::ThinLens => write!(f, "Thin Lens"),
            SurfaceVariant::BeamSplitter => write!(f, "Beam Splitter"),
            SurfaceVariant::Iris => write!(f, "Iris"),
            SurfaceVariant::Probe => write!(f, "Probe"),
            SurfaceVariant::Image => write!(f, "Image"),
        }
    }
}

/// Whether a conic surface is refracting or reflecting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BoundaryVariant {
    Refracting,
    Reflecting,
}

impl std::fmt::Display for BoundaryVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundaryVariant::Refracting => write!(f, "Refracting"),
            BoundaryVariant::Reflecting => write!(f, "Reflecting"),
        }
    }
}

fn default_zero() -> String {
    "0".to_string()
}

/// Opaque, stable identifier for one surface-introducing row (`New` or
/// `ObjectLinkedTo`), assigned once at creation and never reused or
/// renumbered. `SurfaceRefRow::Shared` targets a `RowId` rather than a
/// position, so it survives inserts/deletes anywhere in the document
/// unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct RowId(pub u64);

/// A single row in the surfaces table. All numeric fields are strings for
/// editing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceRow {
    #[serde(default)]
    pub id: RowId,
    pub variant: SurfaceVariant,
    pub boundary_variant: BoundaryVariant,
    pub refractive_index: String,
    pub thickness: String,
    pub semi_diameter: String,
    pub radius_of_curvature: String,
    pub conic_constant: String,
    /// Focal length. Only meaningful for `SurfaceVariant::ThinLens`.
    #[serde(default)]
    pub focal_length: String,
    /// Tilt in UF plane (about cursor-R axis), degrees. Only meaningful for
    /// reflecting Conic/Sphere surfaces and BeamSplitter surfaces.
    #[serde(default = "default_zero")]
    pub theta: String,
    /// Tilt in RF plane (about cursor-U axis), degrees. Only meaningful for
    /// reflecting Conic/Sphere surfaces and BeamSplitter surfaces.
    #[serde(default = "default_zero")]
    pub psi: String,
    /// Material key from rii.db (e.g. "glass:BK7:SCHOTT"). Used when
    /// `SystemSpecs::use_materials` is true.
    #[serde(default)]
    pub material_key: Option<String>,
}

impl SurfaceRow {
    /// Assign a stable row identity. Used whenever a row is actually
    /// inserted into a `PathRow` (fresh ids come from
    /// `SystemSpecs::mint_row_id`); rows constructed directly via the
    /// `new_*` helpers default to `RowId(0)` until this is called.
    pub fn with_id(mut self, id: RowId) -> Self {
        self.id = id;
        self
    }

    pub fn new_object(thickness: &str) -> Self {
        Self {
            id: RowId::default(),
            variant: SurfaceVariant::Object,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: thickness.into(),
            semi_diameter: String::new(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    pub fn new_conic(
        semi_diameter: &str,
        radius_of_curvature: &str,
        conic_constant: &str,
        thickness: &str,
        refractive_index: &str,
    ) -> Self {
        Self {
            id: RowId::default(),
            variant: SurfaceVariant::Conic,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: radius_of_curvature.into(),
            conic_constant: conic_constant.into(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    pub fn new_sphere(
        semi_diameter: &str,
        radius_of_curvature: &str,
        thickness: &str,
        refractive_index: &str,
    ) -> Self {
        Self {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: radius_of_curvature.into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    pub fn new_thin_lens(
        semi_diameter: &str,
        focal_length: &str,
        thickness: &str,
        refractive_index: &str,
    ) -> Self {
        Self {
            id: RowId::default(),
            variant: SurfaceVariant::ThinLens,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
            focal_length: focal_length.into(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    pub fn new_beam_splitter(semi_diameter: &str, thickness: &str, refractive_index: &str) -> Self {
        Self {
            id: RowId::default(),
            variant: SurfaceVariant::BeamSplitter,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "45".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    pub fn new_iris(semi_diameter: &str, thickness: &str, refractive_index: &str) -> Self {
        Self {
            id: RowId::default(),
            variant: SurfaceVariant::Iris,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    pub fn new_image() -> Self {
        Self {
            id: RowId::default(),
            variant: SurfaceVariant::Image,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: String::new(),
            thickness: String::new(),
            semi_diameter: String::new(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    /// Create a default new surface for insertion.
    pub fn new_default() -> Self {
        Self::new_sphere("10.0", "Infinity", "1.0", "1.0")
    }
}

fn default_phi() -> String {
    "90.0".into()
}

/// A user-defined group of components that share a common displacement and
/// rotation.
///
/// `component_first_surfs` stores the first surface index of each component
/// (as reported by `components_view`) that belongs to the group — always a
/// **store index**, unchanged in meaning by the multipath restructuring
/// (FR-OUT-3). These indices are kept in sync with the surfaces table via
/// `SystemSpecs::insert_surface_after`/`delete_surface`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensGroupSpec {
    pub name: String,
    /// First surface indices of the components in this group.
    pub component_first_surfs: Vec<usize>,
    /// Global-frame decenter [R, U, F] in mm.
    pub decenter: [f64; 3],
    /// Passive-RUF Euler angles [θ, ψ, φ] in degrees applied about the group
    /// vertex.
    pub rotation: [f64; 3],
}

impl LensGroupSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            component_first_surfs: Vec::new(),
            decenter: [0.0; 3],
            rotation: [0.0; 3],
        }
    }
}

/// A single row in the fields table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRow {
    /// χ (chi): zenith angle in degrees (Angle mode) or Y position (PointSource
    /// mode).
    pub chi: String,
    /// φ (phi): azimuthal angle in degrees (Angle mode only). Defaults to 90.0.
    #[serde(default = "default_phi")]
    pub phi: String,
    /// X position (PointSource mode only).
    pub x: String,
}

impl FieldRow {
    fn default_row() -> Self {
        Self {
            chi: "0.0".into(),
            phi: "90.0".into(),
            x: "0.0".into(),
        }
    }
}

/// Which field specification mode is active.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FieldMode {
    Angle,
    PointSource,
}

impl std::fmt::Display for FieldMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldMode::Angle => write!(f, "Angle"),
            FieldMode::PointSource => write!(f, "Point Source"),
        }
    }
}

/// Which specs tab is active.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpecsTab {
    Surfaces,
    Fields,
    Aperture,
    Wavelengths,
}

/// Orientation of a linked Object relative to the source path's terminal
/// cursor. Mirrors `crate::specs::paths::LinkedObjectOrientation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkedObjectOrientationRow {
    SameDirection,
    Reversed,
}

/// Per-path declaration of a beam splitter's role at one of its steps.
/// Mirrors `crate::specs::surfaces::BeamSplitterPathKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeamSplitterPathKindRow {
    Transmitting,
    Reflecting,
}

impl std::fmt::Display for BeamSplitterPathKindRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BeamSplitterPathKindRow::Transmitting => write!(f, "Transmitting"),
            BeamSplitterPathKindRow::Reflecting => write!(f, "Reflecting"),
        }
    }
}

/// Editable gap data (thickness + medium) for the gap immediately following
/// one `SurfaceRefRow`. `New` rows carry their own thickness/refractive-index
/// fields directly on `SurfaceRow` (unchanged from the single-path shape);
/// `Shared`/`ObjectLinkedTo` rows have no `SurfaceRow` to hold that data
/// (the surface geometry itself belongs to whichever row introduced it), so
/// they carry their own `gap_after` — the physical gap to the next step can
/// differ per path even when the surface bounding it is shared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapRow {
    pub thickness: String,
    pub refractive_index: String,
    #[serde(default)]
    pub material_key: Option<String>,
}

impl GapRow {
    pub fn new(thickness: &str, refractive_index: &str) -> Self {
        Self {
            thickness: thickness.into(),
            refractive_index: refractive_index.into(),
            material_key: None,
        }
    }
}

impl Default for GapRow {
    fn default() -> Self {
        Self::new("10.0", "1.0")
    }
}

/// One row of a path's own `surface_refs`, mirroring
/// `crate::specs::paths::PathSurfaceRef`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceRefRow {
    /// Today's fully editable row.
    New(SurfaceRow),
    /// References an earlier-introduced row by stable identity — no `path`
    /// field, since which path currently owns `target` is derived on demand
    /// via `StoreIndexTable::owner_of` rather than cached.
    Shared { target: RowId, gap_after: GapRow },
    /// Only legal at step index 0 of a path with path index > 0. `path` is a
    /// real, stored path index (mirrors `PathSurfaceRef::ObjectLinkedTo`
    /// directly).
    ObjectLinkedTo {
        id: RowId,
        path: usize,
        orientation: LinkedObjectOrientationRow,
        gap_after: GapRow,
    },
}

impl SurfaceRefRow {
    /// The stable identity this row introduces, if any. `New` and
    /// `ObjectLinkedTo` rows introduce a fresh store index and so have an
    /// identity; `Shared` rows are pure references and have none of their
    /// own (they cannot themselves be the target of another `Shared` row).
    pub fn row_id(&self) -> Option<RowId> {
        match self {
            SurfaceRefRow::New(row) => Some(row.id),
            SurfaceRefRow::ObjectLinkedTo { id, .. } => Some(*id),
            SurfaceRefRow::Shared { .. } => None,
        }
    }

    pub fn variant(&self) -> Option<SurfaceVariant> {
        match self {
            SurfaceRefRow::New(row) => Some(row.variant),
            SurfaceRefRow::ObjectLinkedTo { .. } => Some(SurfaceVariant::Object),
            SurfaceRefRow::Shared { .. } => None,
        }
    }

    /// Thickness/medium of the gap following this row, for rows that carry
    /// their own (`Shared`/`ObjectLinkedTo`); `None` for `New` rows, whose
    /// gap data lives on the inner `SurfaceRow` instead.
    pub fn gap_after(&self) -> Option<&GapRow> {
        match self {
            SurfaceRefRow::New(_) => None,
            SurfaceRefRow::Shared { gap_after, .. } => Some(gap_after),
            SurfaceRefRow::ObjectLinkedTo { gap_after, .. } => Some(gap_after),
        }
    }

    pub fn gap_after_mut(&mut self) -> Option<&mut GapRow> {
        match self {
            SurfaceRefRow::New(_) => None,
            SurfaceRefRow::Shared { gap_after, .. } => Some(gap_after),
            SurfaceRefRow::ObjectLinkedTo { gap_after, .. } => Some(gap_after),
        }
    }
}

/// One path's worth of `SystemSpecs` data: surfaces, fields, aperture,
/// wavelengths, stop surface, name. `paths[0]` is always present and is the
/// primary path — a single-path system is exactly `paths.len() == 1`, with
/// every `surface_refs` entry `New`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRow {
    #[serde(default)]
    pub name: Option<String>,
    pub surface_refs: Vec<SurfaceRefRow>,
    /// A store index, or `None` for heuristic aperture-stop selection.
    #[serde(default)]
    pub stop_surface: Option<usize>,
    pub fields: Vec<FieldRow>,
    pub field_mode: FieldMode,
    pub aperture_semi_diameter: String,
    pub wavelengths: Vec<String>,
    /// One entry per beam-splitter step in this path, in step order —
    /// mirrors `PathSpec::beam_splitter_arms`.
    #[serde(default)]
    pub beam_splitter_arms: Vec<BeamSplitterPathKindRow>,
}

impl PathRow {
    /// Wrap already-constructed `SurfaceRow`s as a single path's `New` rows,
    /// assigning each a fresh, path-local `RowId` (0..N). Intended for
    /// single-path test/example construction; a real multipath document
    /// mints ids via `SystemSpecs::mint_row_id` instead so ids stay unique
    /// across the whole document.
    pub fn from_surface_rows(rows: Vec<SurfaceRow>) -> Self {
        let surface_refs = rows
            .into_iter()
            .enumerate()
            .map(|(i, row)| SurfaceRefRow::New(row.with_id(RowId(i as u64))))
            .collect();
        Self {
            name: None,
            surface_refs,
            stop_surface: None,
            fields: vec![FieldRow::default_row()],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "12.5".into(),
            wavelengths: vec!["0.567".into()],
            beam_splitter_arms: Vec::new(),
        }
    }

    /// Number of leading steps (after step 0) whose gap is auto-inferred and
    /// so contributes no entry to this path's own gap-index space — the
    /// consecutive run of `Shared` rows immediately following a
    /// `Reversed`-oriented `ObjectLinkedTo` first row. Mirrors
    /// `count_leading_shared`, `core/sequential_model/mod.rs`.
    pub fn n_leading_shared(&self) -> usize {
        match self.surface_refs.first() {
            Some(SurfaceRefRow::ObjectLinkedTo {
                orientation: LinkedObjectOrientationRow::Reversed,
                ..
            }) => self.surface_refs[1..]
                .iter()
                .take_while(|r| matches!(r, SurfaceRefRow::Shared { .. }))
                .count(),
            _ => 0,
        }
    }

    /// Whether the gap immediately after step `step_idx` is auto-inferred
    /// (part of the leading-shared block) rather than user-editable.
    pub fn gap_is_auto_inferred(&self, step_idx: usize) -> bool {
        step_idx < self.n_leading_shared()
    }

    /// Ordinal position (0-based) of `step_idx` among this path's own
    /// beam-splitter steps, or `None` if that step isn't a beam splitter.
    pub fn beam_splitter_ordinal(&self, table: &StoreIndexTable, step_idx: usize) -> Option<usize> {
        let mut ordinal = 0;
        for (i, r) in self.surface_refs.iter().enumerate() {
            let is_bs = match r {
                SurfaceRefRow::New(row) => row.variant == SurfaceVariant::BeamSplitter,
                SurfaceRefRow::Shared { target, .. } => {
                    table.owning_variant(*target) == Some(SurfaceVariant::BeamSplitter)
                }
                SurfaceRefRow::ObjectLinkedTo { .. } => false,
            };
            if i == step_idx {
                return if is_bs { Some(ordinal) } else { None };
            }
            if is_bs {
                ordinal += 1;
            }
        }
        None
    }

    /// Mutable access to the beam-splitter arm entry for the beam-splitter
    /// row at `step_idx`, growing `beam_splitter_arms` with a default
    /// (`Transmitting`) if it hasn't been sized yet — self-healing, mirroring
    /// how `focal_length` self-heals elsewhere in this module. Returns `None`
    /// if `step_idx` isn't a beam-splitter step.
    pub fn beam_splitter_arm_mut(
        &mut self,
        table: &StoreIndexTable,
        step_idx: usize,
    ) -> Option<&mut BeamSplitterPathKindRow> {
        let ordinal = self.beam_splitter_ordinal(table, step_idx)?;
        if ordinal >= self.beam_splitter_arms.len() {
            self.beam_splitter_arms
                .resize(ordinal + 1, BeamSplitterPathKindRow::Transmitting);
        }
        self.beam_splitter_arms.get_mut(ordinal)
    }
}

/// Resolution of every row's store index and, for `Shared` rows, which path
/// currently owns their target — recomputed fresh from `paths` on every
/// call. Cheap: linear in the number of rows.
pub struct StoreIndexTable {
    id_to_store_index: HashMap<RowId, usize>,
    id_to_owner: HashMap<RowId, usize>,
    id_to_variant: HashMap<RowId, SurfaceVariant>,
}

impl StoreIndexTable {
    pub fn build(paths: &[PathRow]) -> Self {
        let mut id_to_store_index = HashMap::new();
        let mut id_to_owner = HashMap::new();
        let mut id_to_variant = HashMap::new();
        let mut next_store_index = 0usize;
        for (path_idx, path) in paths.iter().enumerate() {
            for surf_ref in &path.surface_refs {
                match surf_ref {
                    SurfaceRefRow::New(row) => {
                        id_to_store_index.insert(row.id, next_store_index);
                        id_to_owner.insert(row.id, path_idx);
                        id_to_variant.insert(row.id, row.variant);
                        next_store_index += 1;
                    }
                    SurfaceRefRow::ObjectLinkedTo { id, .. } => {
                        id_to_store_index.insert(*id, next_store_index);
                        id_to_owner.insert(*id, path_idx);
                        id_to_variant.insert(*id, SurfaceVariant::Object);
                        next_store_index += 1;
                    }
                    SurfaceRefRow::Shared { .. } => {
                        // Consumes no store index; resolved via `target`,
                        // which — by construction (FR-SURF-2/3) — always
                        // names a RowId already inserted above.
                    }
                }
            }
        }
        Self {
            id_to_store_index,
            id_to_owner,
            id_to_variant,
        }
    }

    /// Store index a `New`/`ObjectLinkedTo` row occupies.
    pub fn store_index_of(&self, row_id: RowId) -> Option<usize> {
        self.id_to_store_index.get(&row_id).copied()
    }

    /// Which path currently owns (introduced) `row_id`.
    pub fn owner_of(&self, row_id: RowId) -> Option<usize> {
        self.id_to_owner.get(&row_id).copied()
    }

    /// The `SurfaceVariant` of the row that introduced `row_id`.
    pub fn owning_variant(&self, row_id: RowId) -> Option<SurfaceVariant> {
        self.id_to_variant.get(&row_id).copied()
    }

    /// Resolve any `SurfaceRefRow` to the store index it occupies (`New`/
    /// `ObjectLinkedTo`) or references (`Shared`).
    pub fn resolve(&self, r: &SurfaceRefRow) -> Option<usize> {
        r.row_id()
            .and_then(|id| self.store_index_of(id))
            .or_else(|| match r {
                SurfaceRefRow::Shared { target, .. } => self.store_index_of(*target),
                _ => None,
            })
    }
}

/// All user-editable input specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSpecs {
    /// `paths[0]` is always present and is the primary path — a single-path
    /// system is exactly `paths.len() == 1`.
    pub paths: Vec<PathRow>,
    /// Never-decreasing counter used to mint fresh `RowId`s.
    #[serde(default)]
    pub next_row_id: u64,
    /// When true, surfaces use material keys instead of constant n.
    #[serde(default)]
    pub use_materials: bool,
    /// Material keys the user has selected for use in the surfaces table.
    #[serde(default)]
    pub selected_materials: Vec<String>,
    /// Number of rays in the tangential fan for the cross-section view.
    #[serde(default = "default_cross_section_n_rays")]
    pub cross_section_n_rays: u32,
    /// Grid spacing for full-pupil sampling, in normalised pupil coordinates
    /// [0, 1].
    #[serde(default = "default_full_pupil_spacing")]
    pub full_pupil_spacing: String,
    /// Number of rays in each tangential/sagittal fan bundle. Must be odd;
    /// range 3–501. Controls TA curve resolution in the Ray Fan Plot window.
    #[serde(default = "default_n_fan_rays")]
    pub n_fan_rays: u32,
    /// Refractive index of the background medium (used in constant-n mode).
    #[serde(default = "default_background_n")]
    pub background_n: String,
    /// Material key for the background medium (used in materials mode).
    #[serde(default)]
    pub background_material_key: Option<String>,
    /// Active solves on the system, serialized as part of system state. Flat
    /// (not nested in `PathRow`) — see FR-MODEL-5.
    #[serde(default)]
    pub solves: Vec<SolveSpec>,
    /// User-defined lens groups for the lens overlay panel. Flat, like
    /// `solves` — see FR-MODEL-5.
    #[serde(default)]
    pub lens_groups: Vec<LensGroupSpec>,
}

impl SystemSpecs {
    pub fn mint_row_id(&mut self) -> RowId {
        let id = RowId(self.next_row_id);
        self.next_row_id += 1;
        id
    }

    pub fn store_index_table(&self) -> StoreIndexTable {
        StoreIndexTable::build(&self.paths)
    }

    /// Build a single-path `SystemSpecs` around `path`, with `next_row_id`
    /// seeded safely above every id `path` already uses.
    pub fn new_single(path: PathRow) -> Self {
        let next_row_id = path
            .surface_refs
            .iter()
            .filter_map(|r| r.row_id())
            .map(|id| id.0 + 1)
            .max()
            .unwrap_or(0);
        Self {
            paths: vec![path],
            next_row_id,
            use_materials: false,
            selected_materials: Vec::new(),
            cross_section_n_rays: default_cross_section_n_rays(),
            full_pupil_spacing: default_full_pupil_spacing(),
            n_fan_rays: default_n_fan_rays(),
            background_n: default_background_n(),
            background_material_key: None,
            solves: Vec::new(),
            lens_groups: Vec::new(),
        }
    }

    /// Blank system: only the object and image planes, in refractive-index
    /// mode (no materials).
    pub fn new_blank() -> Self {
        let mut path = PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_image(),
        ]);
        path.aperture_semi_diameter = "10.0".into();
        path.wavelengths = vec!["0.5876".into()];
        Self::new_single(path)
    }

    /// Path label for display: the path's own name, or "Path N" if unset.
    pub fn path_label(&self, path_idx: usize) -> String {
        self.paths
            .get(path_idx)
            .and_then(|p| p.name.clone())
            .unwrap_or_else(|| format!("Path {path_idx}"))
    }

    /// Append a new path with a minimal Object/Image skeleton. Always
    /// appended at the end, never inserted mid-list (FR-NAV-2).
    pub fn add_path(&mut self) {
        let obj_id = self.mint_row_id();
        let img_id = self.mint_row_id();
        self.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                SurfaceRefRow::New(SurfaceRow::new_object("Infinity").with_id(obj_id)),
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(img_id)),
            ],
            stop_surface: None,
            fields: vec![FieldRow::default_row()],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "10.0".into(),
            wavelengths: vec!["0.5876".into()],
            beam_splitter_arms: Vec::new(),
        });
    }

    /// Whether the path at index `k` can be removed: `paths.len() > 1`,
    /// `k > 0` (path 0 can never be removed), and no other path references
    /// it via `Shared` (into one of its store indices) or `ObjectLinkedTo`
    /// (into its path index) — FR-NAV-3.
    pub fn path_is_removable(&self, k: usize) -> bool {
        if k == 0 || k >= self.paths.len() || self.paths.len() <= 1 {
            return false;
        }
        let table = self.store_index_table();
        !self.paths.iter().enumerate().any(|(other_idx, other)| {
            other_idx != k
                && other.surface_refs.iter().any(|r| match r {
                    SurfaceRefRow::Shared { target, .. } => table.owner_of(*target) == Some(k),
                    SurfaceRefRow::ObjectLinkedTo { path, .. } => *path == k,
                    SurfaceRefRow::New(_) => false,
                })
        })
    }

    /// Which later path (if any) blocks removing path `k`, for a tooltip
    /// explanation (FR-NAV-3).
    pub fn path_removal_blocker(&self, k: usize) -> Option<usize> {
        let table = self.store_index_table();
        self.paths
            .iter()
            .enumerate()
            .find_map(|(other_idx, other)| {
                if other_idx == k {
                    return None;
                }
                let blocks = other.surface_refs.iter().any(|r| match r {
                    SurfaceRefRow::Shared { target, .. } => table.owner_of(*target) == Some(k),
                    SurfaceRefRow::ObjectLinkedTo { path, .. } => *path == k,
                    SurfaceRefRow::New(_) => false,
                });
                blocks.then_some(other_idx)
            })
    }

    /// Remove the path at index `k`. Returns `false` (no-op) if
    /// `path_is_removable(k)` is false. `Shared` references never need
    /// renumbering (they target a `RowId`, not a path index); only
    /// `ObjectLinkedTo.path`/`SolveSpec.path_id` fields referencing a path
    /// after `k` are decremented (FR-NAV-4).
    pub fn remove_path(&mut self, k: usize) -> bool {
        if !self.path_is_removable(k) {
            return false;
        }
        self.paths.remove(k);
        for p in &mut self.paths {
            for r in &mut p.surface_refs {
                if let SurfaceRefRow::ObjectLinkedTo { path, .. } = r
                    && *path > k
                {
                    *path -= 1;
                }
            }
        }
        for s in &mut self.solves {
            if s.path_id() > k {
                s.set_path_id(s.path_id() - 1);
            }
        }
        true
    }

    /// Insert a default surface into `path_idx`'s own `surface_refs` after
    /// step `step_idx`, renumbering `stop_surface`, `Curvature`-kind solves,
    /// and lens groups (all store-index-typed, globally) and this path's own
    /// `Thickness`-kind solves (path-relative `gap_index`, locally) — see the
    /// design doc §2.5.
    pub fn insert_surface_after(&mut self, path_idx: usize, step_idx: usize) {
        let Some(path) = self.paths.get(path_idx) else {
            return;
        };
        let Some(anchor_ref) = path.surface_refs.get(step_idx) else {
            return;
        };
        let table = self.store_index_table();
        let anchor = table.resolve(anchor_ref);

        let new_id = self.mint_row_id();
        self.paths[path_idx].surface_refs.insert(
            step_idx + 1,
            SurfaceRefRow::New(SurfaceRow::new_default().with_id(new_id)),
        );

        if let Some(anchor) = anchor {
            let pivot = anchor + 1;
            for p in &mut self.paths {
                if let Some(stop) = p.stop_surface
                    && stop >= pivot
                {
                    p.stop_surface = Some(stop + 1);
                }
            }
            for solve in &mut self.solves {
                if solve.parameter() == SolveParameter::RadiusOfCurvature
                    && solve.surface_index() >= pivot
                {
                    solve.set_surface_index(solve.surface_index() + 1);
                }
            }
            for group in &mut self.lens_groups {
                for s in &mut group.component_first_surfs {
                    if *s >= pivot {
                        *s += 1;
                    }
                }
            }
        }

        for solve in &mut self.solves {
            if solve.parameter() == SolveParameter::Thickness
                && solve.path_id() == path_idx
                && solve.surface_index() > step_idx
            {
                solve.set_surface_index(solve.surface_index() + 1);
            }
        }
    }

    /// Whether `delete_surface(path_idx, step_idx)` would succeed — read-only
    /// version of its two guards, for the Surfaces panel to disable the
    /// delete button (with a tooltip) rather than silently no-op on click.
    pub fn can_delete_surface(&self, path_idx: usize, step_idx: usize) -> bool {
        let Some(path) = self.paths.get(path_idx) else {
            return false;
        };
        if path.surface_refs.len() <= 2 || step_idx >= path.surface_refs.len() {
            return false;
        }

        // Guard: ObjectLinkedTo terminal-row dependency.
        if step_idx == path.surface_refs.len() - 1
            && self.paths.iter().enumerate().any(|(i, p)| {
                i != path_idx
                    && p.surface_refs
                        .iter()
                        .any(|r| matches!(r, SurfaceRefRow::ObjectLinkedTo { path, .. } if *path == path_idx))
            })
        {
            return false;
        }

        // Guard: Shared-target dangling reference.
        if let Some(row_id) = path.surface_refs[step_idx].row_id()
            && self.paths.iter().any(|p| {
                p.surface_refs
                    .iter()
                    .any(|r| matches!(r, SurfaceRefRow::Shared { target, .. } if *target == row_id))
            })
        {
            return false;
        }

        true
    }

    /// Remove the row at step `step_idx` of path `path_idx`. No-ops (returns
    /// `false`) if: fewer than 3 rows remain on that path (must keep object +
    /// image); another path's `Shared` row targets this row's identity; or
    /// this is the path's own terminal row and another path's
    /// `ObjectLinkedTo` depends on it — see the design doc §2.5 for why both
    /// guards are needed.
    pub fn delete_surface(&mut self, path_idx: usize, step_idx: usize) -> bool {
        if !self.can_delete_surface(path_idx, step_idx) {
            return false;
        }

        let table = self.store_index_table();
        let pivot = table.resolve(&self.paths[path_idx].surface_refs[step_idx]);
        let introduces_store_index = self.paths[path_idx].surface_refs[step_idx]
            .row_id()
            .is_some();

        self.paths[path_idx].surface_refs.remove(step_idx);

        // Local (path-relative) renumbering: Thickness-kind solves for this
        // path, regardless of the removed row's kind (removing any step
        // shortens this path's own gap list by one).
        self.solves.retain(|s| {
            !(s.parameter() == SolveParameter::Thickness
                && s.path_id() == path_idx
                && s.surface_index() == step_idx)
        });
        for solve in &mut self.solves {
            if solve.parameter() == SolveParameter::Thickness
                && solve.path_id() == path_idx
                && solve.surface_index() > step_idx
            {
                solve.set_surface_index(solve.surface_index() - 1);
            }
        }

        // Global (store-index) renumbering: only when the removed row
        // consumed a store index (New/ObjectLinkedTo) — a Shared row's
        // removal frees no store index and shifts nothing.
        if introduces_store_index && let Some(pivot) = pivot {
            for p in &mut self.paths {
                p.stop_surface = match p.stop_surface {
                    Some(stop) if stop == pivot => None,
                    Some(stop) if stop > pivot => Some(stop - 1),
                    other => other,
                };
            }
            self.solves.retain(|s| {
                !(s.parameter() == SolveParameter::RadiusOfCurvature && s.surface_index() == pivot)
            });
            for solve in &mut self.solves {
                if solve.parameter() == SolveParameter::RadiusOfCurvature
                    && solve.surface_index() > pivot
                {
                    solve.set_surface_index(solve.surface_index() - 1);
                }
            }
            for group in &mut self.lens_groups {
                group.component_first_surfs.retain(|&s| s != pivot);
                for s in &mut group.component_first_surfs {
                    if *s > pivot {
                        *s -= 1;
                    }
                }
            }
            self.lens_groups
                .retain(|g| !g.component_first_surfs.is_empty());
        }

        true
    }

    /// Return the active `Thickness`-kind solve for a given path-relative
    /// gap index (FR-SOLVE-2).
    pub fn thickness_solve_for(&self, path_id: usize, gap_index: usize) -> Option<&SolveSpec> {
        self.solves.iter().find(|s| {
            s.parameter() == SolveParameter::Thickness
                && s.path_id() == path_id
                && s.surface_index() == gap_index
        })
    }

    /// Return the active `Curvature`-kind solve for a given **store index**,
    /// regardless of which path's row is used to look it up (FR-SOLVE-4/5).
    pub fn curvature_solve_for(&self, store_index: usize) -> Option<&SolveSpec> {
        self.solves.iter().find(|s| {
            s.parameter() == SolveParameter::RadiusOfCurvature && s.surface_index() == store_index
        })
    }
}

fn default_cross_section_n_rays() -> u32 {
    3
}

fn default_n_fan_rays() -> u32 {
    65
}

fn default_full_pupil_spacing() -> String {
    "0.1".to_owned()
}

fn default_background_n() -> String {
    "1.0".to_owned()
}

impl Default for SystemSpecs {
    /// Default system: f = 50 mm convexplano lens.
    fn default() -> Self {
        Self::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("12.5", "25.8", "5.3", "1.515"),
            SurfaceRow::new_sphere("12.5", "Infinity", "46.6", "1.0"),
            SurfaceRow::new_image(),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Layout used by mutation tests:
    //   0: Object, 1: Sphere, 2: Sphere, 3: Iris, 4: Image
    fn five_surface_specs() -> SystemSpecs {
        SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("10.0", "50.0", "5.0", "1.5"),
            SurfaceRow::new_sphere("10.0", "Infinity", "5.0", "1.0"),
            SurfaceRow::new_iris("5.0", "1.0", "1.0"),
            SurfaceRow::new_image(),
        ]))
    }

    // --- insert_surface_after ---

    #[test]
    fn insert_before_stop_increments_stop() {
        let mut specs = five_surface_specs();
        specs.paths[0].stop_surface = Some(3);
        specs.insert_surface_after(0, 1); // inserts at step 2, stop was 3 → becomes 4
        assert_eq!(specs.paths[0].stop_surface, Some(4));
    }

    #[test]
    fn insert_at_stop_index_increments_stop() {
        let mut specs = five_surface_specs();
        specs.paths[0].stop_surface = Some(2);
        specs.insert_surface_after(0, 2); // inserts at step 3, stop was 2 → unchanged
        assert_eq!(specs.paths[0].stop_surface, Some(2));
    }

    #[test]
    fn insert_after_stop_leaves_stop_unchanged() {
        let mut specs = five_surface_specs();
        specs.paths[0].stop_surface = Some(2);
        specs.insert_surface_after(0, 3); // inserts after stop → unchanged
        assert_eq!(specs.paths[0].stop_surface, Some(2));
    }

    #[test]
    fn insert_with_no_stop_stays_none() {
        let mut specs = five_surface_specs();
        specs.paths[0].stop_surface = None;
        specs.insert_surface_after(0, 1);
        assert_eq!(specs.paths[0].stop_surface, None);
    }

    // --- delete_surface ---

    #[test]
    fn delete_at_stop_clears_stop() {
        let mut specs = five_surface_specs();
        specs.paths[0].stop_surface = Some(2);
        specs.delete_surface(0, 2);
        assert_eq!(specs.paths[0].stop_surface, None);
    }

    #[test]
    fn delete_before_stop_decrements_stop() {
        let mut specs = five_surface_specs();
        specs.paths[0].stop_surface = Some(3);
        specs.delete_surface(0, 1); // delete before stop → stop was 3, becomes 2
        assert_eq!(specs.paths[0].stop_surface, Some(2));
    }

    #[test]
    fn delete_after_stop_leaves_stop_unchanged() {
        let mut specs = five_surface_specs();
        specs.paths[0].stop_surface = Some(2);
        specs.delete_surface(0, 3);
        assert_eq!(specs.paths[0].stop_surface, Some(2));
    }

    #[test]
    fn delete_with_no_stop_stays_none() {
        let mut specs = five_surface_specs();
        specs.paths[0].stop_surface = None;
        specs.delete_surface(0, 2);
        assert_eq!(specs.paths[0].stop_surface, None);
    }

    // --- SolveSpec serde ---

    #[test]
    fn solve_spec_serde_roundtrip() {
        let specs = vec![
            SolveSpec::MarginalRayHeight {
                gap_index: 2,
                target_height: 0.0,
                wavelength_id: 0,
                path_id: 0,
            },
            SolveSpec::FNumber {
                surface_index: 1,
                target_fno: 4.0,
                wavelength_id: 1,
                path_id: 0,
            },
        ];
        let json = serde_json::to_string(&specs).unwrap();
        let roundtripped: Vec<SolveSpec> = serde_json::from_str(&json).unwrap();
        assert_eq!(specs, roundtripped);
    }

    // --- insert_surface_after renumbers solves ---

    #[test]
    fn insert_surface_after_renumbers_solves() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 2,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        specs.insert_surface_after(0, 1); // insert at step 2, solve was at store idx 2 → becomes 3
        assert_eq!(specs.solves[0].surface_index(), 3);
    }

    #[test]
    fn insert_surface_after_does_not_renumber_solves_at_or_before_insert() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 1,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        specs.insert_surface_after(0, 1); // solve at idx 1, insert after 1 → unchanged
        assert_eq!(specs.solves[0].surface_index(), 1);
    }

    #[test]
    fn insert_surface_after_renumbers_thickness_solve_by_step() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 2,
            target_height: 0.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        specs.insert_surface_after(0, 1); // thickness solve at step 2, insert after step 1 → becomes 3
        assert_eq!(specs.solves[0].surface_index(), 3);
    }

    // --- delete_surface removes and renumbers solves ---

    #[test]
    fn delete_surface_removes_matching_solve() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 2,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        specs.delete_surface(0, 2);
        assert!(specs.solves.is_empty());
    }

    #[test]
    fn delete_surface_renumbers_higher_solves() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 3,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        specs.delete_surface(0, 2); // delete surface 2 → solve at store idx 3 becomes 2
        assert_eq!(specs.solves[0].surface_index(), 2);
    }

    // --- solve_for ---

    #[test]
    fn curvature_solve_for_returns_matching_solve() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 2,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        let found = specs.curvature_solve_for(2);
        assert!(found.is_some());
        assert_eq!(found.unwrap().surface_index(), 2);
    }

    #[test]
    fn curvature_solve_for_returns_none_for_unmatched() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 2,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        assert!(specs.curvature_solve_for(3).is_none());
    }

    #[test]
    fn thickness_solve_for_is_scoped_to_path_and_gap_index() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 2,
            target_height: 5.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        assert!(specs.thickness_solve_for(0, 2).is_some());
        assert!(specs.thickness_solve_for(0, 3).is_none());
        assert!(specs.thickness_solve_for(1, 2).is_none());
    }

    // --- lens_groups index updates ---

    fn specs_with_group(first_surfs: Vec<usize>) -> SystemSpecs {
        let mut specs = five_surface_specs();
        let mut g = LensGroupSpec::new("G1");
        g.component_first_surfs = first_surfs;
        specs.lens_groups = vec![g];
        specs
    }

    #[test]
    fn insert_before_group_surface_increments_group_index() {
        // Group references surface 2; inserting at step 1 should shift it to 3.
        let mut specs = specs_with_group(vec![2]);
        specs.insert_surface_after(0, 1);
        assert_eq!(specs.lens_groups[0].component_first_surfs, vec![3]);
    }

    #[test]
    fn insert_at_group_surface_leaves_group_index_unchanged() {
        // Inserting *after* surface 2 places new surface at 3; group ref 2 unchanged.
        let mut specs = specs_with_group(vec![2]);
        specs.insert_surface_after(0, 2);
        assert_eq!(specs.lens_groups[0].component_first_surfs, vec![2]);
    }

    #[test]
    fn insert_after_group_surface_leaves_group_index_unchanged() {
        // Inserting after surface 3 doesn't affect a group referencing surface 2.
        let mut specs = specs_with_group(vec![2]);
        specs.insert_surface_after(0, 3);
        assert_eq!(specs.lens_groups[0].component_first_surfs, vec![2]);
    }

    #[test]
    fn delete_removes_matching_group_entry() {
        // Deleting surface 2 removes entry 2; entry 1 (before the deleted index)
        // is unaffected.
        let mut specs = specs_with_group(vec![1, 2]);
        specs.delete_surface(0, 2);
        assert_eq!(specs.lens_groups[0].component_first_surfs, vec![1]);
    }

    #[test]
    fn delete_removes_entire_group_when_last_entry_deleted() {
        // If the last entry in a group is removed, the group itself is dropped.
        let mut specs = specs_with_group(vec![2]);
        specs.delete_surface(0, 2);
        assert!(specs.lens_groups.is_empty());
    }

    #[test]
    fn delete_before_group_surface_decrements_group_index() {
        // Deleting surface 1 (before group ref 3) should shift it to 2.
        let mut specs = specs_with_group(vec![3]);
        specs.delete_surface(0, 1);
        assert_eq!(specs.lens_groups[0].component_first_surfs, vec![2]);
    }

    #[test]
    fn delete_after_group_surface_leaves_group_index_unchanged() {
        // Deleting surface 3 doesn't affect a group referencing surface 1.
        let mut specs = specs_with_group(vec![1]);
        specs.delete_surface(0, 3);
        assert_eq!(specs.lens_groups[0].component_first_surfs, vec![1]);
    }

    // === Multipath tests ===

    /// Two-path fixture: path 0 has 4 New rows (Object, Sphere, Sphere,
    /// Image — store indices 0-3); path 1 has Object, a Shared row pointing
    /// at path 0's second Sphere (store index 2), and Image.
    fn two_path_specs() -> SystemSpecs {
        let mut specs = SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("10.0", "50.0", "5.0", "1.5"),
            SurfaceRow::new_sphere("10.0", "Infinity", "5.0", "1.0"),
            SurfaceRow::new_image(),
        ]));
        // path 0 ids: Object=0, Sphere=1, Sphere=2, Image=3 (store indices 0-3)
        let obj1_id = specs.mint_row_id();
        let img1_id = specs.mint_row_id();
        specs.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                SurfaceRefRow::New(SurfaceRow::new_object("Infinity").with_id(obj1_id)),
                SurfaceRefRow::Shared {
                    target: RowId(2),
                    gap_after: GapRow::default(),
                },
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(img1_id)),
            ],
            stop_surface: None,
            fields: vec![FieldRow::default_row()],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "10.0".into(),
            wavelengths: vec!["0.520".into()],
            beam_splitter_arms: Vec::new(),
        });
        specs
    }

    #[test]
    fn store_index_table_resolves_shared_row_to_owner_store_index() {
        let specs = two_path_specs();
        let table = specs.store_index_table();
        assert_eq!(table.store_index_of(RowId(2)), Some(2));
        assert_eq!(table.owner_of(RowId(2)), Some(0));
        // Path 1's Shared row (step 1) resolves to the same store index.
        assert_eq!(table.resolve(&specs.paths[1].surface_refs[1]), Some(2));
    }

    #[test]
    fn store_index_table_resolves_out_of_order_shared_targets() {
        // Mirrors wf_epi_microscope: path 1 references a higher store index
        // before a lower one. Build a 3-path fixture: path 0 introduces
        // store 0..3, path 1 introduces store 4 and Shared-references store
        // 3 then store 1, in that row order.
        let mut specs = SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("10.0", "50.0", "5.0", "1.5"), // store 1
            SurfaceRow::new_sphere("10.0", "Infinity", "5.0", "1.0"), // store 2
            SurfaceRow::new_image(),                              // store 3
        ]));
        let img1_id = specs.mint_row_id();
        specs.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                SurfaceRefRow::Shared {
                    target: RowId(3),
                    gap_after: GapRow::default(),
                }, // higher store idx first
                SurfaceRefRow::Shared {
                    target: RowId(1),
                    gap_after: GapRow::default(),
                }, // lower store idx second
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(img1_id)),
            ],
            stop_surface: None,
            fields: vec![FieldRow::default_row()],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "10.0".into(),
            wavelengths: vec!["0.520".into()],
            beam_splitter_arms: Vec::new(),
        });
        let table = specs.store_index_table();
        assert_eq!(table.resolve(&specs.paths[1].surface_refs[0]), Some(3));
        assert_eq!(table.resolve(&specs.paths[1].surface_refs[1]), Some(1));
    }

    #[test]
    fn insert_in_earlier_path_renumbers_shared_reference_in_later_path() {
        let mut specs = two_path_specs();
        // Insert into path 0 after step 1 (the first Sphere) — a new store
        // index is created at 2, shifting the second Sphere (target of
        // path 1's Shared row) from store 2 to store 3.
        specs.insert_surface_after(0, 1);
        let table = specs.store_index_table();
        assert_eq!(
            table.resolve(&specs.paths[1].surface_refs[1]),
            Some(3),
            "path 1's Shared row must track the same underlying surface after renumbering"
        );
    }

    #[test]
    fn delete_refuses_when_shared_elsewhere_targets_the_row() {
        let mut specs = two_path_specs();
        // Path 0 step 2 (RowId(2), the second Sphere) is Shared-targeted by
        // path 1. Deleting it must be refused.
        let deleted = specs.delete_surface(0, 2);
        assert!(
            !deleted,
            "delete must be refused: a later path Shared-targets this row"
        );
        assert_eq!(specs.paths[0].surface_refs.len(), 4);
    }

    #[test]
    fn delete_of_non_targeted_row_succeeds_and_shared_reference_survives() {
        let mut specs = two_path_specs();
        // Path 0 step 1 (the first Sphere, RowId(1)) is not targeted by
        // anything; deleting it must succeed and leave path 1's Shared
        // reference resolving to the same physical surface (now at a lower
        // store index).
        let deleted = specs.delete_surface(0, 1);
        assert!(deleted);
        let table = specs.store_index_table();
        assert_eq!(table.resolve(&specs.paths[1].surface_refs[1]), Some(1));
    }

    #[test]
    fn delete_refuses_terminal_row_when_object_linked_to_depends_on_it() {
        let mut specs = five_surface_specs();
        let linked_id = specs.mint_row_id();
        specs.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                SurfaceRefRow::ObjectLinkedTo {
                    id: linked_id,
                    path: 0,
                    orientation: LinkedObjectOrientationRow::Reversed,
                    gap_after: GapRow::default(),
                },
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(RowId(999))),
            ],
            stop_surface: None,
            fields: vec![FieldRow::default_row()],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "10.0".into(),
            wavelengths: vec!["0.520".into()],
            beam_splitter_arms: Vec::new(),
        });
        // Path 0's last row (step 4, the Image) is depended on by path 1's
        // ObjectLinkedTo{path: 0}. Deleting it must be refused.
        let last_step = specs.paths[0].surface_refs.len() - 1;
        let deleted = specs.delete_surface(0, last_step);
        assert!(
            !deleted,
            "delete must be refused: ObjectLinkedTo depends on path 0's terminal row"
        );
    }

    #[test]
    fn delete_of_non_terminal_row_is_unaffected_by_object_linked_to_guard() {
        let mut specs = five_surface_specs();
        let linked_id = specs.mint_row_id();
        specs.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                SurfaceRefRow::ObjectLinkedTo {
                    id: linked_id,
                    path: 0,
                    orientation: LinkedObjectOrientationRow::Reversed,
                    gap_after: GapRow::default(),
                },
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(RowId(999))),
            ],
            stop_surface: None,
            fields: vec![FieldRow::default_row()],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "10.0".into(),
            wavelengths: vec!["0.520".into()],
            beam_splitter_arms: Vec::new(),
        });
        // Deleting a non-terminal row of path 0 (step 1) is unaffected by
        // the ObjectLinkedTo guard.
        let deleted = specs.delete_surface(0, 1);
        assert!(deleted);
    }

    #[test]
    fn add_path_appends_and_is_addressable() {
        let mut specs = five_surface_specs();
        assert_eq!(specs.paths.len(), 1);
        specs.add_path();
        assert_eq!(specs.paths.len(), 2);
        assert_eq!(specs.paths[1].surface_refs.len(), 2);
    }

    #[test]
    fn path_zero_is_never_removable() {
        let specs = two_path_specs();
        assert!(!specs.path_is_removable(0));
    }

    #[test]
    fn single_path_system_path_is_not_removable() {
        let specs = five_surface_specs();
        assert!(!specs.path_is_removable(0));
    }

    #[test]
    fn path_referenced_by_shared_is_not_removable() {
        let specs = two_path_specs();
        // Nothing references path 1, so it IS removable.
        assert!(specs.path_is_removable(1));
    }

    #[test]
    fn removing_referenced_path_blocked_by_later_shared_reference() {
        // 3-path system: path 2 Shared-references path 1's surface.
        let mut specs = two_path_specs(); // path 0, path 1 (references path 0)
        let img2_id = specs.mint_row_id();
        // path 1's Image row currently has some id; find it.
        let path1_image_id = specs.paths[1]
            .surface_refs
            .iter()
            .find_map(|r| match r {
                SurfaceRefRow::New(row) if row.variant == SurfaceVariant::Image => Some(row.id),
                _ => None,
            })
            .unwrap();
        specs.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                SurfaceRefRow::Shared {
                    target: path1_image_id,
                    gap_after: GapRow::default(),
                },
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(img2_id)),
            ],
            stop_surface: None,
            fields: vec![FieldRow::default_row()],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "10.0".into(),
            wavelengths: vec!["0.520".into()],
            beam_splitter_arms: Vec::new(),
        });
        assert!(
            !specs.path_is_removable(1),
            "path 1 is referenced by path 2's Shared row"
        );
        assert_eq!(specs.path_removal_blocker(1), Some(2));
    }

    #[test]
    fn remove_path_renumbers_later_object_linked_to() {
        // 3 paths, no cross-references into path 1; path 2 references path 0.
        let mut specs = five_surface_specs();
        specs.add_path(); // path 1
        let linked_id = specs.mint_row_id();
        let img2_id = specs.mint_row_id();
        specs.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                SurfaceRefRow::ObjectLinkedTo {
                    id: linked_id,
                    path: 0,
                    orientation: LinkedObjectOrientationRow::SameDirection,
                    gap_after: GapRow::default(),
                },
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(img2_id)),
            ],
            stop_surface: None,
            fields: vec![FieldRow::default_row()],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "10.0".into(),
            wavelengths: vec!["0.520".into()],
            beam_splitter_arms: Vec::new(),
        }); // path 2
        assert!(specs.remove_path(1));
        assert_eq!(specs.paths.len(), 2);
        // path 2's ObjectLinkedTo{path: 0} is now path 1's own row; 0 < 1 (the
        // removed index) so it is unchanged.
        match &specs.paths[1].surface_refs[0] {
            SurfaceRefRow::ObjectLinkedTo { path, .. } => assert_eq!(*path, 0),
            other => panic!("expected ObjectLinkedTo, got {other:?}"),
        }
    }

    #[test]
    fn remove_path_decrements_solve_path_ids_after_removed_path() {
        let mut specs = five_surface_specs();
        specs.add_path(); // path 1
        specs.add_path(); // path 2
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 0,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 2,
        }];
        assert!(specs.remove_path(1));
        assert_eq!(specs.solves[0].path_id(), 1);
    }

    /// FR-SER-1 / VT-SER-1: a pre-multipath save file (flat
    /// `surfaces: Vec<SurfaceRow>` shape) must fail to parse cleanly rather
    /// than being silently misinterpreted — no migration shim is provided,
    /// per this repo's alpha-stage convention.
    #[test]
    fn old_flat_surfaces_shape_fails_to_deserialize() {
        let old_shaped_json = r#"{
            "surfaces": [{"variant": "Object", "boundary_variant": "Refracting",
                "refractive_index": "1.0", "thickness": "Infinity",
                "semi_diameter": "", "radius_of_curvature": "", "conic_constant": ""}],
            "fields": [],
            "aperture_semi_diameter": "10.0",
            "wavelengths": ["0.5876"],
            "field_mode": "Angle"
        }"#;
        let result: Result<SystemSpecs, _> = serde_json::from_str(old_shaped_json);
        assert!(
            result.is_err(),
            "old flat-shaped SystemSpecs JSON must fail to parse, not silently succeed"
        );
    }

    /// VT-SER-2 / VT-NAV-6: new per-path save/load round-trips structurally,
    /// including `Shared` references, and never serializes `active_path`
    /// (which lives on `AppState`, not `SystemSpecs` — FR-MODEL-4/FR-SER-2).
    #[test]
    fn new_shape_round_trips_and_omits_active_path() {
        let specs = two_path_specs();
        let json = serde_json::to_string_pretty(&specs).unwrap();
        assert!(
            !json.contains("active_path"),
            "SystemSpecs JSON must never contain active_path"
        );
        let roundtripped: SystemSpecs = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped.paths.len(), specs.paths.len());
        assert!(matches!(
            roundtripped.paths[1].surface_refs[1],
            SurfaceRefRow::Shared {
                target: RowId(2),
                ..
            }
        ));
    }
}
