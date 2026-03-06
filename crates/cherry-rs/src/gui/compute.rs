use std::sync::mpsc::{Receiver, Sender};

#[cfg(feature = "ri-info")]
use std::{collections::HashMap, rc::Rc};

use crate::{ParaxialView, SequentialModel, ray_trace_3d_view};

use super::{
    convert,
    model::SystemSpecs,
    result_package::{BoundingBox3D, ResultPackage, SurfaceDesc},
};

pub struct ComputeRequest {
    pub id: u64,
    pub specs: SystemSpecs,
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

    let seq = match SequentialModel::new(&parsed.gaps, &parsed.surfaces, &parsed.wavelengths) {
        Ok(s) => s,
        Err(e) => return ResultPackage::error(req.id, format!("Model error: {e}")),
    };

    let wavelengths = seq.wavelengths().to_vec();
    let surfaces = build_surface_descs(&seq);
    let fields = build_field_descs(&parsed.fields);

    let pv = match ParaxialView::new(&seq, &parsed.fields, false) {
        Ok(p) => p,
        Err(e) => {
            return ResultPackage {
                id: req.id,
                wavelengths,
                surfaces,
                fields,
                paraxial: None,
                ray_trace: None,
                bounding_box: BoundingBox3D::default(),
                error: Some(format!("Paraxial error: {e}")),
            };
        }
    };

    let trace = match ray_trace_3d_view(&parsed.aperture, &parsed.fields, &seq, &pv, None) {
        Ok(t) => Some(t),
        Err(e) => {
            log::warn!("Ray trace failed: {e}");
            None
        }
    };

    ResultPackage {
        id: req.id,
        wavelengths,
        surfaces,
        fields,
        paraxial: Some(pv),
        ray_trace: trace,
        bounding_box: BoundingBox3D::default(),
        error: None,
    }
}

fn build_surface_descs(seq: &SequentialModel) -> Vec<SurfaceDesc> {
    use crate::core::sequential_model::Surface;
    seq.surfaces()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let name = match s {
                Surface::Conic(_) => "Conic",
                Surface::Image(_) => "Image",
                Surface::Object(_) => "Object",
                Surface::Probe(_) => "Probe",
                Surface::Stop(_) => "Stop",
            };
            SurfaceDesc {
                index: i,
                label: format!("{name} [{i}]"),
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
                crate::FieldSpec::Angle { angle, .. } => {
                    format!("{angle:.3}\u{00b0}")
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
        let seq = SequentialModel::new(&parsed.gaps, &parsed.surfaces, &parsed.wavelengths)
            .expect("model");
        let descs = build_surface_descs(&seq);

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

    #[test]
    fn field_descs_angle_mode() {
        use crate::{FieldSpec, specs::fields::PupilSampling};
        let fields = vec![
            FieldSpec::Angle {
                angle: 0.0,
                pupil_sampling: PupilSampling::TangentialRayFan,
            },
            FieldSpec::Angle {
                angle: 5.0,
                pupil_sampling: PupilSampling::TangentialRayFan,
            },
        ];
        let descs = build_field_descs(&fields);
        assert_eq!(descs[0].label, "0.000\u{00b0}");
        assert_eq!(descs[1].label, "5.000\u{00b0}");
    }
}
