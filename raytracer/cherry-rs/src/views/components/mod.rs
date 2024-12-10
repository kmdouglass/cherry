use std::collections::HashSet;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{
    core::{sequential_model::Surface, Float, RefractiveIndex},
    RefractiveIndexSpec, SequentialModel, SequentialSubModel,
};

const TOL: Float = 1e-6;

/// A component is a part of an optical system that can interact with light
/// rays.
///
/// Components come in two types: elements, and stops. Elements are the most
/// basic compound optical component and are represented as a set of surfaces
/// pairs. Stops are hard stops that block light rays.
///
/// To avoid copying data, only indexes are stored from the surface models are
/// stored.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Component {
    Element { surf_idxs: (usize, usize) },
    Stop { stop_idx: usize },
    UnpairedSurface { surf_idx: usize },
}

/// Determine the components of an optical system.
///
/// Components are the basic building blocks of an optical system. They are
/// either elements or stops. Elements are pairs of surfaces that interact with
/// light rays. Stops are hard stops that block light rays.
///
/// Components serve to group surfaces together into individual lenses.
///
/// # Arguments
/// * `sequential_model` - The sequential model of the optical system.
/// * `background` - The refractive index of the background medium.
pub fn components_view(
    sequential_model: &SequentialModel,
    background: RefractiveIndexSpec,
) -> Result<HashSet<Component>> {
    let mut components = HashSet::new();

    let surfaces = sequential_model.surfaces();
    let surface_pairs = surfaces.iter().zip(surfaces.iter().skip(1)).enumerate();
    let max_idx = surfaces.len() - 1;
    let mut paired_surfaces = HashSet::new();

    // TODO: This is a temporary solution to get the submodel due to the need for
    // gaps. Ignore wavelengths and axes, just get any submodel for now
    let sequential_sub_model = sequential_model
        .submodels()
        .values()
        .next()
        .ok_or(anyhow!("No submodels found in the sequential model."))?;
    let gaps = sequential_sub_model.gaps();

    let background_refractive_index = RefractiveIndex::try_from_spec(&background, None)?;

    if max_idx < 2 {
        // There are no components because only the object and image plane exist.
        return Ok(components);
    }

    for (i, surf_pair) in surface_pairs {
        if i == 0 || i == max_idx {
            // Don't include the object or image plane surfaces
            continue;
        }

        if let Surface::Stop(_) = surf_pair.0 {
            // Stops are special, so be sure that they're added before anything else.
            components.insert(Component::Stop { stop_idx: i });
            continue;
        }

        if let Surface::Stop(_) = surf_pair.1 {
            // Ensure that stops following surfaces are NOT added as a component
            continue;
        }

        if same_medium(gaps[i].refractive_index, background_refractive_index) {
            // Don't include surface pairs that go from background to another medium because
            // these are gaps.
            continue;
        }

        if let Surface::Image(_) = surf_pair.1 {
            // Check whether the next to last surface has already been paired with another.
            if !paired_surfaces.contains(&i) {
                components.insert(Component::UnpairedSurface { surf_idx: i });
                continue;
            }
        }

        components.insert(Component::Element {
            surf_idxs: (i, i + 1),
        });
        paired_surfaces.insert(i);
        paired_surfaces.insert(i + 1);
    }

    Ok(components)
}

/// Two different media are considered the same if their refractive indices are
/// within a small tolerance of each other.
fn same_medium(eta_1: RefractiveIndex, eta_2: RefractiveIndex) -> bool {
    (eta_1.n() - eta_2.n()).abs() < TOL && (eta_1.k() - eta_2.k()).abs() < TOL
}

#[cfg(test)]
mod tests {
    use crate::{core::Float, n, GapSpec, RefractiveIndexSpec, SequentialModel, SurfaceSpec};

    use super::*;

    const AIR: RefractiveIndexSpec = RefractiveIndexSpec {
        real: crate::RealSpec::Constant(1.0),
        imag: None,
    };

    const NBK7: RefractiveIndexSpec = RefractiveIndexSpec {
        real: crate::RealSpec::Constant(1.515),
        imag: None,
    };

    pub fn empty_system() -> SequentialModel {
        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: 1.0,
            refractive_index: RefractiveIndexSpec {
                real: crate::RealSpec::Constant(1.0),
                imag: None,
            },
        };
        let surf_1 = SurfaceSpec::Image;

        let surfaces = vec![surf_0, surf_1];
        let gaps = vec![gap_0];
        let wavelengths = vec![0.567];

        SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
    }

    pub fn silly_unpaired_surface() -> SequentialModel {
        // A silly system for edge case testing only.

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: Float::INFINITY,
            refractive_index: AIR,
        };
        let surf_1 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: 25.8,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        };
        let gap_1 = GapSpec {
            thickness: 5.3,
            refractive_index: NBK7,
        };
        let surf_2 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        };
        let gap_2 = GapSpec {
            thickness: 46.6,
            refractive_index: AIR,
        };
        let surf_3 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: 25.8,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        }; // Surface is unpaired
        let gap_3 = GapSpec {
            thickness: 20.0,
            refractive_index: NBK7,
        };
        let surf_4 = SurfaceSpec::Image;

        let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
        let gaps = vec![gap_0, gap_1, gap_2, gap_3];
        let wavelengths = vec![0.567];

        SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
    }

    pub fn silly_single_surface_and_stop() -> SequentialModel {
        // A silly system for edge case testing only.

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: Float::INFINITY,
            refractive_index: AIR,
        };
        let surf_1 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: 25.8,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        };
        let gap_1 = GapSpec {
            thickness: 10.0,
            refractive_index: NBK7,
        };
        let surf_2 = SurfaceSpec::Stop {
            semi_diameter: 12.5,
        };
        let gap_2 = GapSpec {
            thickness: 10.0,
            refractive_index: AIR,
        };
        let surf_3 = SurfaceSpec::Image;

        let surfaces = vec![surf_0, surf_1, surf_2, surf_3];
        let gaps = vec![gap_0, gap_1, gap_2];
        let wavelengths = vec![0.567];

        SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
    }

    pub fn wollaston_landscape_lens() -> SequentialModel {
        // Wollaston landscape lens: https://www.youtube.com/watch?v=YN6gTqYVYcw
        // f/5, EFL = 50 mm
        // Aperture stop is a hard stop in front of the lens

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: Float::INFINITY,
            refractive_index: n!(1.0),
        };
        let surf_1 = SurfaceSpec::Stop { semi_diameter: 5.0 };
        let gap_1 = GapSpec {
            thickness: 5.0,
            refractive_index: n!(1.0),
        };
        let surf_2 = SurfaceSpec::Conic {
            semi_diameter: 6.882,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        };
        let gap_2 = GapSpec {
            thickness: 5.0,
            refractive_index: n!(1.515),
        };
        let surf_3 = SurfaceSpec::Conic {
            semi_diameter: 7.367,
            radius_of_curvature: -25.84,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        };
        let gap_3 = GapSpec {
            thickness: 47.974,
            refractive_index: n!(1.0),
        };
        let surf_4 = SurfaceSpec::Image;

        let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
        let gaps = vec![gap_0, gap_1, gap_2, gap_3];
        let wavelengths = vec![0.5876];

        SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
    }

    // pub fn petzval_lens() -> SequentialModel {
    //     let surfaces = vec![
    //         SurfaceSpec::Object,
    //         SurfaceSpec::Conic {
    //             semi_diameter: 28.478,
    //             radius_of_curvature: 99.56266,
    //             conic_constant: 0.0,
    //             surf_type: crate::SurfaceType::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 26.276,
    //             radius_of_curvature: -86.84002,
    //             conic_constant: 0.0,
    //             surf_type: crate::SurfaceType::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 21.01,
    //             radius_of_curvature: -1187.63858,
    //             conic_constant: 0.0,
    //             surf_type: crate::SurfaceType::Refracting,
    //         },
    //         SurfaceSpec::Stop {
    //             semi_diameter: 33.262,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 20.543,
    //             radius_of_curvature: 57.47491,
    //             conic_constant: 0.0,
    //             surf_type: crate::SurfaceType::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 20.074,
    //             radius_of_curvature: -54.61685,
    //             conic_constant: 0.0,
    //             surf_type: crate::SurfaceType::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 16.492,
    //             radius_of_curvature: -614.68633,
    //             conic_constant: 0.0,
    //             surf_type: crate::SurfaceType::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 17.297,
    //             radius_of_curvature: -38.17110,
    //             conic_constant: 0.0,
    //             surf_type: crate::SurfaceType::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 18.94,
    //             radius_of_curvature: Float::INFINITY,
    //             conic_constant: 0.0,
    //             surf_type: crate::SurfaceType::Refracting,
    //         },
    //         SurfaceSpec::Image,
    //     ];
    //     let gaps = vec![
    //         GapSpec::from_thickness_and_real_refractive_index(Float::INFINITY,
    // 1.0),         GapSpec::from_thickness_and_real_refractive_index(13.0,
    // 1.5168),         GapSpec::from_thickness_and_real_refractive_index(4.0,
    // 1.6645),         GapSpec::from_thickness_and_real_refractive_index(40.0,
    // 1.0),         GapSpec::from_thickness_and_real_refractive_index(40.0,
    // 1.0),         GapSpec::from_thickness_and_real_refractive_index(12.0,
    // 1.6074),         GapSpec::from_thickness_and_real_refractive_index(3.0,
    // 1.6727),         GapSpec::from_thickness_and_real_refractive_index(46.
    // 82210, 1.0),         GapSpec::from_thickness_and_real_refractive_index(2.
    // 0, 1.6727),         GapSpec::from_thickness_and_real_refractive_index(1.
    // 87179, 1.0),     ];
    //     let wavelengths = vec![0.5876];

    //     SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
    // }

    #[test]
    fn test_new_no_components() {
        let sequential_model = empty_system();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 0);
    }

    #[test]
    fn test_planoconvex_lens() {
        let sequential_model = crate::examples::convexplano_lens::sequential_model();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 1);
        assert!(components.contains(&Component::Element { surf_idxs: (1, 2) }));
    }

    #[test]
    fn test_silly_single_surface_and_stop() {
        // This is not a useful system but a good test.
        let sequential_model = silly_single_surface_and_stop();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 1);
        assert!(components.contains(&Component::Stop { stop_idx: 2 })); // Hard stop
    }

    #[test]
    fn test_silly_unpaired_surface() {
        // This is not a useful system but a good test.
        let sequential_model = silly_unpaired_surface();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 2);
        assert!(components.contains(&Component::Element { surf_idxs: (1, 2) }));
        assert!(components.contains(&Component::UnpairedSurface { surf_idx: 3 }));
    }

    #[test]
    fn test_wollaston_landscape_lens() {
        let sequential_model = wollaston_landscape_lens();

        let components = components_view(&sequential_model, AIR).unwrap();

        assert_eq!(components.len(), 2);
        assert!(components.contains(&Component::Stop { stop_idx: 1 })); // Hard stop
        assert!(components.contains(&Component::Element { surf_idxs: (2, 3) }));
        // Lens
    }
}
