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
    math::{linalg::mat3x3::Mat3x3, vec3::Vec3},
};

use crate::core::math::geometry::reference_frames::Cursor;
use crate::specs::surfaces::SurfaceSpec;

/// Position and orientation of a surface in the global coordinate system.
#[derive(Debug, Clone)]
pub struct Placement {
    /// Vertex position in the global coordinate system.
    pub position: Vec3,

    /// Cumulative path length along the axis to this surface.
    pub track: Float,

    /// Rotation from the global frame into the surface's local frame.
    ///
    /// Equals `surface_tilt_rotation × cursor_rotation` (global-to-local).
    pub rotation_matrix: Mat3x3,

    /// Inverse of `rotation_matrix` (local-to-global).
    ///
    /// Because the matrix is orthogonal this equals
    /// `rotation_matrix.transpose()`.
    pub inv_rotation_matrix: Mat3x3,

    /// Rotation from the global frame into the optical-axis (cursor) frame
    /// only, without any surface tilt applied.
    ///
    /// Needed for aperture-projection calculations (`projected_semi_diameter`)
    /// and for `is_rotationally_symmetric`.
    pub cursor_rotation_matrix: Mat3x3,
}

impl Placement {
    /// Build a [`Placement`] from a surface spec and the current cursor state.
    ///
    /// The cursor holds the current position and optical-axis orientation.
    /// The spec may carry an additional surface-tilt rotation that is composed
    /// on top of the cursor orientation.
    pub(crate) fn from_spec(spec: &SurfaceSpec, cursor: &Cursor) -> Self {
        let cursor_rotation_matrix = cursor.rotation_matrix();
        let rotation_matrix = match spec {
            SurfaceSpec::Conic { rotation, .. }
            | SurfaceSpec::Image { rotation }
            | SurfaceSpec::Probe { rotation }
            | SurfaceSpec::Stop { rotation, .. } => {
                rotation.rotation_matrix() * cursor_rotation_matrix
            }
            SurfaceSpec::Object => cursor_rotation_matrix,
        };
        Self::new(
            cursor.pos(),
            cursor.track(),
            rotation_matrix,
            cursor_rotation_matrix,
        )
    }

    /// Create a new [`Placement`] from its constituent parts.
    pub fn new(
        position: Vec3,
        track: Float,
        rotation_matrix: Mat3x3,
        cursor_rotation_matrix: Mat3x3,
    ) -> Self {
        let inv_rotation_matrix = rotation_matrix.transpose();
        Self {
            position,
            track,
            rotation_matrix,
            inv_rotation_matrix,
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
