use cherry_rs::{BoundaryType, GapSpec, Rotation3D, SequentialModel, Surface, Vec3, n};

#[derive(Debug)]
struct FlatNoOp;

impl Surface for FlatNoOp {
    fn boundary_type(&self) -> BoundaryType {
        BoundaryType::NoOp
    }

    fn sag(&self, _pos: Vec3) -> f64 {
        0.0
    }

    fn norm(&self, _pos: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }

    fn semi_diameter(&self) -> f64 {
        f64::INFINITY
    }
}

fn air_gap(thickness: f64) -> GapSpec {
    GapSpec {
        thickness,
        refractive_index: n!(1.0),
    }
}

#[test]
fn from_surfaces_constructs_minimal_model() {
    let surfaces: Vec<(Box<dyn Surface>, Rotation3D)> = vec![
        (Box::new(FlatNoOp), Rotation3D::None),
        (Box::new(FlatNoOp), Rotation3D::None),
    ];
    let gaps = vec![air_gap(10.0)];
    let wavelengths = vec![0.587];

    assert!(SequentialModel::from_surfaces(surfaces, &gaps, &wavelengths).is_ok());
}

#[test]
fn from_surfaces_wrong_gap_count_errors() {
    let surfaces: Vec<(Box<dyn Surface>, Rotation3D)> = vec![
        (Box::new(FlatNoOp), Rotation3D::None),
        (Box::new(FlatNoOp), Rotation3D::None),
    ];
    let gaps = vec![air_gap(10.0), air_gap(5.0)]; // one too many
    let wavelengths = vec![0.587];

    assert!(SequentialModel::from_surfaces(surfaces, &gaps, &wavelengths).is_err());
}

#[test]
fn from_surfaces_wavelengths_are_preserved() {
    let surfaces: Vec<(Box<dyn Surface>, Rotation3D)> = vec![
        (Box::new(FlatNoOp), Rotation3D::None),
        (Box::new(FlatNoOp), Rotation3D::None),
    ];
    let gaps = vec![air_gap(10.0)];
    let wavelengths = vec![0.486, 0.587, 0.656];

    let model =
        SequentialModel::from_surfaces(surfaces, &gaps, &wavelengths).expect("model should build");

    assert_eq!(model.wavelengths(), wavelengths.as_slice());
}
