use crate::vec3::Vec3;

use super::SagNorm;

struct Conic {
    roc: f32,
    k: f32,
}

impl SagNorm for Conic {
    fn sag_norm(self, pos: Vec3) -> (f32, Vec3) {
        let r = pos.x().powi(2) + pos.y().powi(2);
        let x = r.powi(2) / self.roc;
        let sag = x / (1.0 + (1.0 - (1.0 + self.k) * x / self.roc).sqrt());
        
        let norm = Vec3::new(pos.x(), pos.y(), sag).normalize();
        (sag, norm)
    }
}