/// A quadric surface in R3.
use crate::core::Float;

/// A quadric surface in R3.
///
/// The quadric surface is defined by the equation:
///
/// ```text
/// Q(x, y, z) = Ax^2 + By^2 + Cz^2 + Dxy + Exz + Fyz + Gx + Hy + Iz + J = 0
/// ```
#[derive(Debug)]
pub struct Quadric {
    /// Coefficients of the quadric equation.
    a: Float,
    b: Float,
    c: Float,
    d: Float,
    e: Float,
    f: Float,
    g: Float,
    h: Float,
    i: Float,
    j: Float,
}

impl Quadric {
    pub fn new(
        a: Float,
        b: Float,
        c: Float,
        d: Float,
        e: Float,
        f: Float,
        g: Float,
        h: Float,
        i: Float,
        j: Float,
    ) -> Self {
        Quadric {
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            h,
            i,
            j,
        }
    }

    pub fn a(&self) -> Float {
        self.a
    }
    pub fn b(&self) -> Float {
        self.b
    }
    pub fn c(&self) -> Float {
        self.c
    }
    pub fn d(&self) -> Float {
        self.d
    }
    pub fn e(&self) -> Float {
        self.e
    }
    pub fn f(&self) -> Float {
        self.f
    }
    pub fn g(&self) -> Float {
        self.g
    }
    pub fn h(&self) -> Float {
        self.h
    }
    pub fn i(&self) -> Float {
        self.i
    }
    pub fn j(&self) -> Float {
        self.j
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quadric_new() {
        let q = Quadric::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0);
        assert_eq!(q.a(), 1.0);
        assert_eq!(q.b(), 2.0);
        assert_eq!(q.c(), 3.0);
        assert_eq!(q.d(), 4.0);
        assert_eq!(q.e(), 5.0);
        assert_eq!(q.f(), 6.0);
        assert_eq!(q.g(), 7.0);
        assert_eq!(q.h(), 8.0);
        assert_eq!(q.i(), 9.0);
        assert_eq!(q.j(), 10.0);
    }
}
