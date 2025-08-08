/// Quadratic (order 2) polynomials.

use anyhow::Result;

use crate::core::Float;

struct Quadratic {
    a: Float,
    b: Float,
    c: Float,
}

impl Quadratic {
    fn new(a: Float, b: Float, c: Float) -> Result<Self> {
        if a == 0.0 {
            return Err(anyhow::anyhow!("Coefficient 'a' cannot be zero for a quadratic polynomial."));
        }
        Ok(Self { a, b, c })
    }

    fn roots(&self) -> Result<(Float, Float)> {
        let discriminant = self.b * self.b - 4.0 * self.a * self.c;
        if discriminant < 0.0 {
            return Err(anyhow::anyhow!("No real roots exist for the given quadratic polynomial."));
        }

        !todo!("Drop down to linear solver if a = 0");

        // This works so long as signum never returns zero, which is the case for Rust's f32 and f64 types.
        // https://doc.rust-lang.org/std/primitive.f32.html#method.signum
        // https://doc.rust-lang.org/std/primitive.f64.html#method.signum 
        let u = -self.b * self.b.signum() * discriminant.sqrt();

        let root1 = u / (2.0 * self.a);
        let root2 = 2.0 * self.c / u;
        
        Ok((root1, root2))
    }

}
