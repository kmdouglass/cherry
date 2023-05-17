use crate::vec3::Vec3;

type RefractiveIndex = fn(f32) -> (f32, f32);
type SagNorm = fn(Vec3) -> (f32, Vec3);

struct RefractingSurface {
    pos: Vec3,
    dir: Vec3,
    n: RefractiveIndex,
    sag_norm: SagNorm,
}

impl RefractingSurface {
    fn new(pos: Vec3, dir: Vec3, n: RefractiveIndex, sag_norm: SagNorm) -> Self {
        Self {
            pos,
            dir,
            n,
            sag_norm,
        }
    }
}

enum Surface {
    Refracting(RefractingSurface),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refracting_surface() {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let n = |z| (1.0, 1.0);
        let sag_norm = |pos| (0.0, Vec3::new(0.0, 0.0, 1.0));
        let surface = RefractingSurface::new(pos, dir, n, sag_norm);
        assert_eq!(surface.pos, pos);
        assert_eq!(surface.dir, dir);
    }
}
