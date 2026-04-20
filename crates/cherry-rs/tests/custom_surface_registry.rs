use cherry_rs::{
    BoundaryType, GapSpec, Rotation3D, SequentialModel, Surface, SurfaceRegistry, SurfaceSpec,
    Vec3, n,
};
use serde_json::json;

#[derive(Debug)]
struct FlatNoOp {
    semi_diameter: f64,
}

impl Surface for FlatNoOp {
    fn boundary_type(&self) -> BoundaryType {
        BoundaryType::NoOp
    }

    fn sag_norm(&self, _pos: Vec3) -> (f64, Vec3) {
        (0.0, Vec3::new(0.0, 0.0, 1.0))
    }

    fn semi_diameter(&self) -> f64 {
        self.semi_diameter
    }
}

fn air_gap(thickness: f64) -> GapSpec {
    GapSpec {
        thickness,
        refractive_index: n!(1.0),
    }
}

fn flat_no_op_constructor(params: &serde_json::Value) -> anyhow::Result<Box<dyn Surface>> {
    let sd = params["semi_diameter"]
        .as_f64()
        .ok_or_else(|| anyhow::anyhow!("missing semi_diameter"))?;
    Ok(Box::new(FlatNoOp { semi_diameter: sd }))
}

#[test]
fn registry_builds_known_type() {
    let mut registry = SurfaceRegistry::new();
    registry.register("flat_no_op", flat_no_op_constructor);

    let result = registry.build("flat_no_op", &json!({"semi_diameter": 5.0}));
    assert!(result.is_ok());
}

#[test]
fn registry_errors_on_unknown_type() {
    let registry = SurfaceRegistry::new();
    let result = registry.build("flat_no_op", &json!({}));
    assert!(result.is_err());
}

#[test]
fn new_with_registry_accepts_custom_spec() {
    let mut registry = SurfaceRegistry::new();
    registry.register("flat_no_op", flat_no_op_constructor);

    let surface_specs = vec![
        SurfaceSpec::Object,
        SurfaceSpec::Custom {
            type_id: "flat_no_op".to_string(),
            params: json!({"semi_diameter": 5.0}),
            rotation: Rotation3D::None,
        },
        SurfaceSpec::Image {
            rotation: Rotation3D::None,
        },
    ];
    let gaps = vec![air_gap(f64::INFINITY), air_gap(10.0)];
    let wavelengths = vec![0.587];

    let result = SequentialModel::new_with_registry(&gaps, &surface_specs, &wavelengths, &registry);
    assert!(result.is_ok());
}

#[test]
fn new_without_registry_rejects_custom_spec() {
    let surface_specs = vec![
        SurfaceSpec::Object,
        SurfaceSpec::Custom {
            type_id: "flat_no_op".to_string(),
            params: json!({"semi_diameter": 5.0}),
            rotation: Rotation3D::None,
        },
        SurfaceSpec::Image {
            rotation: Rotation3D::None,
        },
    ];
    let gaps = vec![air_gap(f64::INFINITY), air_gap(10.0)];
    let wavelengths = vec![0.587];

    let result = SequentialModel::new(&gaps, &surface_specs, &wavelengths);
    assert!(result.is_err());
}

#[test]
fn params_are_forwarded_to_constructor() {
    let mut registry = SurfaceRegistry::new();
    registry.register("flat_no_op", flat_no_op_constructor);

    let surface_specs = vec![
        SurfaceSpec::Object,
        SurfaceSpec::Custom {
            type_id: "flat_no_op".to_string(),
            params: json!({"semi_diameter": 7.5}),
            rotation: Rotation3D::None,
        },
        SurfaceSpec::Image {
            rotation: Rotation3D::None,
        },
    ];
    let gaps = vec![air_gap(f64::INFINITY), air_gap(10.0)];
    let wavelengths = vec![0.587];

    let model = SequentialModel::new_with_registry(&gaps, &surface_specs, &wavelengths, &registry)
        .expect("model should build");

    // The custom surface is at index 1.
    assert_eq!(model.surfaces()[1].semi_diameter(), 7.5);
}
