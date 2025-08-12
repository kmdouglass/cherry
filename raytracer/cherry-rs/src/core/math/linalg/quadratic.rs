/// Quadratic (order 2) polynomials.
use anyhow::Result;

use crate::core::{Float, math::constants::REL_TOL};

/// A quadratic polynomial where the leading coefficient is 1.
pub struct NormalizedQuadratic {
    b: Float,
    c: Float,
}

impl NormalizedQuadratic {
    pub fn new(b: Float, c: Float) -> Self {
        Self { b, c }
    }

    /// Returns the roots of the quadratic polynomial ax^2 + bx + c = 0. The
    /// smaller root is always returned first.
    ///
    /// The following cases are not addressed because they are not necessary for
    /// this library:
    /// - Complex roots
    /// - a == 0: This cannot be the case as we check for it in the new method.
    pub fn roots(&self) -> Result<(Float, Float)> {
        let discriminant = self.b * self.b - 4.0 * self.c;
        if discriminant < -REL_TOL * self.b * self.b {
            return Err(anyhow::anyhow!(
                "No real roots exist for the quadratic polynomial."
            ));
        }

        // Check for repeated roots
        if discriminant.abs() < REL_TOL * self.b * self.b {
            let root = -self.b / 2.0;
            return Ok((root, root));
        }

        // This works so long as signum never returns zero, which is the case for Rust's
        // f32 and f64 types. https://doc.rust-lang.org/std/primitive.f64.html#method.signum
        let u = -self.b - self.b.signum() * discriminant.sqrt();

        let root1 = u / 2.0;
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
    fn normalized_quadratic_roots() {
        let poly = NormalizedQuadratic::new(-3.0, 2.0);
        let (root1, root2) = poly.roots().unwrap();
        assert!((root1 - 1.0).abs() < REL_TOL);
        assert!((root2 - 2.0).abs() < REL_TOL);
    }

    #[test]
    fn normalized_quadratic_no_real_roots() {
        let poly = NormalizedQuadratic::new(0.0, 1.0);
        assert!(poly.roots().is_err());
    }

    #[test]
    fn normalized_quadratic_repeated_roots() {
        let poly = NormalizedQuadratic::new(-2.0, 1.0);
        let (root1, root2) = poly.roots().unwrap();
        assert!((root1 - 1.0).abs() < REL_TOL);
        assert!((root2 - 1.0).abs() < REL_TOL);
    }
}
