use std::collections::HashSet;
use std::rc::Rc;

use anyhow::{Result, anyhow};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    BoundaryKind, RefractiveIndexSpec, SequentialModel, SequentialSubModel, SurfaceKind,
    core::{Float, refractive_index::RefractiveIndex},
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Component {
    /// A refracting element. `surf_idxs` contains the ordered surface indices
    /// (front, optional cemented interfaces, back). Length is always ≥ 2.
    Element {
        surf_idxs: Vec<usize>,
    },
    Iris {
        stop_idx: usize,
    },
    Mirror {
        surf_idx: usize,
    },
    UnpairedSurface {
        surf_idx: usize,
    },
}

/// Determine the components of an optical system.
///
/// Components are the basic building blocks of an optical system. They are
/// either elements or irises. Elements are pairs of surfaces that interact with
/// light rays. Irises are hard stops that clip rays.
///
/// Components serve to group surfaces together into individual lenses.
///
/// # Arguments
/// * `sequential_model` - The sequential model of the optical system.
/// * `background` - The refractive index of the background medium.
pub fn components_view(
    sequential_model: &SequentialModel,
    background: Rc<dyn RefractiveIndexSpec>,
) -> Result<Vec<Component>> {
    let surfaces = sequential_model.surfaces();
    let n_surfs = surfaces.len();

    let wavelength = sequential_model
        .wavelengths()
        .first()
        .copied()
        .unwrap_or(0.5876);

    let background_ri = RefractiveIndex::try_from_spec(background.as_ref(), wavelength)?;

    let sequential_sub_model = sequential_model
        .submodel(0)
        .ok_or(anyhow!("No submodel found for wavelength index 0."))?;
    let gaps = sequential_sub_model.gaps();

    if n_surfs < 3 {
        // Only object and image plane exist; no real components.
        return Ok(vec![]);
    }

    // Collect non-element components (mirrors, irises) and track which surfaces
    // are already claimed so we can detect unpaired surfaces later.
    let mut non_elements: Vec<Component> = Vec::new();
    let mut claimed: HashSet<usize> = HashSet::new();

    for (i, surface) in surfaces.iter().enumerate().skip(1).take(n_surfs - 2) {
        let kind = surface.surface_kind();
        if matches!(surface.boundary_kind(), BoundaryKind::Reflecting) {
            non_elements.push(Component::Mirror { surf_idx: i });
            claimed.insert(i);
        } else if kind == SurfaceKind::Iris {
            non_elements.push(Component::Iris { stop_idx: i });
            claimed.insert(i);
        }
    }

    // Pass 1: for every non-background gap, find the nearest non-probe surface
    // on each side and emit a candidate length-2 element.  Probes inside a glass
    // run are skipped so they don't split an element.
    let mut candidates: Vec<Vec<usize>> = Vec::new();

    'gap_loop: for (gap_idx, gap) in gaps.iter().enumerate().skip(1).take(n_surfs - 2) {
        if same_medium(gap.refractive_index, background_ri) {
            continue; // background gap — not inside glass
        }

        // The front surface is the surface at index gap_idx (left side of the
        // gap), walking backwards past any probes to find a real boundary.
        let mut front = gap_idx;
        while front > 0 && surfaces[front].surface_kind() == SurfaceKind::Probe {
            front -= 1;
        }

        // The back surface is gap_idx+1, walking forwards past any probes.
        let mut back = gap_idx + 1;
        while back < n_surfs && surfaces[back].surface_kind() == SurfaceKind::Probe {
            back += 1;
        }

        // Skip if probe-walk escaped the model bounds or landed on object/image.
        if front == 0
            || back >= n_surfs
            || back == n_surfs - 1 && surfaces[back].surface_kind() == SurfaceKind::Image
        {
            continue 'gap_loop;
        }

        // Skip if either boundary is a mirror or iris (handled above).
        if claimed.contains(&front) || claimed.contains(&back) {
            continue 'gap_loop;
        }

        candidates.push(vec![front, back]);
    }

    // Pass 2: merge candidates that share a surface index into compound elements
    // (cemented doublets, triplets, etc.).  Repeat until stable.
    let mut changed = true;
    while changed {
        changed = false;
        let mut i = 0;
        while i < candidates.len() {
            let mut merged = false;
            for j in (i + 1)..candidates.len() {
                let shared = candidates[i].iter().any(|s| candidates[j].contains(s));
                if shared {
                    // Merge j into i.
                    let other = candidates.remove(j);
                    for s in other {
                        if !candidates[i].contains(&s) {
                            candidates[i].push(s);
                        }
                    }
                    candidates[i].sort_unstable();
                    changed = true;
                    merged = true;
                    break;
                }
            }
            if !merged {
                i += 1;
            }
        }
    }

    // Convert candidates to Element components, tracking which surfaces are now
    // part of an element so unpaired surfaces can be detected.
    let mut elements: Vec<Component> = Vec::new();
    for mut surfs in candidates {
        surfs.sort_unstable();
        for &s in &surfs {
            claimed.insert(s);
        }
        elements.push(Component::Element { surf_idxs: surfs });
    }

    // Detect unpaired surfaces: refracting surfaces that border at least one
    // non-background gap but were not merged into any element. This includes
    // the surface just before Image and surfaces adjacent to an iris or mirror
    // in a non-background medium (e.g. an iris submerged in glass).
    for i in 1..(n_surfs - 1) {
        if claimed.contains(&i) {
            continue;
        }
        let kind = surfaces[i].surface_kind();
        if kind == SurfaceKind::Object
            || kind == SurfaceKind::Image
            || kind == SurfaceKind::Probe
            || kind == SurfaceKind::Iris
            || matches!(surfaces[i].boundary_kind(), BoundaryKind::Reflecting)
        {
            continue;
        }
        let borders_non_background = !same_medium(gaps[i - 1].refractive_index, background_ri)
            || !same_medium(gaps[i].refractive_index, background_ri);
        if borders_non_background {
            non_elements.push(Component::UnpairedSurface { surf_idx: i });
            claimed.insert(i);
        }
    }

    // Combine all components and sort by first surface index.
    let mut result: Vec<Component> = elements.into_iter().chain(non_elements).collect();
    result.sort_by_key(|c| match c {
        Component::Element { surf_idxs } => *surf_idxs.first().unwrap_or(&usize::MAX),
        Component::Iris { stop_idx } => *stop_idx,
        Component::Mirror { surf_idx } => *surf_idx,
        Component::UnpairedSurface { surf_idx } => *surf_idx,
    });
    Ok(result)
}

/// Two different media are considered the same if their refractive indices are
/// within a small tolerance of each other.
///
/// Both values must be evaluated at the **same wavelength** before calling
/// this function; comparing values evaluated at different wavelengths will
/// produce incorrect results for dispersive materials.
fn same_medium(eta_1: RefractiveIndex, eta_2: RefractiveIndex) -> bool {
    (eta_1.n() - eta_2.n()).abs() < TOL && (eta_1.k() - eta_2.k()).abs() < TOL
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::examples::{concave_mirror, convexplano_lens};
    use crate::{GapSpec, Rotation3D, SequentialModel, SurfaceSpec, Vec3, core::Float, n};

    use super::*;

    pub fn empty_system() -> SequentialModel {
        let air: Rc<dyn RefractiveIndexSpec> = n!(1.0);

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: 1.0,
            refractive_index: air,
        };
        let surf_1 = SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };

        let surfaces = vec![surf_0, surf_1];
        let gaps = vec![gap_0];
        let wavelengths = vec![0.567];

        SequentialModel::from_surface_specs(&gaps, &surfaces, &wavelengths, None).unwrap()
    }

    pub fn silly_unpaired_surface() -> SequentialModel {
        // A silly system for edge case testing only.
        let air = n!(1.0);
        let nbk7 = n!(1.515);

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: Float::INFINITY,
            refractive_index: air.clone(),
        };
        let surf_1 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: 25.8,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_1 = GapSpec {
            thickness: 5.3,
            refractive_index: nbk7.clone(),
        };
        let surf_2 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_2 = GapSpec {
            thickness: 46.6,
            refractive_index: air,
        };
        let surf_3 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: 25.8,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        }; // Surface is unpaired
        let gap_3 = GapSpec {
            thickness: 20.0,
            refractive_index: nbk7,
        };
        let surf_4 = SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };

        let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
        let gaps = vec![gap_0, gap_1, gap_2, gap_3];
        let wavelengths = vec![0.567];

        SequentialModel::from_surface_specs(&gaps, &surfaces, &wavelengths, None).unwrap()
    }

    pub fn silly_single_surface_and_stop() -> SequentialModel {
        // A silly system for edge case testing only.
        let air = n!(1.0);
        let nbk7 = n!(1.515);

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: Float::INFINITY,
            refractive_index: air.clone(),
        };
        let surf_1 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: 25.8,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_1 = GapSpec {
            thickness: 10.0,
            refractive_index: nbk7,
        };
        let surf_2 = SurfaceSpec::Iris {
            semi_diameter: 12.5,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_2 = GapSpec {
            thickness: 10.0,
            refractive_index: air,
        };
        let surf_3 = SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };

        let surfaces = vec![surf_0, surf_1, surf_2, surf_3];
        let gaps = vec![gap_0, gap_1, gap_2];
        let wavelengths = vec![0.567];

        SequentialModel::from_surface_specs(&gaps, &surfaces, &wavelengths, None).unwrap()
    }

    pub fn wollaston_landscape_lens() -> SequentialModel {
        // Wollaston landscape lens: https://www.youtube.com/watch?v=YN6gTqYVYcw
        // f/5, EFL = 50 mm
        // Aperture stop is a hard stop in front of the lens
        let air = n!(1.0);
        let nbk7 = n!(1.515);

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: Float::INFINITY,
            refractive_index: air.clone(),
        };
        let surf_1 = SurfaceSpec::Iris {
            semi_diameter: 5.0,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_1 = GapSpec {
            thickness: 5.0,
            refractive_index: air.clone(),
        };
        let surf_2 = SurfaceSpec::Conic {
            semi_diameter: 6.882,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_2 = GapSpec {
            thickness: 5.0,
            refractive_index: nbk7,
        };
        let surf_3 = SurfaceSpec::Conic {
            semi_diameter: 7.367,
            radius_of_curvature: -25.84,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_3 = GapSpec {
            thickness: 47.974,
            refractive_index: air,
        };
        let surf_4 = SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };

        let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
        let gaps = vec![gap_0, gap_1, gap_2, gap_3];
        let wavelengths = vec![0.5876];

        SequentialModel::from_surface_specs(&gaps, &surfaces, &wavelengths, None).unwrap()
    }

    // pub fn petzval_lens() -> SequentialModel {
    //     let surfaces = vec![
    //         SurfaceSpec::Object,
    //         SurfaceSpec::Conic {
    //             semi_diameter: 28.478,
    //             radius_of_curvature: 99.56266,
    //             conic_constant: 0.0,
    //             surf_kind: crate::BoundaryKind::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 26.276,
    //             radius_of_curvature: -86.84002,
    //             conic_constant: 0.0,
    //             surf_kind: crate::BoundaryKind::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 21.01,
    //             radius_of_curvature: -1187.63858,
    //             conic_constant: 0.0,
    //             surf_kind: crate::BoundaryKind::Refracting,
    //         },
    //         SurfaceSpec::Iris {
    //             semi_diameter: 33.262,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 20.543,
    //             radius_of_curvature: 57.47491,
    //             conic_constant: 0.0,
    //             surf_kind: crate::BoundaryKind::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 20.074,
    //             radius_of_curvature: -54.61685,
    //             conic_constant: 0.0,
    //             surf_kind: crate::BoundaryKind::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 16.492,
    //             radius_of_curvature: -614.68633,
    //             conic_constant: 0.0,
    //             surf_kind: crate::BoundaryKind::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 17.297,
    //             radius_of_curvature: -38.17110,
    //             conic_constant: 0.0,
    //             surf_kind: crate::BoundaryKind::Refracting,
    //         },
    //         SurfaceSpec::Conic {
    //             semi_diameter: 18.94,
    //             radius_of_curvature: Float::INFINITY,
    //             conic_constant: 0.0,
    //             surf_kind: crate::BoundaryKind::Refracting,
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

    //     SequentialModel::from_surface_specs(&gaps, &surfaces, &wavelengths,
    // None).unwrap() }

    #[test]
    fn test_concave_mirror() {
        let sequential_model = concave_mirror::sequential_model(n!(1.0), &[0.5876]);

        let components = components_view(&sequential_model, n!(1.0)).unwrap();

        assert_eq!(components.len(), 1);
        assert!(components.contains(&Component::Mirror { surf_idx: 1 }));
    }

    #[test]
    fn test_new_no_components() {
        let sequential_model = empty_system();

        let components = components_view(&sequential_model, n!(1.0)).unwrap();

        assert_eq!(components.len(), 0);
    }

    #[test]
    fn test_planoconvex_lens() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let sequential_model = convexplano_lens::sequential_model(air, nbk7, &wavelengths);
        let components = components_view(&sequential_model, n!(1.0)).unwrap();

        assert_eq!(components.len(), 1);
        assert!(components.contains(&Component::Element {
            surf_idxs: vec![1, 2]
        }));
    }

    #[test]
    fn test_silly_single_surface_and_stop() {
        // Sphere1 borders a glass gap on its right side even though the iris
        // is claimed — it must be emitted as an UnpairedSurface.
        let sequential_model = silly_single_surface_and_stop();

        let components = components_view(&sequential_model, n!(1.0)).unwrap();

        assert_eq!(components.len(), 2);
        assert!(components.contains(&Component::Iris { stop_idx: 2 }));
        assert!(components.contains(&Component::UnpairedSurface { surf_idx: 1 }));
    }

    #[test]
    fn test_silly_unpaired_surface() {
        // This is not a useful system but a good test.
        let sequential_model = silly_unpaired_surface();

        let components = components_view(&sequential_model, n!(1.0)).unwrap();

        assert_eq!(components.len(), 2);
        assert!(components.contains(&Component::Element {
            surf_idxs: vec![1, 2]
        }));
        assert!(components.contains(&Component::UnpairedSurface { surf_idx: 3 }));
    }

    #[test]
    fn test_wollaston_landscape_lens() {
        let sequential_model = wollaston_landscape_lens();

        let components = components_view(&sequential_model, n!(1.0)).unwrap();

        assert_eq!(components.len(), 2);
        assert!(components.contains(&Component::Iris { stop_idx: 1 })); // Hard stop
        assert!(components.contains(&Component::Element {
            surf_idxs: vec![2, 3]
        }));
        // Lens
    }

    pub fn mirror_then_probe() -> SequentialModel {
        let air = n!(1.0);
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Conic {
                semi_diameter: 12.5,
                radius_of_curvature: -200.0,
                conic_constant: 0.0,
                surf_kind: crate::BoundaryKind::Reflecting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Probe {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];
        let gaps = vec![
            GapSpec {
                thickness: Float::INFINITY,
                refractive_index: air.clone(),
            },
            GapSpec {
                thickness: 50.0,
                refractive_index: air.clone(),
            },
            GapSpec {
                thickness: 50.0,
                refractive_index: air.clone(),
            },
        ];
        SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], None).unwrap()
    }

    #[test]
    fn test_mirror_before_probe() {
        // Regression: mirror must appear even when a probe sits between it and Image.
        let model = mirror_then_probe();
        let components = components_view(&model, n!(1.0)).unwrap();
        assert_eq!(components.len(), 1);
        assert!(components.contains(&Component::Mirror { surf_idx: 1 }));
    }

    #[test]
    fn test_f_theta_scan_lens() {
        // With the correct background (Ciddor air ≈ 1.00029), air gaps between
        // lens elements must be recognized as background so that the three
        // glass elements are kept separate rather than merged.
        use crate::examples::f_theta_scan_lens;
        let air = n!(1.00029);
        let glass = n!(1.847);
        let model = f_theta_scan_lens::sequential_model(air.clone(), glass, &[0.5876]);
        let components = components_view(&model, air).unwrap();
        assert_eq!(components.len(), 4); // 1 stop + 3 elements
        assert!(components.contains(&Component::Iris { stop_idx: 1 }));
        assert!(components.contains(&Component::Element {
            surf_idxs: vec![2, 3]
        }));
        assert!(components.contains(&Component::Element {
            surf_idxs: vec![4, 5]
        }));
        assert!(components.contains(&Component::Element {
            surf_idxs: vec![6, 7]
        }));
    }

    pub fn cemented_doublet() -> SequentialModel {
        // Crown-flint cemented doublet: air → BK7 → SF2 → air
        // Surfaces: Object[0], Conic[1] (front), Conic[2] (cemented), Conic[3] (back),
        // Image[4]
        let air = n!(1.0);
        let bk7 = n!(1.515);
        let sf2 = n!(1.648);

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: Float::INFINITY,
            refractive_index: air.clone(),
        };
        let surf_1 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: 50.0,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_1 = GapSpec {
            thickness: 6.0,
            refractive_index: bk7,
        };
        let surf_2 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: -30.0,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_2 = GapSpec {
            thickness: 3.0,
            refractive_index: sf2,
        };
        let surf_3 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: -100.0,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_3 = GapSpec {
            thickness: 50.0,
            refractive_index: air,
        };
        let surf_4 = SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };

        let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
        let gaps = vec![gap_0, gap_1, gap_2, gap_3];
        SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], None).unwrap()
    }

    #[test]
    fn test_cemented_doublet_is_single_element() {
        // A cemented doublet (BK7 + SF2) must be detected as one element spanning
        // all three bounding surfaces [1, 2, 3], not two separate elements.
        let model = cemented_doublet();
        let components = components_view(&model, n!(1.0)).unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(
            components[0],
            Component::Element {
                surf_idxs: vec![1, 2, 3]
            }
        );
    }

    pub fn singlet_with_probe() -> SequentialModel {
        // Singlet with a probe surface inside the glass: Object[0], Conic[1],
        // Probe[2] (inside glass), Conic[3], Image[4]. The probe must not split
        // the element.
        let air = n!(1.0);
        let bk7 = n!(1.515);

        let surf_0 = SurfaceSpec::Object;
        let gap_0 = GapSpec {
            thickness: Float::INFINITY,
            refractive_index: air.clone(),
        };
        let surf_1 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: 50.0,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_1 = GapSpec {
            thickness: 4.0,
            refractive_index: bk7.clone(),
        };
        let surf_2 = SurfaceSpec::Probe {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_2 = GapSpec {
            thickness: 2.0,
            refractive_index: bk7,
        };
        let surf_3 = SurfaceSpec::Conic {
            semi_diameter: 12.5,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            surf_kind: crate::BoundaryKind::Refracting,
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };
        let gap_3 = GapSpec {
            thickness: 50.0,
            refractive_index: air,
        };
        let surf_4 = SurfaceSpec::Image {
            rotation: Rotation3D::None,
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation_offset: Rotation3D::None,
        };

        let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
        let gaps = vec![gap_0, gap_1, gap_2, gap_3];
        SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], None).unwrap()
    }

    #[test]
    fn test_probe_inside_glass_does_not_split_element() {
        // A probe inside the glass of a singlet must not split it into two elements.
        // Expected: one Element with surf_idxs [1, 3], skipping the probe at [2].
        let model = singlet_with_probe();
        let components = components_view(&model, n!(1.0)).unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(
            components[0],
            Component::Element {
                surf_idxs: vec![1, 3]
            }
        );
    }

    pub fn iris_in_glass() -> SequentialModel {
        // Iris submerged in glass on both sides: Object → Sphere → Iris → Sphere →
        // Image. Both gaps adjacent to the iris are non-background (glass), so
        // each surrounding Sphere is an independent ungrouped surface.
        let air = n!(1.0);
        let glass = n!(1.517);
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 12.7,
                radius_of_curvature: 102.4,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Iris {
                semi_diameter: 13.0,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Sphere {
                semi_diameter: 12.7,
                radius_of_curvature: -102.4,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];
        let gaps = vec![
            GapSpec {
                thickness: 200.0,
                refractive_index: air.clone(),
            },
            GapSpec {
                thickness: 1.8,
                refractive_index: glass.clone(),
            },
            GapSpec {
                thickness: 1.8,
                refractive_index: glass,
            },
            GapSpec {
                thickness: 196.0,
                refractive_index: air,
            },
        ];
        SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], None).unwrap()
    }

    #[test]
    fn test_iris_in_glass() {
        // Regression: when an iris sits between two non-background surfaces,
        // both surrounding refracting surfaces must appear as UnpairedSurface.
        let model = iris_in_glass();
        let components = components_view(&model, n!(1.0)).unwrap();

        assert_eq!(components.len(), 3);
        assert!(components.contains(&Component::Iris { stop_idx: 2 }));
        assert!(components.contains(&Component::UnpairedSurface { surf_idx: 1 }));
        assert!(components.contains(&Component::UnpairedSurface { surf_idx: 3 }));
    }
}
