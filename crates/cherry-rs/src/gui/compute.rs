use std::sync::mpsc::{Receiver, Sender};

#[cfg(feature = "ri-info")]
use std::{collections::HashMap, rc::Rc};

use crate::{
    ParaxialView, SequentialModel, SequentialModelBuilder, SequentialSubModel, components_view,
    cross_section_view, ray_trace_3d_view, specs::fields::PupilSampling, trace_ray_bundle,
    views::ray_trace_3d::SamplingConfig,
};

use super::{
    convert,
    model::{SolveParameter, SolveSpec, SystemSpecs},
    result_package::{ResultPackage, SolvedValues, SurfaceDesc},
};

pub struct ComputeRequest {
    pub id: u64,
    pub specs: SystemSpecs,
    /// Which path the cross-section ray-fan overlay traces (FR-XS-6). Every
    /// other computed value covers every path in one pass regardless of this
    /// field.
    pub active_path: usize,
}

/// Spawn the compute thread on native or as a Web Worker on WASM.
pub fn spawn_compute_thread<F: FnOnce() + Send + 'static>(f: F) {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(f);
    #[cfg(target_arch = "wasm32")]
    wasm_thread::spawn(f);
}

/// Deserialize raw bytes into a material map.
#[cfg(feature = "ri-info")]
fn deserialize_materials(data: &[u8]) -> HashMap<String, Rc<lib_ria::Material>> {
    let mut store: lib_ria::Store = match bitcode::deserialize(data) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Compute thread: cannot deserialize material database: {e}");
            return HashMap::new();
        }
    };
    let keys: Vec<String> = store.keys().cloned().collect();
    let mut materials = HashMap::with_capacity(keys.len());
    for key in keys {
        if let Some(mat) = store.remove(&key) {
            materials.insert(key, Rc::new(mat));
        }
    }
    materials
}

/// Load the material store from disk. Returns an empty map on failure.
#[cfg(all(feature = "ri-info", not(target_arch = "wasm32")))]
fn load_materials() -> HashMap<String, Rc<lib_ria::Material>> {
    let filename = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/rii.db");
    let data = match std::fs::read(&filename) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Compute thread: cannot read {}: {e}", filename.display());
            return HashMap::new();
        }
    };
    deserialize_materials(&data)
}

/// Background compute loop. Drains the channel and processes only the latest
/// request, then sends the result back.
pub fn compute_loop(
    rx: Receiver<ComputeRequest>,
    tx: Sender<ResultPackage>,
    #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
    materials_rx: std::sync::mpsc::Receiver<Vec<u8>>,
) {
    // Native: load materials from disk at startup.
    #[cfg(all(feature = "ri-info", not(target_arch = "wasm32")))]
    let materials = load_materials();

    // WASM: block until the main thread sends the fetched database bytes, then
    // deserialize. The coordinator runs in a Web Worker so blocking is safe.
    #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
    let materials = match materials_rx.recv() {
        Ok(bytes) => deserialize_materials(&bytes),
        Err(_) => HashMap::new(),
    };

    loop {
        // Block until we receive at least one request.
        let first = match rx.recv() {
            Ok(r) => r,
            Err(_) => return, // channel closed; exit thread
        };

        // Drain and take the latest, discarding stale intermediate requests.
        let mut latest = first;
        while let Ok(r) = rx.try_recv() {
            latest = r;
        }

        let result = run_compute(
            latest,
            #[cfg(feature = "ri-info")]
            &materials,
        );

        if tx.send(result).is_err() {
            return; // receiver dropped; exit thread
        }
    }
}

fn run_compute(
    req: ComputeRequest,
    #[cfg(feature = "ri-info")] materials: &HashMap<String, Rc<lib_ria::Material>>,
) -> ResultPackage {
    #[cfg(feature = "ri-info")]
    let parsed = convert::convert_specs(&req.specs, materials);
    #[cfg(not(feature = "ri-info"))]
    let parsed = convert::convert_specs(&req.specs);

    let parsed = match parsed {
        Ok(p) => p,
        Err(e) => return ResultPackage::error(req.id, format!("Specs error: {e}")),
    };

    let build_result = SequentialModelBuilder::new()
        .paths(parsed.path_specs)
        .solves(parsed.solves)
        .build();
    let build_result = match build_result {
        Ok(r) => r,
        Err(e) => return ResultPackage::error(req.id, format!("Model error: {e}")),
    };
    let seq = build_result.model;

    let active_path = req.active_path.min(seq.path_count().saturating_sub(1));
    let solved_values = extract_solved_values(&req.specs.solves, &seq);

    let wavelengths = seq.wavelengths().to_vec();
    let wavelengths_by_path: Vec<Vec<f64>> = (0..seq.path_count())
        .map(|p| seq.wavelengths_for_path(p).to_vec())
        .collect();
    let surfaces = build_surface_descs(&seq, active_path);
    let fields_by_path: Vec<Vec<super::result_package::FieldDesc>> = parsed
        .field_specs_by_path
        .iter()
        .map(|fs| build_field_descs(fs))
        .collect();

    let pv = match ParaxialView::new(&seq, &parsed.field_specs_by_path, false) {
        Ok(p) => p,
        Err(e) => {
            return ResultPackage {
                id: req.id,
                wavelengths,
                wavelengths_by_path,
                surfaces,
                fields_by_path,
                field_specs_by_path: parsed.field_specs_by_path,
                paraxial: None,
                ray_trace: None,
                cross_section: None,
                error: Some(format!("Paraxial error: {e}")),
                solved_values,
                components: Vec::new(),
            };
        }
    };

    let full_pupil_spacing = req
        .specs
        .full_pupil_spacing
        .trim()
        .parse::<f64>()
        .unwrap_or(0.1);
    let config = SamplingConfig {
        n_fan_rays: req.specs.n_fan_rays as usize,
        full_pupil_spacing,
    };
    let trace = match ray_trace_3d_view(
        &parsed.aperture_specs_by_path,
        &parsed.field_specs_by_path,
        &seq,
        &pv,
        config,
    ) {
        Ok(t) => Some(t),
        Err(e) => {
            log::warn!("Ray trace failed: {e}");
            None
        }
    };

    // Ray-fan overlay for the cross-section window: active-path-only
    // (FR-XS-6), not every path simultaneously.
    let cross_section_rays = trace_ray_bundle(
        active_path,
        &parsed.aperture_specs_by_path[active_path],
        &parsed.field_specs_by_path[active_path],
        &seq,
        &pv,
        PupilSampling::TangentialRayFan {
            n: req.specs.cross_section_n_rays as usize,
        },
    )
    .ok();

    let components = components_view(&seq, parsed.background.clone()).unwrap_or_default();
    let cross_section = Some(cross_section_view(
        &seq,
        cross_section_rays.as_deref(),
        &components,
    ));

    ResultPackage {
        id: req.id,
        wavelengths,
        wavelengths_by_path,
        surfaces,
        fields_by_path,
        field_specs_by_path: parsed.field_specs_by_path,
        paraxial: Some(pv),
        ray_trace: trace,
        cross_section,
        error: None,
        solved_values,
        components,
    }
}

/// Extract post-solve values directly from the built model, keyed the same
/// way the Surfaces-tab UI looks them up: `Thickness`-kind solves by
/// (`path_id`, path-relative `gap_index`); `Curvature`-kind solves by global
/// store index. Reading from the final `seq` (rather than
/// `BuildResult::gap_specs`/`surface_specs`, which are only populated for the
/// single-path build branch) works uniformly for both single- and
/// multipath models.
fn extract_solved_values(solves: &[SolveSpec], seq: &SequentialModel) -> SolvedValues {
    let mut sv = SolvedValues::default();
    for solve in solves {
        match solve.parameter() {
            SolveParameter::Thickness => {
                if let Some(submodel) = seq.submodels_for_path(solve.path_id()).first()
                    && let Some(gap) = submodel.gaps().get(solve.surface_index())
                {
                    sv.gap_thicknesses
                        .insert(solve.surface_index(), gap.thickness);
                }
            }
            SolveParameter::RadiusOfCurvature => {
                if let Some(surface) = seq.surfaces().get(solve.surface_index()) {
                    sv.surface_rocs
                        .insert(solve.surface_index(), surface.roc(0.0));
                }
            }
        }
    }
    sv
}

fn build_surface_descs(seq: &SequentialModel, active_path: usize) -> Vec<SurfaceDesc> {
    use std::collections::HashMap;

    use crate::SurfaceKind;

    let path_steps: HashMap<usize, usize> = seq
        .path_surface_indices(active_path)
        .iter()
        .enumerate()
        .map(|(step, &store_idx)| (store_idx, step))
        .collect();

    seq.surfaces()
        .iter()
        .zip(seq.placements().iter())
        .enumerate()
        .map(|(i, (s, p))| {
            let name = match s.surface_kind() {
                SurfaceKind::BeamSplitter => "Beam Splitter",
                SurfaceKind::Conic => "Conic",
                SurfaceKind::Image => "Image",
                SurfaceKind::Object => "Object",
                SurfaceKind::Probe => "Probe",
                SurfaceKind::Iris => "Iris",
                SurfaceKind::Sphere => "Sphere",
                SurfaceKind::ThinLens => "Thin Lens",
                SurfaceKind::Custom => "Custom",
            };
            SurfaceDesc {
                index: i,
                label: format!("{name} [{i}]"),
                pos: p.position,
                rot_mat: p.rotation_matrix,
                path_step: path_steps.get(&i).copied(),
            }
        })
        .collect()
}

fn build_field_descs(fields: &[crate::FieldSpec]) -> Vec<super::result_package::FieldDesc> {
    use super::result_package::FieldDesc;
    fields
        .iter()
        .map(|f| {
            let label = match f {
                crate::FieldSpec::Angle { chi, phi, .. } => {
                    format!("\u{03c7}={chi:.3}\u{00b0}, \u{03c6}={phi:.3}\u{00b0}")
                }
                crate::FieldSpec::PointSource { x, y, .. } => {
                    format!("({x}, {y}) mm")
                }
            };
            FieldDesc { label }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{convert, model::SystemSpecs};

    #[test]
    fn surface_desc_labels_use_variant_names() {
        let specs = SystemSpecs::default();
        #[cfg(not(feature = "ri-info"))]
        let parsed = convert::convert_specs(&specs).expect("convert");
        #[cfg(feature = "ri-info")]
        let parsed = convert::convert_specs(&specs, &Default::default()).expect("convert");
        let seq = SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .build()
            .expect("model")
            .model;
        let descs = build_surface_descs(&seq, 0);

        assert!(
            descs[0].label.starts_with("Object [0]"),
            "first surface should be Object [0], got {}",
            descs[0].label
        );
        assert!(
            descs.last().unwrap().label.starts_with("Image"),
            "last surface should start with Image, got {}",
            descs.last().unwrap().label
        );
        // All labels should follow the "Variant [index]" format.
        for desc in &descs {
            assert!(
                desc.label.contains('['),
                "label should contain '[', got {}",
                desc.label
            );
        }
    }

    /// Regression: `SurfaceDesc.path_step` must be scoped to whichever path
    /// is passed as `active_path`, not a global/store index — a downstream
    /// consumer (`RayBundle`) is indexed by per-path step position, and a
    /// surface only reachable from the *other* path has no step at all in
    /// this path's own bundle.
    #[test]
    fn build_surface_descs_scopes_path_step_to_active_path() {
        use std::rc::Rc;

        use crate::examples::beam_splitter::two_path_model;
        use crate::specs::gaps::ConstantRefractiveIndex;

        // Store indices: 0=Object, 1=BeamSplitter (shared), 2=Image_T
        // (path 0's own), 3=Image_R (path 1's own).
        let n_air = Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let seq = two_path_model(n_air, &[0.5876], 10.0, 10.0);

        let descs0 = build_surface_descs(&seq, 0);
        assert_eq!(descs0[0].path_step, Some(0), "path 0's own object");
        assert_eq!(descs0[1].path_step, Some(1), "shared beam splitter");
        assert_eq!(descs0[2].path_step, Some(2), "path 0's own image");
        assert_eq!(
            descs0[3].path_step, None,
            "path 1's own image is not reachable from path 0"
        );

        let descs1 = build_surface_descs(&seq, 1);
        assert_eq!(descs1[0].path_step, Some(0), "shared object");
        assert_eq!(descs1[1].path_step, Some(1), "shared beam splitter");
        assert_eq!(
            descs1[2].path_step, None,
            "path 0's own image is not reachable from path 1"
        );
        assert_eq!(descs1[3].path_step, Some(2), "path 1's own image");
    }

    #[test]
    fn field_descs_angle_mode() {
        use crate::FieldSpec;
        let fields = vec![
            FieldSpec::Angle {
                chi: 0.0,
                phi: 90.0,
            },
            FieldSpec::Angle {
                chi: 5.0,
                phi: 90.0,
            },
        ];
        let descs = build_field_descs(&fields);
        assert_eq!(
            descs[0].label,
            "\u{03c7}=0.000\u{00b0}, \u{03c6}=90.000\u{00b0}"
        );
        assert_eq!(
            descs[1].label,
            "\u{03c7}=5.000\u{00b0}, \u{03c6}=90.000\u{00b0}"
        );
    }
}
