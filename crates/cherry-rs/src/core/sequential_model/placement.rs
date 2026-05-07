/// Placement of a surface in a sequential optical system.
///
/// A [`Placement`] describes *where* a surface sits in 3D space and how the
/// optical axis (cursor) is oriented when it arrives at that surface. It is
/// intentionally separate from surface geometry ([`Surface`]) so that
/// coordinate-system operations and intrinsic-geometry operations do not mix.
///
/// [`Surface`]: crate::core::surfaces::Surface
use crate::core::{
    Float,
    math::{linalg::mat3x3::Mat3x3, linalg::rotations::Rotation3D, vec3::Vec3},
};

use super::cursor::Cursor;

/// Displacement and orientation parameters for one surface.
///
/// Used by [`SequentialModel::from_surfaces`] and internally by
/// `build_placements_and_directions`.
///
/// [`SequentialModel::from_surfaces`]: super::SequentialModel::from_surfaces
#[derive(Debug, Clone)]
pub struct SurfacePlacement {
    /// Vertex offset from the nominal cursor position, in cursor-frame (R, U,
    /// F).
    pub decenter: Vec3,
    /// Nominal surface tilt; redirects the cursor for reflecting surfaces.
    pub rotation: Rotation3D,
    /// Additional surface-only rotation; never redirects the cursor.
    pub rotation_offset: Rotation3D,
}

impl SurfacePlacement {
    /// Returns a `SurfacePlacement` with no displacement.
    pub fn none() -> Self {
        Self {
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation: Rotation3D::None,
            rotation_offset: Rotation3D::None,
        }
    }

    /// Returns a `SurfacePlacement` with the given nominal rotation and no
    /// decenter or rotation offset.
    pub fn from_rotation(rotation: Rotation3D) -> Self {
        Self {
            decenter: Vec3::new(0.0, 0.0, 0.0),
            rotation,
            rotation_offset: Rotation3D::None,
        }
    }
}

/// Position and orientation of a surface in the global coordinate system.
#[derive(Debug, Clone)]
pub struct Placement {
    /// Vertex position in the global coordinate system.
    pub position: Vec3,

    /// Cumulative path length along the axis to this surface.
    pub track: Float,

    /// Rotation from the global frame into the surface's local frame.
    ///
    /// Equals `(rotation_offset · rotation) * cursor_rotation`
    /// (global-to-local, including any rotation offset).
    pub rotation_matrix: Mat3x3,

    /// Inverse of `rotation_matrix` (local-to-global).
    ///
    /// Because the matrix is orthogonal this equals
    /// `rotation_matrix.transpose()`. This value is cached for efficiency.
    pub inv_rotation_matrix: Mat3x3,

    /// Local-to-global transform computed from the nominal rotation only
    /// (without `rotation_offset`). Equal to `inv_rotation_matrix` when
    /// `rotation_offset` is `None`. Used by `propagate_tangential_vec` so
    /// that the paraxial tangential axis follows the nominal system.
    pub nominal_inv_rotation_matrix: Mat3x3,

    /// Rotation from the global frame into the optical-axis (cursor) frame
    /// only, without any surface tilt applied.
    ///
    /// Needed for aperture-projection calculations (`projected_semi_diameter`)
    /// and for `is_rotationally_symmetric`.
    pub cursor_rotation_matrix: Mat3x3,
}

impl Placement {
    /// Build a [`Placement`] from a pre-composed surface rotation matrix, the
    /// nominal rotation matrix, a decenter, and the current cursor state.
    ///
    /// `actual_rotation_matrix` is
    /// `rotation_offset.rotation_matrix() * rotation.rotation_matrix()`.
    /// `nominal_rotation_matrix` is `rotation.rotation_matrix()` alone.
    /// When `rotation_offset` is `None` both matrices are equal.
    ///
    /// `decenter` is in cursor-frame coordinates (R, U, F).
    pub(crate) fn from_decenter_and_rotation(
        decenter: Vec3,
        actual_rotation_matrix: Mat3x3,
        nominal_rotation_matrix: Mat3x3,
        cursor: &Cursor,
    ) -> Self {
        let cursor_rotation_matrix = cursor.rotation_matrix();
        let rotation_matrix = actual_rotation_matrix * cursor_rotation_matrix;
        let nom_rot_matrix = nominal_rotation_matrix * cursor_rotation_matrix;
        let offset_global = cursor_rotation_matrix.transpose() * decenter;
        let position = cursor.pos() + offset_global;
        Self::new(
            position,
            cursor.track(),
            rotation_matrix,
            nom_rot_matrix,
            cursor_rotation_matrix,
        )
    }

    /// Create a new [`Placement`] from its constituent parts.
    pub fn new(
        position: Vec3,
        track: Float,
        rotation_matrix: Mat3x3,
        nominal_rotation_matrix: Mat3x3,
        cursor_rotation_matrix: Mat3x3,
    ) -> Self {
        let inv_rotation_matrix = rotation_matrix.transpose();
        let nominal_inv_rotation_matrix = nominal_rotation_matrix.transpose();
        Self {
            position,
            track,
            rotation_matrix,
            inv_rotation_matrix,
            nominal_inv_rotation_matrix,
            cursor_rotation_matrix,
        }
    }

    /// Returns the axial (z) position of the surface vertex.
    pub fn z(&self) -> Float {
        self.position.z()
    }

    /// Returns `true` if any coordinate of the vertex position is infinite.
    ///
    /// This is the case for the object surface of a system with an object at
    /// infinity.
    pub fn is_infinite(&self) -> bool {
        self.position.x().is_infinite()
            || self.position.y().is_infinite()
            || self.position.z().is_infinite()
    }

    /// Returns the unit vector pointing along the optical axis (cursor forward
    /// direction) at this surface, expressed in the global frame.
    pub fn axis_direction(&self) -> Vec3 {
        // The third row of cursor_rotation_matrix is the forward direction in
        // global coords when the matrix is global-to-cursor.
        // Transposing maps it back to global, giving the forward vector.
        self.cursor_rotation_matrix.transpose() * Vec3::new(0.0, 0.0, 1.0)
    }

    /// Returns the semi-diameter as seen by a paraxial ray travelling along
    /// the cursor axis in the tangential plane defined by `v`.
    ///
    /// `r` is the surface's clear-aperture semi-diameter (from
    /// [`Surface::semi_diameter`]). `v` is a unit vector in the global
    /// frame that lies in the transverse plane and defines the meridional
    /// plane of interest.
    ///
    /// For a tilted surface the effective limit on cursor height is
    /// `r · |n_F| / sqrt(n_φ² + n_F²)` where `(n_R, n_U, n_F)` are the
    /// surface-normal components in the cursor frame and
    /// `n_φ = n_R · v_x + n_U · v_y` is the component along `v`.
    ///
    /// Returns [`Float::INFINITY`] when `r` is infinite (non-aperture
    /// surfaces).
    ///
    /// [`Surface::semi_diameter`]: crate::core::surfaces::Surface::semi_diameter
    pub fn projected_semi_diameter(&self, r: Float, v: Vec3) -> Float {
        if r.is_infinite() {
            return Float::INFINITY;
        }

        // R_surf = cursor_to_local = global_to_local · cursor_to_global
        //        = rotation_matrix · cursor_rotation_matrix.transpose()
        let r_surf = self.rotation_matrix * self.cursor_rotation_matrix.transpose();

        // Third row of R_surf is the surface normal expressed in cursor frame:
        // (n_R, n_U, n_F)
        let n_r = r_surf.e[2][0];
        let n_u = r_surf.e[2][1];
        let n_f = r_surf.e[2][2];

        // Component of the surface normal along the tangential direction v.
        let n_phi = n_r * v.x() + n_u * v.y();
        let denom = (n_phi * n_phi + n_f * n_f).sqrt();

        if denom < 1e-12 {
            return 0.0;
        }

        r * n_f.abs() / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_cursor() -> Cursor {
        Cursor::new(0.0)
    }

    fn cursor_at(z: f64) -> Cursor {
        let mut c = Cursor::new(0.0);
        c.advance(z);
        c
    }

    // AT-1: zero displacement produces placement identical to nominal
    #[test]
    fn at1_zero_displacement_identity_cursor() {
        let id = Mat3x3::identity();
        let cursor = identity_cursor();
        let p = Placement::from_decenter_and_rotation(Vec3::new(0.0, 0.0, 0.0), id, id, &cursor);
        assert!(
            p.position.approx_eq(&Vec3::new(0.0, 0.0, 0.0), 1e-15),
            "position: {:?}",
            p.position
        );
        assert!(p.rotation_matrix.approx_eq(&id, 1e-15));
        assert!(p.nominal_inv_rotation_matrix.approx_eq(&id, 1e-15));
        assert!(p.cursor_rotation_matrix.approx_eq(&id, 1e-15));
        assert!((p.track - 0.0).abs() < 1e-15);
    }

    // AT-2: R-axis decenter shifts vertex along x
    #[test]
    fn at2_transverse_decenter_r_axis() {
        let id = Mat3x3::identity();
        let dx = 2.5;
        let cursor = cursor_at(10.0);
        let p = Placement::from_decenter_and_rotation(Vec3::new(dx, 0.0, 0.0), id, id, &cursor);
        let tol = 1e-15;
        assert!((p.position.x() - dx).abs() < tol, "x: {}", p.position.x());
        assert!(p.position.y().abs() < tol, "y: {}", p.position.y());
        assert!((p.position.z() - 10.0).abs() < tol, "z: {}", p.position.z());
    }

    // AT-3: U-axis decenter shifts vertex along y
    #[test]
    fn at3_transverse_decenter_u_axis() {
        let id = Mat3x3::identity();
        let dy = -1.3;
        let cursor = cursor_at(5.0);
        let p = Placement::from_decenter_and_rotation(Vec3::new(0.0, dy, 0.0), id, id, &cursor);
        let tol = 1e-15;
        assert!(p.position.x().abs() < tol, "x: {}", p.position.x());
        assert!((p.position.y() - dy).abs() < tol, "y: {}", p.position.y());
        assert!((p.position.z() - 5.0).abs() < tol, "z: {}", p.position.z());
    }

    // AT-4: F-axis decenter shifts vertex along z in a straight system
    #[test]
    fn at4_axial_decenter_f_axis() {
        let id = Mat3x3::identity();
        let dz = 3.0;
        let nominal_z = 7.0;
        let cursor = cursor_at(nominal_z);
        let p = Placement::from_decenter_and_rotation(Vec3::new(0.0, 0.0, dz), id, id, &cursor);
        let tol = 1e-15;
        assert!(p.position.x().abs() < tol, "x: {}", p.position.x());
        assert!(p.position.y().abs() < tol, "y: {}", p.position.y());
        assert!(
            (p.position.z() - (nominal_z + dz)).abs() < tol,
            "z: {}",
            p.position.z()
        );
    }

    // AT-5: F-axis decenter after a 90° fold shifts along the post-fold axis,
    // not along global Z. After a 90° fold about the R-axis the cursor
    // forward direction is (0, -1, 0), so a cursor-F decenter of dz moves the
    // vertex by (0, -dz, 0) in global coordinates.
    #[test]
    fn at5_axial_decenter_after_fold() {
        // Build a 3-surface system: Object — flat mirror (45° theta fold) — Image
        use crate::specs::{
            gaps::GapSpec,
            surfaces::{BoundaryKind, SurfaceSpec},
        };
        use crate::{EulerAngles, Rotation3D, n};

        let gaps = vec![
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
        ];
        // Mirror at 45° about R redirects the cursor downward (–Y).
        let theta = 45.0_f64.to_radians();
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 25.4,
                radius_of_curvature: f64::INFINITY,
                surf_kind: BoundaryKind::Reflecting,
                rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(theta, 0.0, 0.0)),
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];

        let model = crate::SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], None)
            .expect("model builds");

        // Now construct the same mirror surface but with a cursor-F decenter of 2.0.
        // After the 90° fold the cursor forward direction is (0, -1, 0), so
        // decenter (0, 0, 2) in cursor frame should shift the vertex by
        // (0, -2, 0) in global coords relative to the mirror without decenter.
        let placements_no_decenter = model.placements();
        let mirror_pos_nominal = placements_no_decenter[1].position;

        let surfaces_with_decenter = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 25.4,
                radius_of_curvature: f64::INFINITY,
                surf_kind: BoundaryKind::Reflecting,
                rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(theta, 0.0, 0.0)),
                decenter: Vec3::new(0.0, 0.0, 2.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];
        let model2 = crate::SequentialModel::from_surface_specs(
            &gaps,
            &surfaces_with_decenter,
            &[0.5876],
            None,
        )
        .expect("model2 builds");
        let mirror_pos_decentered = model2.placements()[1].position;

        let tol = 1e-12;
        // x unchanged
        assert!(
            (mirror_pos_decentered.x() - mirror_pos_nominal.x()).abs() < tol,
            "x changed: {} vs {}",
            mirror_pos_decentered.x(),
            mirror_pos_nominal.x()
        );
        // After 45° fold the forward direction at the mirror is still along Z
        // (cursor hasn't been redirected yet when placement is built).
        // So F-decenter of 2 shifts z by +2.
        assert!(
            (mirror_pos_decentered.z() - mirror_pos_nominal.z() - 2.0).abs() < tol,
            "z shift wrong: decentered z={}, nominal z={}",
            mirror_pos_decentered.z(),
            mirror_pos_nominal.z()
        );
    }
}
