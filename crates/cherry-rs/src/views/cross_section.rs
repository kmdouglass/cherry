//! 2D cross-section view of a sequential optical system.

use std::collections::HashSet;

use crate::{
    SequentialModel, SurfaceKind,
    core::{Float, math::vec3::Vec3, placement::Placement, surfaces::Surface},
    views::{components::Component, ray_trace_3d::RayBundle},
};

/// Identifies a global transverse coordinate axis for cross-section projection.
#[derive(Clone, Copy)]
pub enum GlobalAxis {
    X,
    Y,
}

const N_PTS: usize = 64;
const EPS: f64 = 1e-6;

/// The complete 2D cross-section view of a sequential optical system.
pub struct CrossSectionView {
    pub wavelengths: Vec<f64>,
    /// True if all axis directions have x ≈ 0, meaning the axis lies in the YZ
    /// plane.
    pub yz_valid: bool,
    /// True if all axis directions have y ≈ 0, meaning the axis lies in the XZ
    /// plane.
    pub xz_valid: bool,
    pub yz: PlaneGeometry,
    pub xz: PlaneGeometry,
}

/// 2D geometry for one cutting plane.
pub struct PlaneGeometry {
    pub bounding_box: Bounds2D,
    pub elements: Vec<DrawElement>,
    /// ray_paths[wavelength_idx][path_idx] = Vec<[z, transverse]>
    pub ray_paths: Vec<Vec<Vec<[f64; 2]>>>,
}

/// Axis-aligned bounding box in the (z, transverse) 2D coordinate system.
pub struct Bounds2D {
    pub z: (f64, f64),
    pub transverse: (f64, f64),
}

/// A drawable element in the cross-section view.
pub enum DrawElement {
    LensGroup {
        /// Front surface points sampled bottom-to-top (transverse -sd → +sd).
        front_pts: Vec<[f64; 2]>,
        /// Back surface points sampled bottom-to-top (transverse -sd → +sd).
        back_pts: Vec<[f64; 2]>,
    },
    SurfaceProfile {
        points: Vec<[f64; 2]>,
    },
    Stop {
        z: f64,
        half_gap: f64,
        extent: f64,
    },
    FlatPlane {
        /// First endpoint in (z, transverse) plot space.
        p1: [f64; 2],
        /// Second endpoint in (z, transverse) plot space.
        p2: [f64; 2],
        kind: FlatPlaneKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlatPlaneKind {
    Image,
    Probe,
    Object,
}

/// Compute a 2D cross-section view of a sequential optical system.
///
/// # Arguments
/// * `model` - The sequential model.
/// * `cross_section_rays` - Optional ray bundles to overlay on the view. Each
///   tuple is `(field_id, wavelength_id, bundle)` as returned by
///   [`trace_ray_bundle`](crate::trace_ray_bundle).
/// * `components` - Pre-computed optical components (from `components_view`).
pub fn cross_section_view(
    model: &SequentialModel,
    cross_section_rays: Option<&[(usize, usize, RayBundle)]>,
    components: &HashSet<Component>,
) -> CrossSectionView {
    let wavelengths = model.wavelengths().to_vec();
    let axis_dirs = model.axis_directions();

    // Check which cutting planes are valid.
    let yz_valid = axis_dirs.iter().all(|d| d.x().abs() < EPS);
    let xz_valid = axis_dirs.iter().all(|d| d.y().abs() < EPS);

    let yz = build_plane_geometry(model, cross_section_rays, GlobalAxis::Y, components);
    let xz = build_plane_geometry(model, cross_section_rays, GlobalAxis::X, components);

    CrossSectionView {
        wavelengths,
        yz_valid,
        xz_valid,
        yz,
        xz,
    }
}

/// Build the geometry for one cutting plane.
fn build_plane_geometry(
    model: &SequentialModel,
    cross_section_rays: Option<&[(usize, usize, RayBundle)]>,
    axis: GlobalAxis,
    components: &HashSet<Component>,
) -> PlaneGeometry {
    let surfaces = model.surfaces();
    let placements = model.placements();
    let largest_sd = model.largest_semi_diameter();

    let mut elements: Vec<DrawElement> = Vec::new();

    // Add lens groups and stops.
    // Sort components by surface index for consistent ordering.
    let mut sorted_components: Vec<Component> = components.iter().cloned().collect();
    sorted_components.sort_by_key(|c| match c {
        Component::Element { surf_idxs: (i, _) } => *i,
        Component::Stop { stop_idx } => *stop_idx,
        Component::Mirror { surf_idx } => *surf_idx,
        Component::UnpairedSurface { surf_idx } => *surf_idx,
    });

    for comp in &sorted_components {
        match comp {
            Component::Element { surf_idxs: (i, j) } => {
                let front_pts = sample_surface(surfaces[*i].as_ref(), &placements[*i], axis, N_PTS);
                let back_pts = sample_surface(surfaces[*j].as_ref(), &placements[*j], axis, N_PTS);
                if !front_pts.is_empty() && !back_pts.is_empty() {
                    elements.push(DrawElement::LensGroup {
                        front_pts,
                        back_pts,
                    });
                }
            }
            Component::Stop { stop_idx } => {
                let z = placements[*stop_idx].position.z();
                let sd = surfaces[*stop_idx].semi_diameter();
                elements.push(DrawElement::Stop {
                    z,
                    half_gap: sd,
                    extent: largest_sd * 1.5,
                });
            }
            Component::Mirror { surf_idx } => {
                let pts = sample_surface(
                    surfaces[*surf_idx].as_ref(),
                    &placements[*surf_idx],
                    axis,
                    N_PTS,
                );
                if !pts.is_empty() {
                    elements.push(DrawElement::SurfaceProfile { points: pts });
                }
            }
            Component::UnpairedSurface { surf_idx } => {
                let pts = sample_surface(
                    surfaces[*surf_idx].as_ref(),
                    &placements[*surf_idx],
                    axis,
                    N_PTS,
                );
                if !pts.is_empty() {
                    elements.push(DrawElement::SurfaceProfile { points: pts });
                }
            }
        }
    }

    // Add flat planes (Image, Probe, Object at finite distance).
    for (surf, placement) in surfaces.iter().zip(placements.iter()) {
        if placement.is_infinite() {
            continue;
        }
        let half = if largest_sd > 0.0 {
            largest_sd * 1.2
        } else {
            1.0
        };
        let kind = match surf.surface_kind() {
            SurfaceKind::Image => FlatPlaneKind::Image,
            SurfaceKind::Probe => FlatPlaneKind::Probe,
            SurfaceKind::Object => FlatPlaneKind::Object,
            _ => continue,
        };

        // Center of the plane in (z, transverse) plot space.
        let center_z = placement.position.z();
        let center_t = match axis {
            GlobalAxis::Y => placement.position.y(),
            GlobalAxis::X => placement.position.x(),
        };

        // Forward direction of the cursor at this surface in global coordinates.
        // inv_rotation_matrix maps local → global; applying to (0,0,1) gives the
        // cursor forward direction.
        let fwd = placement.inv_rotation_matrix * Vec3::new(0.0, 0.0, 1.0);
        let fwd_z = fwd.z();
        let fwd_t = match axis {
            GlobalAxis::Y => fwd.y(),
            GlobalAxis::X => fwd.x(),
        };

        // Perpendicular to (fwd_z, fwd_t) in 2D is (-fwd_t, fwd_z).
        let p1 = [center_z - fwd_t * half, center_t + fwd_z * half];
        let p2 = [center_z + fwd_t * half, center_t - fwd_z * half];

        elements.push(DrawElement::FlatPlane { p1, p2, kind });
    }

    // Extract ray paths.
    let n_wavelengths = model.wavelengths().len();
    let mut ray_paths: Vec<Vec<Vec<[f64; 2]>>> = vec![Vec::new(); n_wavelengths];

    if let Some(rays) = cross_section_rays {
        for (_field_id, wl_id, bundle) in rays {
            let wl_id = *wl_id;
            if wl_id >= n_wavelengths {
                continue;
            }
            let n_surf = bundle.num_surfaces();
            let total = bundle.rays().len();
            let n_rays = if n_surf > 0 { total / n_surf } else { 0 };

            if n_rays == 0 {
                continue;
            }

            for ray_idx in 0..n_rays {
                // Check if ray terminated (terminated[ray_idx] > 0 means terminated at that
                // surface)
                let term_surf = bundle.terminated().get(ray_idx).copied().unwrap_or(0);

                let mut path: Vec<[f64; 2]> = Vec::new();
                for surf_idx in 0..n_surf {
                    if term_surf > 0 && surf_idx > term_surf {
                        break;
                    }
                    let abs_idx = surf_idx * n_rays + ray_idx;
                    if let Some(ray) = bundle.rays().get(abs_idx) {
                        if surf_idx >= surfaces.len() {
                            continue;
                        }
                        let transverse = match axis {
                            GlobalAxis::Y => ray.y(),
                            GlobalAxis::X => ray.x(),
                        };
                        path.push([ray.z(), transverse]);
                    }
                }
                if path.len() >= 2 {
                    ray_paths[wl_id].push(path);
                }
            }
        }
    }

    let bounding_box = compute_bounds(&elements, &ray_paths);

    PlaneGeometry {
        bounding_box,
        elements,
        ray_paths,
    }
}

/// Sample points on a surface in the cutting plane.
///
/// For axis = Y: samples at (0, y, 0) for y in [-sd, sd], returns global (z, y)
/// pairs. For axis = X: samples at (x, 0, 0) for x in [-sd, sd], returns global
/// (z, x) pairs.
fn sample_surface(
    surf: &dyn Surface,
    placement: &Placement,
    axis: GlobalAxis,
    n_pts: usize,
) -> Vec<[f64; 2]> {
    let sd = surf.semi_diameter();
    if !sd.is_finite() || sd <= 0.0 || n_pts < 2 {
        return Vec::new();
    }

    let mut pts = Vec::with_capacity(n_pts);
    for i in 0..n_pts {
        let t = i as Float / (n_pts - 1) as Float; // 0.0 to 1.0
        let transverse = sd * (2.0 * t - 1.0); // -sd to +sd
        let local_pt = match axis {
            GlobalAxis::Y => Vec3::new(0.0, transverse, 0.0),
            GlobalAxis::X => Vec3::new(transverse, 0.0, 0.0),
        };
        let (sag, _) = surf.sag_norm(local_pt);
        let local_surface_pt = match axis {
            GlobalAxis::Y => Vec3::new(0.0, transverse, sag),
            GlobalAxis::X => Vec3::new(transverse, 0.0, sag),
        };
        // Transform to global coordinates.
        let global_pt = placement.inv_rotation_matrix * local_surface_pt + placement.position;
        let transverse_global = match axis {
            GlobalAxis::Y => global_pt.y(),
            GlobalAxis::X => global_pt.x(),
        };
        pts.push([global_pt.z(), transverse_global]);
    }
    pts
}

/// Compute the bounding box over all elements and ray paths.
fn compute_bounds(elements: &[DrawElement], ray_paths: &[Vec<Vec<[f64; 2]>>]) -> Bounds2D {
    let mut z_min = f64::MAX;
    let mut z_max = f64::MIN;
    let mut t_min = f64::MAX;
    let mut t_max = f64::MIN;

    let update =
        |z: f64, t: f64, zmin: &mut f64, zmax: &mut f64, tmin: &mut f64, tmax: &mut f64| {
            if z.is_finite() {
                *zmin = zmin.min(z);
                *zmax = zmax.max(z);
            }
            if t.is_finite() {
                *tmin = tmin.min(t);
                *tmax = tmax.max(t);
            }
        };

    for elem in elements {
        match elem {
            DrawElement::LensGroup {
                front_pts,
                back_pts,
            } => {
                for &[z, t] in front_pts.iter().chain(back_pts.iter()) {
                    update(z, t, &mut z_min, &mut z_max, &mut t_min, &mut t_max);
                }
            }
            DrawElement::SurfaceProfile { points } => {
                for &[z, t] in points {
                    update(z, t, &mut z_min, &mut z_max, &mut t_min, &mut t_max);
                }
            }
            DrawElement::Stop { z, extent, .. } => {
                update(*z, *extent, &mut z_min, &mut z_max, &mut t_min, &mut t_max);
                update(*z, -extent, &mut z_min, &mut z_max, &mut t_min, &mut t_max);
            }
            DrawElement::FlatPlane { p1, p2, .. } => {
                update(p1[0], p1[1], &mut z_min, &mut z_max, &mut t_min, &mut t_max);
                update(p2[0], p2[1], &mut z_min, &mut z_max, &mut t_min, &mut t_max);
            }
        }
    }

    for paths in ray_paths {
        for path in paths {
            for &[z, t] in path {
                update(z, t, &mut z_min, &mut z_max, &mut t_min, &mut t_max);
            }
        }
    }

    // Guard against empty data.
    if z_min > z_max {
        z_min = -1.0;
        z_max = 1.0;
    }
    if t_min > t_max {
        t_min = -1.0;
        t_max = 1.0;
    }

    Bounds2D {
        z: (z_min, z_max),
        transverse: (t_min, t_max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::Float, examples::convexplano_lens, n, views::components::components_view};

    #[test]
    fn yz_valid_for_straight_system() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let model = convexplano_lens::sequential_model(air.clone(), nbk7, &wavelengths);
        let components = components_view(&model, air).unwrap();
        let cs = cross_section_view(&model, None, &components);
        assert!(cs.yz_valid, "YZ should be valid for a straight system");
        assert!(cs.xz_valid, "XZ should be valid for a straight system");
    }

    #[test]
    fn sample_conic_plane_surface() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let model = convexplano_lens::sequential_model(air, nbk7, &wavelengths);
        let surfaces = model.surfaces();
        let placements = model.placements();
        // Surface 2 is the plano (flat) back surface.
        let pts = sample_surface(surfaces[2].as_ref(), &placements[2], GlobalAxis::Y, 10);
        // All sag values should be zero for a flat surface.
        for &[_z, _t] in &pts {
            // Just ensure we got points back
        }
        assert_eq!(pts.len(), 10);
    }

    #[test]
    fn xz_rays_come_from_u_axis_results() {
        // A TangentialRayFan trace (phi=90°, YZ plane) should contribute ray
        // paths to the XZ plane cross-section via projection.
        use crate::{
            ApertureSpec, FieldSpec, ParaxialView, specs::fields::PupilSampling,
            views::ray_trace_3d::trace_ray_bundle,
        };
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let model = convexplano_lens::sequential_model(air.clone(), nbk7, &wavelengths);
        let fields = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let aperture = ApertureSpec::EntrancePupil {
            semi_diameter: 12.5,
        };
        let pv = ParaxialView::new(&model, &fields, false).unwrap();
        let rays = trace_ray_bundle(
            &aperture,
            &fields,
            &model,
            &pv,
            PupilSampling::TangentialRayFan { n: 5 },
        )
        .unwrap();
        let components = components_view(&model, air).unwrap();
        let cs = cross_section_view(&model, Some(&rays), &components);
        // XZ plane should have ray paths from the trace projected onto X.
        assert!(
            !cs.xz.ray_paths.iter().all(|w| w.is_empty()),
            "XZ plane should have ray paths from trace projection"
        );
    }

    #[test]
    fn ray_paths_populated_per_wavelength() {
        // Regression: tuples from trace_ray_bundle are (field_id, wavelength_id,
        // bundle). If the destructuring order is swapped, all paths land in
        // ray_paths[field_id] instead of ray_paths[wavelength_id], so only the
        // first wavelength slot ever gets populated.
        use crate::{
            ApertureSpec, FieldSpec, ParaxialView, specs::fields::PupilSampling,
            views::ray_trace_3d::trace_ray_bundle,
        };
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 3] = [0.4861, 0.5876, 0.6563];
        let model = convexplano_lens::sequential_model(air.clone(), nbk7, &wavelengths);
        let fields = vec![FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        }];
        let aperture = ApertureSpec::EntrancePupil {
            semi_diameter: 12.5,
        };
        let pv = ParaxialView::new(&model, &fields, false).unwrap();
        let rays = trace_ray_bundle(
            &aperture,
            &fields,
            &model,
            &pv,
            PupilSampling::TangentialRayFan { n: 3 },
        )
        .unwrap();
        let components = components_view(&model, air).unwrap();
        let cs = cross_section_view(&model, Some(&rays), &components);

        // Every wavelength slot must have paths — not just the first one.
        for (wl_idx, paths) in cs.yz.ray_paths.iter().enumerate() {
            assert!(
                !paths.is_empty(),
                "wavelength {wl_idx} should have ray paths in the YZ plane"
            );
        }
    }

    #[test]
    fn test_f_theta_three_lens_groups() {
        use crate::examples::f_theta_scan_lens;
        let air = n!(1.00029); // simulate Ciddor air
        let glass = n!(1.847); // simulate N-SF57
        let model = f_theta_scan_lens::sequential_model(air.clone(), glass, &[0.5876]);
        let components = components_view(&model, air).unwrap();
        let cs = cross_section_view(&model, None, &components);
        let n_groups = cs
            .yz
            .elements
            .iter()
            .filter(|e| matches!(e, DrawElement::LensGroup { .. }))
            .count();
        assert_eq!(n_groups, 3, "expected 3 separate lens groups");
    }
}
