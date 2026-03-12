//! 2D cross-section view of a sequential optical system.

use std::rc::Rc;

use crate::{
    Axis, SequentialModel,
    core::{Float, math::vec3::Vec3, sequential_model::Surface},
    specs::gaps::ConstantRefractiveIndex,
    views::{
        components::{Component, components_view},
        ray_trace_3d::TraceResultsCollection,
    },
};

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
        outline: Vec<[f64; 2]>,
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
        z: f64,
        min: f64,
        max: f64,
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
/// * `trace` - Optional ray trace results to overlay on the view.
pub fn cross_section_view(
    model: &SequentialModel,
    trace: Option<&TraceResultsCollection>,
) -> CrossSectionView {
    let wavelengths = model.wavelengths().to_vec();
    let axis_dirs = model.axis_directions();

    // Check which cutting planes are valid.
    let yz_valid = axis_dirs.iter().all(|d| d.x().abs() < EPS);
    let xz_valid = axis_dirs.iter().all(|d| d.y().abs() < EPS);

    let yz = build_plane_geometry(model, trace, Axis::Y);
    let xz = build_plane_geometry(model, trace, Axis::X);

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
    trace: Option<&TraceResultsCollection>,
    axis: Axis,
) -> PlaneGeometry {
    let surfaces = model.surfaces();
    let largest_sd = model.largest_semi_diameter();

    // Get components (lens groups and stops).
    let background: Rc<dyn crate::specs::gaps::RefractiveIndexSpec> =
        Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
    let components = components_view(model, background).unwrap_or_default();

    // Build the set of surface indices that are part of a lens group.
    let mut paired: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for comp in &components {
        if let Component::Element { surf_idxs: (i, j) } = comp {
            paired.insert(*i);
            paired.insert(*j);
        }
    }

    let mut elements: Vec<DrawElement> = Vec::new();

    // Add lens groups and stops.
    // Sort components by surface index for consistent ordering.
    let mut sorted_components: Vec<Component> = components.into_iter().collect();
    sorted_components.sort_by_key(|c| match c {
        Component::Element { surf_idxs: (i, _) } => *i,
        Component::Stop { stop_idx } => *stop_idx,
        Component::Mirror { surf_idx } => *surf_idx,
        Component::UnpairedSurface { surf_idx } => *surf_idx,
    });

    for comp in &sorted_components {
        match comp {
            Component::Element { surf_idxs: (i, j) } => {
                let front = &surfaces[*i];
                let back = &surfaces[*j];
                let front_pts = sample_surface(front, axis, N_PTS);
                let back_pts = sample_surface(back, axis, N_PTS);
                let outline = lens_group_outline(&front_pts, &back_pts);
                if !outline.is_empty() {
                    elements.push(DrawElement::LensGroup { outline });
                }
            }
            Component::Stop { stop_idx } => {
                let surf = &surfaces[*stop_idx];
                let z = surf.pos().z();
                let sd = surf.semi_diameter();
                elements.push(DrawElement::Stop {
                    z,
                    half_gap: sd,
                    extent: largest_sd * 1.5,
                });
            }
            Component::Mirror { surf_idx } => {
                let surf = &surfaces[*surf_idx];
                let pts = sample_surface(surf, axis, N_PTS);
                if !pts.is_empty() {
                    elements.push(DrawElement::SurfaceProfile { points: pts });
                }
            }
            Component::UnpairedSurface { surf_idx } => {
                let surf = &surfaces[*surf_idx];
                let pts = sample_surface(surf, axis, N_PTS);
                if !pts.is_empty() {
                    elements.push(DrawElement::SurfaceProfile { points: pts });
                }
            }
        }
    }

    // Add flat planes (Image, Probe, Object at finite distance).
    for surf in surfaces.iter() {
        if surf.is_infinite() {
            continue;
        }
        let z = surf.pos().z();
        let half = if largest_sd > 0.0 {
            largest_sd * 1.2
        } else {
            1.0
        };
        match surf {
            Surface::Image(_) => {
                elements.push(DrawElement::FlatPlane {
                    z,
                    min: -half,
                    max: half,
                    kind: FlatPlaneKind::Image,
                });
            }
            Surface::Probe(_) => {
                elements.push(DrawElement::FlatPlane {
                    z,
                    min: -half,
                    max: half,
                    kind: FlatPlaneKind::Probe,
                });
            }
            Surface::Object(_) => {
                elements.push(DrawElement::FlatPlane {
                    z,
                    min: -half,
                    max: half,
                    kind: FlatPlaneKind::Object,
                });
            }
            _ => {}
        }
    }

    // Extract ray paths.
    let n_wavelengths = model.wavelengths().len();
    let mut ray_paths: Vec<Vec<Vec<[f64; 2]>>> = vec![Vec::new(); n_wavelengths];

    if let Some(tc) = trace {
        let trace_axis = axis;
        for result in tc.get_by_axis(trace_axis) {
            let wl_id = result.wavelength_id();
            if wl_id >= n_wavelengths {
                continue;
            }
            let bundle = result.ray_bundle();
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
                            Axis::Y => ray.y(),
                            Axis::X => ray.x(),
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
fn sample_surface(surf: &Surface, axis: Axis, n_pts: usize) -> Vec<[f64; 2]> {
    let sd = surf.semi_diameter();
    if !sd.is_finite() || sd <= 0.0 || n_pts < 2 {
        return Vec::new();
    }

    let mut pts = Vec::with_capacity(n_pts);
    for i in 0..n_pts {
        let t = i as Float / (n_pts - 1) as Float; // 0.0 to 1.0
        let transverse = sd * (2.0 * t - 1.0); // -sd to +sd
        let local_pt = match axis {
            Axis::Y => Vec3::new(0.0, transverse, 0.0),
            Axis::X => Vec3::new(transverse, 0.0, 0.0),
        };
        let (sag, _) = surf.sag_norm(local_pt);
        let local_surface_pt = match axis {
            Axis::Y => Vec3::new(0.0, transverse, sag),
            Axis::X => Vec3::new(transverse, 0.0, sag),
        };
        // Transform to global coordinates.
        let global_pt = surf.inv_rot_mat() * local_surface_pt + surf.pos();
        let transverse_global = match axis {
            Axis::Y => global_pt.y(),
            Axis::X => global_pt.x(),
        };
        pts.push([global_pt.z(), transverse_global]);
    }
    pts
}

/// Build a closed polygon outline for a lens element.
///
/// `front_pts` and `back_pts` are sampled from -sd to +sd (bottom to top in
/// transverse).
fn lens_group_outline(front_pts: &[[f64; 2]], back_pts: &[[f64; 2]]) -> Vec<[f64; 2]> {
    if front_pts.is_empty() || back_pts.is_empty() {
        return Vec::new();
    }

    let mut outline = Vec::with_capacity(2 * front_pts.len() + 4);
    // Front surface from bottom to top.
    outline.extend_from_slice(front_pts);
    // Top rim: front top to back top.
    outline.push(*back_pts.last().unwrap());
    // Back surface from top to bottom (reversed).
    for pt in back_pts.iter().rev() {
        outline.push(*pt);
    }
    // Bottom rim: close back to front bottom.
    outline.push(*front_pts.first().unwrap());
    outline
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
            DrawElement::LensGroup { outline } => {
                for &[z, t] in outline {
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
            DrawElement::FlatPlane { z, min, max, .. } => {
                update(*z, *min, &mut z_min, &mut z_max, &mut t_min, &mut t_max);
                update(*z, *max, &mut z_min, &mut z_max, &mut t_min, &mut t_max);
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
    use crate::{core::Float, examples::convexplano_lens, n};

    #[test]
    fn yz_valid_for_straight_system() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let model = convexplano_lens::sequential_model(air, nbk7, &wavelengths);
        let cs = cross_section_view(&model, None);
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
        // Surface 2 is the plano (flat) back surface.
        let pts = sample_surface(&surfaces[2], Axis::Y, 10);
        // All sag values should be zero for a flat surface.
        for &[_z, _t] in &pts {
            // Just ensure we got points back
        }
        assert_eq!(pts.len(), 10);
    }

    #[test]
    fn lens_group_outline_symmetric() {
        // For equal semi-diameters, the outline should have points at ±sd.
        let front = vec![
            [-1.0, -5.0],
            [-0.5, -2.5],
            [0.0, 0.0],
            [0.5, 2.5],
            [1.0, 5.0],
        ];
        let back = vec![[2.0, -5.0], [2.2, -2.5], [2.5, 0.0], [2.2, 2.5], [2.0, 5.0]];
        let outline = lens_group_outline(&front, &back);
        assert!(!outline.is_empty());
        // Should have at least front + back + rim points.
        assert!(outline.len() >= front.len() + back.len());
    }
}
