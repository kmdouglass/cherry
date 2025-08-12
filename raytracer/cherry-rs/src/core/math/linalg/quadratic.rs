/// Quadratic (order 2) polynomials.
use anyhow::Result;

use crate::core::{
    Float,
    math::constants::{CHARACTERISTIC_LENS_SIZE_MM, GEOM_ZERO_TOL},
};

pub struct Quadratic {
    a: Float,
    b: Float,
    c: Float,
}

impl Quadratic {
    pub fn new(a: Float, b: Float, c: Float) -> Result<Self> {
        if a < GEOM_ZERO_TOL {
            return Err(anyhow::anyhow!(
                "Coefficient 'a' is too close to zero for a quadratic polynomial."
            ));
        }
        Ok(Self { a, b, c })
    }

    /// Returns the roots of the quadratic polynomial ax^2 + bx + c = 0. The
    /// smaller root is always returned first.
    ///
    /// We do not address the following cases:
    /// - Complex roots: This implementation only handles real roots.
    /// - a == 0: This cannot be the case as we check for it in the new method.
    pub fn roots(&self) -> Result<(Float, Float)> {
        let discriminant = self.b * self.b - 4.0 * self.a * self.c;
        if discriminant < 0.0 {
            return Err(anyhow::anyhow!(
                "No real roots exist for the given quadratic polynomial."
            ));
        }

        // Check for repeated roots
        if discriminant.abs()
            < GEOM_ZERO_TOL / CHARACTERISTIC_LENS_SIZE_MM / CHARACTERISTIC_LENS_SIZE_MM
        {
            let root = -self.b / (2.0 * self.a);
            return Ok((root, root));
        }

        // This works so long as signum never returns zero, which is the case for Rust's
        // f32 and f64 types. https://doc.rust-lang.org/std/primitive.f32.html#method.signum
        // https://doc.rust-lang.org/std/primitive.f64.html#method.signum
        let u = -self.b - self.b.signum() * discriminant.sqrt();

        let root1 = u / (2.0 * self.a);
        let root2 = 2.0 * self.c / u;

        if root1 > root2 {
            return Ok((root2, root1));
        }

        Ok((root1, root2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadratic_new_valid_inputs() {
        let poly = Quadratic::new(1.0, -3.0, 2.0);
        assert!(poly.is_ok());
    }

    #[test]
    fn quadratic_new_invalid_a() {
        let poly = Quadratic::new(0.0, 1.0, 1.0);
        assert!(poly.is_err());
    }

    #[test]
    fn quadratic_roots() {
        let poly = Quadratic::new(1.0, -3.0, 2.0).unwrap();
        let (root1, root2) = poly.roots().unwrap();
        assert!((root1 - 1.0).abs() < GEOM_ZERO_TOL);
        assert!((root2 - 2.0).abs() < GEOM_ZERO_TOL);
    }

    #[test]
    fn quadratic_no_real_roots() {
        let poly = Quadratic::new(1.0, 0.0, 1.0).unwrap();
        assert!(poly.roots().is_err());
    }

    #[test]
    fn quadratic_repeated_roots() {
        let poly = Quadratic::new(1.0, -2.0, 1.0).unwrap();
        let (root1, root2) = poly.roots().unwrap();
        assert!((root1 - 1.0).abs() < GEOM_ZERO_TOL);
        assert!((root2 - 1.0).abs() < GEOM_ZERO_TOL);
    }
}
