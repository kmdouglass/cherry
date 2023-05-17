mod conics;

use crate::vec3::Vec3;

enum Surface {
    RefractingCircularConic(conics::RefractingCircularConic),
}

impl Surface {
    fn new_refr_circ_conic(axial_pos: f32, diam: f32, n: f32, roc: f32, k: f32) -> Self {
        let pos = Vec3::new(0.0, 0.0, axial_pos);
        let dir = Vec3::new(0.0, 0.0, 1.0);

        Self::RefractingCircularConic(conics::RefractingCircularConic::new(
            pos, dir, diam, n, roc, k,
        ))
    }
}
