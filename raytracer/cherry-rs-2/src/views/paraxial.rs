/// A paraxial view into an optical system.
use std::cell::OnceCell;

use ndarray::{arr2, s, Array, Array2, Array3};

use crate::{
    core::{
        argmin,
        sequential_model::{Axis, SequentialSubModel, Step, Surface},
        Float,
    },
    specs::surfaces::SurfaceType,
};

const DEFAULT_THICKNESS: Float = 0.0;

/// A 2 x Nr array of paraxial rays.
///
/// Nr is the number of rays. The first column is the height of the ray at the
/// surface, and the second column is the paraxial angle of the ray at the
/// surface.
type ParaxialRays = Array2<Float>;

/// A Ns x 2 x Nr array of paraxial ray trace results.
///
/// Ns is the number of surfaces, and Nr is the number of rays. The first
/// element of the 2nd dimension is the height of the ray at the surface, and
/// the second element is the angle of the ray at the surface.
type ParaxialRayTraceResults = Array3<Float>;

/// A 2 x 2 array representing a ray transfer matrix for paraxial rays.
type RayTransferMatrix = Array2<Float>;

struct ParaxialSubView {
    pseudo_marginal_ray: ParaxialRayTraceResults,
    reverse_parallel_ray: ParaxialRayTraceResults,

    aperture_stop: OnceCell<usize>,
}

impl ParaxialSubView {
    /// Create a new paraxial view of an optical system.
    pub fn new(
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
    ) -> Self {
        let pseudo_marginal_ray =
            Self::calc_pseudo_marginal_ray(sequential_sub_model, surfaces, axis);
        let reverse_parallel_ray =
            Self::calc_reverse_parallel_ray(sequential_sub_model, surfaces, axis);

        Self {
            pseudo_marginal_ray: pseudo_marginal_ray,
            reverse_parallel_ray: reverse_parallel_ray,

            aperture_stop: OnceCell::new(),
        }
    }

    pub fn aperture_stop(&self, surfaces: &[Surface]) -> &usize {
        self.aperture_stop.get_or_init(|| {
            // Get all the semi-diameters of the surfaces and put them in an ndarray.
            let semi_diameters = Array::from_vec(
                surfaces
                    .iter()
                    .map(|surface| surface.semi_diameter())
                    .collect::<Vec<Float>>(),
            );

            let ratios = semi_diameters
                / self.pseudo_marginal_ray[[self.pseudo_marginal_ray.shape()[0] - 1, 0, 0]];

            // Do not include the object or image surfaces when computing the aperture stop.
            argmin(&ratios.slice(s![1..-1])) + 1
        })
    }

    /// Compute the pseudo-marginal ray.
    pub fn calc_pseudo_marginal_ray(
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
    ) -> ParaxialRayTraceResults {
        let ray = if sequential_sub_model.is_obj_at_inf() {
            // Ray parallel to axis at a height of 1
            arr2(&[[1.0], [0.0]])
        } else {
            // Ray starting from the axis at an angle of 1
            arr2(&[[0.0], [1.0]])
        };

        Self::trace(ray, sequential_sub_model, surfaces, axis, false)
    }

    /// Compute the reverse parallel ray.
    pub fn calc_reverse_parallel_ray(
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
    ) -> ParaxialRayTraceResults {
        let ray = arr2(&[[1.0], [0.0]]);

        Self::trace(ray, sequential_sub_model, surfaces, axis, true)
    }

    /// Compute the ray transfer matrix for each gap/surface pair.
    fn rtms(
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
        reverse: bool,
    ) -> Vec<RayTransferMatrix> {
        let mut txs: Vec<RayTransferMatrix> = Vec::new();
        let mut forward_iter;
        let mut reverse_iter;
        let steps: &mut dyn Iterator<Item = Step> = if reverse {
            reverse_iter = sequential_sub_model.iter(surfaces).reverse();
            &mut reverse_iter
        } else {
            forward_iter = sequential_sub_model.iter(surfaces);
            &mut forward_iter
        };

        for (gap_0, surface, gap_1) in steps {
            let t = if gap_0.thickness.is_infinite() {
                DEFAULT_THICKNESS
            } else if reverse {
                -gap_0.thickness
            } else {
                gap_0.thickness
            };

            let roc = surface.roc(axis);

            let n_0 = gap_0.refractive_index.n();

            let n_1 = if let Some(gap_1) = gap_1 {
                gap_1.refractive_index.n()
            } else {
                gap_0.refractive_index.n()
            };

            let rtm = surface_to_rtm(surface, t, roc, n_0, n_1);
            txs.push(rtm);
        }

        txs
    }

    fn trace(
        rays: ParaxialRays,
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        axis: Axis,
        reverse: bool,
    ) -> ParaxialRayTraceResults {
        let txs = Self::rtms(sequential_sub_model, surfaces, axis, reverse);

        // Initialize the results array by assigning the input rays to the first
        // surface.
        let mut results = Array3::zeros((txs.len() + 1, 2, rays.shape()[1]));
        results.slice_mut(s![0, .., ..]).assign(&rays);

        // Iterate over the surfaces and compute the ray trace results.
        for (i, tx) in txs.iter().enumerate() {
            let rays = results.slice(s![i, .., ..]);
            let rays = tx.dot(&rays);
            results.slice_mut(s![i + 1, .., ..]).assign(&rays);
        }

        results
    }
}

/// Compute the ray transfer matrix for propagation to and interaction with a
/// surface.
fn surface_to_rtm(
    surface: &Surface,
    t: Float,
    roc: Float,
    n_0: Float,
    n_1: Float,
) -> RayTransferMatrix {
    let surface_type = surface.surface_type();

    match surface {
        // Conics and torics behave the same in paraxial subviews.
        Surface::Conic(_) | Surface::Toric(_) => match surface_type {
            SurfaceType::Refracting => arr2(&[
                [1.0, t],
                [
                    (n_0 - n_1) / n_1 / roc,
                    t * (n_0 - n_1) / n_1 / roc + n_0 / n_1,
                ],
            ]),
            SurfaceType::Reflecting => arr2(&[[1.0, t], [-2.0 / roc, 1.0 - 2.0 * t / roc]]),
            SurfaceType::NoOp => panic!("Conics and torics cannot be NoOp surfaces."),
        },
        Surface::Image(_) | Surface::Probe(_) | Surface::Stop(_) => arr2(&[[1.0, t], [0.0, 1.0]]),
        Surface::Object(_) => arr2(&[[1.0, 0.0], [0.0, 1.0]]),
    }
}

// Consider moving these to integration tests once the paraxial view and
// sequential models are combined into a system.
#[cfg(test)]
mod test {
    use approx::assert_abs_diff_eq;
    use ndarray::arr3;

    use crate::core::sequential_model::SubModelID;
    use crate::examples::convexplano_lens;
    use crate::systems::System;

    use super::*;

    fn setup() -> (ParaxialSubView, System) {
        let system = convexplano_lens::system();
        let seq_sub_model = system
            .sequential_model()
            .submodels()
            .get(&SubModelID(Some(0usize), Axis::Y))
            .expect("Submodel not found.");

        (
            ParaxialSubView::new(seq_sub_model, system.sequential_model().surfaces(), Axis::Y),
            system,
        )
    }

    #[test]
    fn test_aperture_stop() {
        let (view, system) = setup();

        let aperture_stop = view.aperture_stop(system.sequential_model().surfaces());
        let expected = 1;

        assert_eq!(*aperture_stop, expected);
    }

    #[test]
    fn test_pseudo_marginal_ray() {
        let system = convexplano_lens::system();
        let seq_sub_model = system
            .sequential_model()
            .submodels()
            .get(&SubModelID(Some(0usize), Axis::Y))
            .expect("Submodel not found.");
        let pseudo_marginal_ray = ParaxialSubView::calc_pseudo_marginal_ray(
            &seq_sub_model,
            system.sequential_model().surfaces(),
            Axis::Y,
        );

        let expected = arr3(&[
            [[1.0000], [0.0]],
            [[1.0000], [-0.0132]],
            [[0.9302], [-0.0200]],
            [[0.0], [-0.0200]],
        ]);

        assert_abs_diff_eq!(pseudo_marginal_ray, expected, epsilon = 1e-4);
    }

    #[test]
    fn test_reverse_parallel_ray() {
        let system = convexplano_lens::system();
        let seq_sub_model = system
            .sequential_model()
            .submodels()
            .get(&SubModelID(Some(0usize), Axis::Y))
            .expect("Submodel not found.");
        let reverse_parallel_ray = ParaxialSubView::calc_reverse_parallel_ray(
            &seq_sub_model,
            system.sequential_model().surfaces(),
            Axis::Y,
        );

        let expected = arr3(&[[[1.0000], [0.0]], [[1.0000], [0.0]], [[1.0000], [0.0200]]]);

        assert_abs_diff_eq!(reverse_parallel_ray, expected, epsilon = 1e-4);
    }
}
