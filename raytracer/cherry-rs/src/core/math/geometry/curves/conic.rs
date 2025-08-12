/// A conic section.
use crate::core::{Float, math::linalg::mat2::Mat2, math::linalg::mat3::Mat3};

const TOL: Float = 1e-12;

#[derive(Debug, PartialEq, Eq)]
pub enum ConicType {
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
    matrix: Mat3,
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
        let matrix = Mat3::new(
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
        Conic { matrix }
    }

    /// Determines the class of the conic section.
    pub fn classify(&self) -> ConicType {
        let matrix_quadratic_form = self.matrix_quadratic_form();

        let det_full = self.matrix.determinant();
        let det_quad_form = matrix_quadratic_form.determinant();
        let trace_quad_form = matrix_quadratic_form.trace();

        if det_full.abs() < TOL {
            return ConicType::Degenerate;
        } else if det_quad_form.abs() < TOL {
            return ConicType::Parabola;
        } else if trace_quad_form > 0.0 {
            if det_full * trace_quad_form < 0.0 {
                return ConicType::Ellipse;
            } else {
                return ConicType::Empty;
            }
        } else {
            return ConicType::Hyperbola;
        }
    }

    /// Returns the matrix representing the conic section.
    pub fn matrix(&self) -> &Mat3 {
        &self.matrix
    }

    /// Returns the matrix of the quadratic form.
    pub fn matrix_quadratic_form(&self) -> Mat2 {
        Mat2::new(
            self.matrix.e[0][0],
            self.matrix.e[0][1] * 2.0,
            self.matrix.e[1][0] * 2.0,
            self.matrix.e[1][1],
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
        let q_matrix = conic.matrix_quadratic_form();
        assert_eq!(q_matrix.e[0][0], 1.0);
        assert_eq!(q_matrix.e[0][1], 2.0);
        assert_eq!(q_matrix.e[1][0], 2.0);
        assert_eq!(q_matrix.e[1][1], 1.0);
    }

    #[test]
    fn conic_classify() {
        let conic = Conic::new(1.0, 0.0, 1.0, 0.0, 0.0, -1.0);
        assert_eq!(conic.classify(), ConicType::Ellipse);

        let conic = Conic::new(1.0, 0.0, 1.0, 0.0, 0.0, 10.0);
        assert_eq!(conic.classify(), ConicType::Empty);

        let conic = Conic::new(1.0, 0.0, 0.0, 0.0, 5.0, -1.0);
        assert_eq!(conic.classify(), ConicType::Parabola);

        let conic = Conic::new(1.0, 2.0, -1.0, 4.0, -5.0, 1.0);
        assert_eq!(conic.classify(), ConicType::Hyperbola);

        let conic = Conic::new(1.0, 0.0, -1.0, 0.0, 0.0, 0.0);
        assert_eq!(conic.classify(), ConicType::Degenerate);
    }
}
