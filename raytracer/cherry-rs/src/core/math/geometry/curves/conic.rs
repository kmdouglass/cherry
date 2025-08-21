/// A conic section curve.
use anyhow::Result;

use crate::core::{
    Float, PI,
    math::{
        constants::ZERO_TOL,
        linalg::{mat2x2::Mat2x2, mat3x3::Mat3x3},
        vec2::Vec2,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConicClass {
    Degenerate,
    Ellipse,

    /// A conic with no real points.
    Empty,
    Hyperbola,
    Parabola,
}

/// A conic section in R2.
///
/// The conic section is a curve represented by a quadratic equation in the
/// form:
///
/// ```text
/// Q(x, y) = Ax^2 + Bxy + Cy^2 + Dx + Ey + F = 0
/// ```
///
/// Internally, the conic is represented by a 3x3 matrix in homogeneous
/// coordinates:
///
/// ```text
/// A_Q = |   A  B/2 D/2 |
///       | B/2    C E/2 |
///       | D/2  E/2   F |
/// ```
///
/// The matrix of the quadratic form of the conic, A33, is defined as:
///
/// ```text
/// A_33 = |   A B/2 |
///        | B/2   C |
/// ```
#[derive(Debug)]
pub struct Conic {
    /// The matrix representing the conic section.
    matrix: Mat3x3,
    conic_class: ConicClass,
}

impl ConicClass {
    fn is_central_conic(&self) -> bool {
        matches!(self, ConicClass::Ellipse | ConicClass::Hyperbola)
    }
}

impl Conic {
    /// Constructs a new `Conic` from the coefficients of the quadratic
    /// equation.
    ///
    /// # Arguments
    /// * `a` - Coefficient for x^2
    /// * `b` - Coefficient for xy
    /// * `c` - Coefficient for y^2
    /// * `d` - Coefficient for x
    /// * `e` - Coefficient for y
    /// * `f` - Constant term
    pub fn new(a: Float, b: Float, c: Float, d: Float, e: Float, f: Float) -> Self {
        let matrix = Mat3x3::new(
            a,
            b / 2.0,
            d / 2.0,
            b / 2.0,
            c,
            e / 2.0,
            d / 2.0,
            e / 2.0,
            f,
        );

        let conic_class = Conic::classify(&matrix);

        Conic {
            matrix,
            conic_class,
        }
    }

    /// Returns the coordinates for the conic's center.
    ///
    /// If the conic is not a central conic, this method returns `None`.
    ///
    /// The center is calculated using the formula:
    /// ```text
    /// x_c = (b * e - 2 * c * d) / (4 * a * c - b^2)
    /// y_c = (b * d - 2 * a * e) / (4 * a * c - b^2)
    /// ```
    pub fn center(&self) -> Option<Vec2> {
        if self.conic_class.is_central_conic() {
            let a = self.matrix.e[0][0];
            let b = self.matrix.e[0][1] * 2.0;
            let c = self.matrix.e[1][1];
            let d = self.matrix.e[0][2] * 2.0;
            let e = self.matrix.e[1][2] * 2.0;
            let f = self.matrix.e[2][2];

            let x_c = (b * e - 2.0 * c * d) / (4.0 * a * c - b * b);
            let y_c = (b * d - 2.0 * a * e) / (4.0 * a * c - b * b);

            Some(Vec2 { x: x_c, y: y_c })
        } else {
            None
        }
    }

    /// Returns the type of the conic section.
    pub fn class(&self) -> ConicClass {
        self.conic_class
    }

    /// Returns the matrix representing the conic section.
    pub fn matrix(&self) -> &Mat3x3 {
        &self.matrix
    }

    /// Returns samples from the conic curve.
    ///
    /// The number of samples returned is at most `num_samples`.
    pub fn sample(&self, num_samples: usize) -> Result<Vec<Vec2>> {
        let mut samples = Vec::with_capacity(num_samples);
        let step = 2.0 * PI / num_samples as f64;

        match self.conic_class {
            ConicClass::Ellipse => {
                // A ConicClass::Ellipse is real by defintition, so no need to check for
                // degeneracy.
                let a_33 = Conic::matrix_quadratic_form(&self.matrix);
                let center = self
                    .center()
                    .ok_or_else(|| anyhow::anyhow!("Ellipse center is not defined"))?;

                let (eigenvalue0, eigenvalue1, eigenvectors) = a_33.eig()?;

                // Coefficients of the ellipse in its standard form, x^2/a^2 + y^2/b^2 = 1.
                let k = -&self.matrix.determinant() / a_33.determinant();
                let a = (k / eigenvalue0).sqrt();
                let b = (k / eigenvalue1).sqrt();

                let mut curr_sample: Vec2 = Vec2 { x: 0.0, y: 0.0 };
                for i in 0..num_samples {
                    let theta = i as Float * step;

                    // Parametric equations for the ellipse.
                    curr_sample.x = a * theta.cos();
                    curr_sample.y = b * theta.sin();

                    // Transform into the coordinate system of the non-standard form ellipse.
                    // Eigenvectors are the rows of the matrix, so no need to transpose.
                    curr_sample = eigenvectors * curr_sample + center;
                    samples.push(curr_sample);
                }

                return Ok(samples);
            }
            ConicClass::Degenerate | ConicClass::Empty => {
                // No samples for empty conic.
                return Ok(samples);
            }
            _ => {
                panic!("Sampling not implemented for this conic class");
            }
        }

        Ok(samples)
    }

    /// Determines the class of the conic section.
    fn classify(matrix: &Mat3x3) -> ConicClass {
        let matrix_quadratic_form = Conic::matrix_quadratic_form(matrix);

        let det_full = matrix.determinant();
        let det_quad_form = matrix_quadratic_form.determinant();
        let trace_quad_form = matrix_quadratic_form.trace();

        if det_full.abs() < ZERO_TOL {
            return ConicClass::Degenerate;
        } else if det_quad_form.abs() < ZERO_TOL {
            return ConicClass::Parabola;
        } else if trace_quad_form > 0.0 {
            if det_full * trace_quad_form < 0.0 {
                return ConicClass::Ellipse;
            } else {
                return ConicClass::Empty;
            }
        } else {
            return ConicClass::Hyperbola;
        }
    }

    /// Returns the matrix of the quadratic form.
    fn matrix_quadratic_form(matrix: &Mat3x3) -> Mat2x2 {
        Mat2x2::new(
            matrix.e[0][0],
            matrix.e[0][1] * 2.0,
            matrix.e[1][0] * 2.0,
            matrix.e[1][1],
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conic_new() {
        let conic = Conic::new(1.0, 0.0, 1.0, 0.0, 0.0, -1.0);
        assert_eq!(conic.matrix().e[0][0], 1.0);
        assert_eq!(conic.matrix().e[1][1], 1.0);
        assert_eq!(conic.matrix().e[2][2], -1.0);
    }

    #[test]
    fn conic_matrix_quadratic_form() {
        let conic = Conic::new(1.0, 2.0, 1.0, -4.0, 5.0, -1.0);
        let q_matrix = Conic::matrix_quadratic_form(conic.matrix());
        assert_eq!(q_matrix[0][0], 1.0);
        assert_eq!(q_matrix[0][1], 2.0);
        assert_eq!(q_matrix[1][0], 2.0);
        assert_eq!(q_matrix[1][1], 1.0);
    }

    #[test]
    fn conic_classify() {
        let conic = Conic::new(1.0, 0.0, 1.0, 0.0, 0.0, -1.0);
        assert_eq!(conic.class(), ConicClass::Ellipse);

        let conic = Conic::new(1.0, 0.0, 1.0, 0.0, 0.0, 10.0);
        assert_eq!(conic.class(), ConicClass::Empty);

        let conic = Conic::new(1.0, 0.0, 0.0, 0.0, 5.0, -1.0);
        assert_eq!(conic.class(), ConicClass::Parabola);

        let conic = Conic::new(1.0, 2.0, -1.0, 4.0, -5.0, 1.0);
        assert_eq!(conic.class(), ConicClass::Hyperbola);

        let conic = Conic::new(1.0, 0.0, -1.0, 0.0, 0.0, 0.0);
        assert_eq!(conic.class(), ConicClass::Degenerate);
    }
}
