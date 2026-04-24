//! A compact f-theta scan lens with three N-SF57 glass elements.
//!
//! The lens maps scan angle θ to a linear image height f·θ, making it
//! suitable for laser scanning applications.
//!
//! # Reference
//!
//! Milton Laikin, *Lens Design*, 4th ed., CRC Press, 2007, p. 251.
use std::rc::Rc;

use crate::{
    BoundaryType, FieldSpec, GapSpec, RefractiveIndexSpec, Rotation3D, SequentialModel, SurfaceSpec,
};

pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    n_glass: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[f64],
) -> SequentialModel {
    let gap_0 = GapSpec {
        thickness: f64::INFINITY,
        refractive_index: n_air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 5.0,
        refractive_index: n_air.clone(),
    };
    let gap_2 = GapSpec {
        thickness: 0.3,
        refractive_index: n_glass.clone(),
    };
    let gap_3 = GapSpec {
        thickness: 0.02,
        refractive_index: n_air.clone(),
    };
    let gap_4 = GapSpec {
        thickness: 0.5292,
        refractive_index: n_glass.clone(),
    };
    let gap_5 = GapSpec {
        thickness: 4.2927,
        refractive_index: n_air.clone(),
    };
    let gap_6 = GapSpec {
        thickness: 0.59,
        refractive_index: n_glass.clone(),
    };
    let gap_7 = GapSpec {
        thickness: 17.6,
        refractive_index: n_air,
    };
    let gaps = vec![gap_0, gap_1, gap_2, gap_3, gap_4, gap_5, gap_6, gap_7];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::Iris {
        semi_diameter: 0.5,
        rotation: Rotation3D::None,
    };
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 2.0,
        radius_of_curvature: -2.2136,
        conic_constant: 0.0,
        surf_type: BoundaryType::Refracting,
        rotation: Rotation3D::None,
    };
    let surf_3 = SurfaceSpec::Conic {
        semi_diameter: 2.0,
        radius_of_curvature: -2.6575,
        conic_constant: 0.0,
        surf_type: BoundaryType::Refracting,
        rotation: Rotation3D::None,
    };
    let surf_4 = SurfaceSpec::Conic {
        semi_diameter: 2.0,
        radius_of_curvature: -5.5022,
        conic_constant: 0.0,
        surf_type: BoundaryType::Refracting,
        rotation: Rotation3D::None,
    };
    let surf_5 = SurfaceSpec::Conic {
        semi_diameter: 2.0,
        radius_of_curvature: -3.8129,
        conic_constant: 0.0,
        surf_type: BoundaryType::Refracting,
        rotation: Rotation3D::None,
    };
    let surf_6 = SurfaceSpec::Conic {
        semi_diameter: 3.0,
        radius_of_curvature: 7.9951,
        conic_constant: 0.0,
        surf_type: BoundaryType::Refracting,
        rotation: Rotation3D::None,
    };
    let surf_7 = SurfaceSpec::Conic {
        semi_diameter: 3.0,
        radius_of_curvature: 8.3651,
        conic_constant: 0.0,
        surf_type: BoundaryType::Refracting,
        rotation: Rotation3D::None,
    };
    let surf_8 = SurfaceSpec::Image {
        rotation: Rotation3D::None,
    };
    let surfaces = vec![
        surf_0, surf_1, surf_2, surf_3, surf_4, surf_5, surf_6, surf_7, surf_8,
    ];

    SequentialModel::new(&gaps, &surfaces, wavelengths, None).unwrap()
}

pub fn field_specs() -> Vec<FieldSpec> {
    vec![FieldSpec::Angle {
        chi: 0.0,
        phi: 90.0,
    }]
}
