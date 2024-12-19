use std::rc::Rc;

use cherry_rs::{
    FieldSpec, GapSpec, PupilSampling, RefractiveIndexSpec, SequentialModel, SurfaceSpec,
    SurfaceType,
};

pub fn sequential_model(
    air: Rc<dyn RefractiveIndexSpec>,
    nbk7: Rc<dyn RefractiveIndexSpec>,
) -> SequentialModel {
    let gap_0 = GapSpec {
        thickness: f64::INFINITY,
        refractive_index: air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 5.3,
        refractive_index: nbk7,
    };
    let gap_2 = GapSpec {
        thickness: 46.6,
        refractive_index: air,
    };
    let gaps = vec![gap_0, gap_1, gap_2];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: 25.8,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: f64::INFINITY,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_3 = SurfaceSpec::Image;
    let surfaces = vec![surf_0, surf_1, surf_2, surf_3];

    let wavelengths: Vec<f64> = vec![0.5678];

    SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
}

pub const FIELD_SPECS: [FieldSpec; 2] = [
    FieldSpec::Angle {
        angle: 0.0,
        pupil_sampling: PupilSampling::ChiefAndMarginalRays,
    },
    FieldSpec::Angle {
        angle: 5.0,
        pupil_sampling: PupilSampling::ChiefAndMarginalRays,
    },
];

#[cfg(test)]
mod test_constant_refractive_indexes {
    use approx::assert_abs_diff_eq;
    use ndarray::{arr3, Array3};

    use cherry_rs::{n, ImagePlane, ParaxialView, Pupil};

    use super::FIELD_SPECS;

    const APERTURE_STOP: usize = 1;
    const BACK_FOCAL_DISTANCE: f64 = 46.5987;
    const BACK_PRINCIPAL_PLANE: f64 = 1.8017;
    const EFFECTIVE_FOCAL_LENGTH: f64 = 50.097;
    const ENTRANCE_PUPIL: Pupil = Pupil {
        location: 0.0,
        semi_diameter: 12.5,
    };
    const EXIT_PUPIL: Pupil = Pupil {
        location: 1.8017,
        semi_diameter: 12.5,
    };
    const FRONT_FOCAL_DISTANCE: f64 = -EFFECTIVE_FOCAL_LENGTH;
    const FRONT_PRINCIPAL_PLANE: f64 = 0.0;

    // For a 5 degree field angle
    const PARAXIAL_IMAGE_PLANE: ImagePlane = ImagePlane {
        location: 51.8987,
        semi_diameter: 4.3829,
    };

    // For a 5 degree field angle
    // Paraxial angle = tan(field angle)
    fn chief_ray() -> Array3<f64> {
        arr3(&[
            [[0.0], [0.087489]],
            [[0.0], [0.0577482]],
            [[0.306067], [0.087489]],
            [[4.382944], [0.087489]],
        ])
    }

    fn marginal_ray() -> Array3<f64> {
        arr3(&[
            [[12.5000], [0.0]],
            [[12.5000], [-0.1647]],
            [[11.6271], [-0.2495]],
            [[-0.0003], [-0.2495]],
        ])
    }

    #[test]
    fn test_paraxial_view_chief_ray() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
        let chief_ray = chief_ray();

        for sub_model_id in sub_models.keys() {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.chief_ray();

            assert_abs_diff_eq!(chief_ray, result, epsilon = 1e-4);
        }
    }

    #[test]
    fn test_paraxial_view_aperture_stop() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for sub_model_id in sub_models.keys() {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.aperture_stop();

            assert_eq!(APERTURE_STOP, *result)
        }
    }

    #[test]
    fn test_paraxial_view_back_focal_distance() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.back_focal_distance();

            assert_abs_diff_eq!(BACK_FOCAL_DISTANCE, *result, epsilon = 1e-4)
        }
    }

    #[test]
    fn test_paraxial_view_back_principal_plane() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.back_principal_plane();

            assert_abs_diff_eq!(BACK_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
        }
    }

    #[test]
    fn test_paraxial_view_entrance_pupil() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.entrance_pupil();

            assert_eq!(ENTRANCE_PUPIL, *result)
        }
    }

    #[test]
    fn test_paraxial_view_exit_pupil() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.exit_pupil();

            assert_abs_diff_eq!(EXIT_PUPIL.location, result.location, epsilon = 1e-4);
            assert_abs_diff_eq!(
                EXIT_PUPIL.semi_diameter,
                result.semi_diameter,
                epsilon = 1e-4
            );
        }
    }

    #[test]
    fn test_paraxial_view_effective_focal_length() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.effective_focal_length();

            assert_abs_diff_eq!(EFFECTIVE_FOCAL_LENGTH, *result, epsilon = 1e-4)
        }
    }

    #[test]
    fn test_paraxial_view_front_focal_distance() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.front_focal_distance();

            assert_abs_diff_eq!(FRONT_FOCAL_DISTANCE, *result, epsilon = 1e-4)
        }
    }

    #[test]
    fn test_paraxial_view_front_principal_plane() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.front_principal_plane();

            assert_abs_diff_eq!(FRONT_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
        }
    }

    #[test]
    fn test_paraxial_view_image_plane() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.paraxial_image_plane();

            assert_abs_diff_eq!(
                PARAXIAL_IMAGE_PLANE.location,
                result.location,
                epsilon = 1e-4
            );
            assert_abs_diff_eq!(
                PARAXIAL_IMAGE_PLANE.semi_diameter,
                result.semi_diameter,
                epsilon = 1e-4
            );
        }
    }

    #[test]
    fn test_paraxial_view_marginal_ray() {
        let model = super::sequential_model(n!(1.0), n!(1.515));
        let sub_models = model.submodels();
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
        let marginal_ray = marginal_ray();

        for sub_model_id in sub_models.keys() {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.marginal_ray();

            assert_abs_diff_eq!(marginal_ray, result, epsilon = 1e-4);
        }
    }
}

#[cfg(feature = "ri-info")]
#[cfg(test)]
mod test_ri_info {
    /// Tests the lens with material data from the refractiveindex.info
    /// database.
    use std::io::Read;
    use std::rc::Rc;

    use anyhow::Result;
    use approx::assert_abs_diff_eq;
    use lib_ria::Store;

    use cherry_rs::ParaxialView;

    pub fn load_store() -> Result<Store> {
        let filename = std::path::PathBuf::from("data/rii.db");
        let file = std::fs::File::open(filename)?;
        let reader = std::io::BufReader::new(file);
        let data = reader.bytes().collect::<Result<Vec<u8>, _>>()?;

        let store: Store = bitcode::deserialize(&data)?;
        Ok(store)
    }

    const BACK_FOCAL_DISTANCE: f64 = 46.5987;

    #[test]
    fn test_paraxial_view_back_focal_distance() {
        let mut store = load_store().unwrap();

        // Remove the item so that we can pass ownership to a Rc.
        let air = Rc::new(store.remove("other:air:Ciddor").unwrap());
        let nbk7 = Rc::new(store.remove("glass:BK7:SCHOTT").unwrap());

        let model = super::sequential_model(air, nbk7);
        let sub_models = model.submodels();
        let view = ParaxialView::new(&model, &super::FIELD_SPECS, false)
            .expect("Could not create paraxial view");

        for (sub_model_id, _) in sub_models {
            let sub_view = view.subviews().get(sub_model_id).unwrap();
            let result = sub_view.back_focal_distance();

            assert_abs_diff_eq!(BACK_FOCAL_DISTANCE, *result, epsilon = 1e-4)
        }
    }
}
