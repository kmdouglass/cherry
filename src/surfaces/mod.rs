mod conics;

use crate::vec3::Vec3;

type RefractiveIndex = fn(f32) -> (f32, f32);

/// A surface with routines to compute its sag and normal vectors.
trait SagNorm {
    fn sag_norm(self, pos: Vec3) -> (f32, Vec3);
}

/// A refracting surface with circular cross-section.
struct RefrCircSurf<S: SagNorm> {

    // Position of the center of the lens relative to the global reference frame.
    pos: Vec3,

    // Euler angles of the optics axis through the lens relative to the global reference frame.
    dir: Vec3,
    radius: f32,
    n: RefractiveIndex,
    sag_norm: S,
}

impl<S: SagNorm> RefrCircSurf<S> {
    fn new(pos: Vec3, dir: Vec3, radius: f32, n: RefractiveIndex, sag_norm: dyn SagNorm) -> Self {
        Self {
            pos,
            dir,
            radius,
            n,
            sag_norm,
        }
    }
}

enum Surface<S: SagNorm> {
    RefrCircSurf(RefrCircSurf<S>),
}

impl<S: SagNorm> Surface<S> {
    fn new_refr_circ_surf(
        pos: f32,
        radius: f32,
        n: f32,
    ) -> Self {
        let pos = Vec3::new(0.0, 0.0, pos);
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let n = |_| (n, 0.0);
        Self::RefrCircSurf(RefrCircSurf::new(pos, dir, radius, n, sag_norm))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refracting_surface() {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let radius = 1.0;
        let n = |z| (1.0, 1.0);
        let sag_norm = |pos| (0.0, Vec3::new(0.0, 0.0, 1.0));
        let surface = RefrCircSurf::new(pos, dir, radius, n, sag_norm);
        assert_eq!(surface.pos, pos);
        assert_eq!(surface.dir, dir);
    }
}
