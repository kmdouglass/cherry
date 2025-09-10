#[cfg(feature = "ri-info")]
mod test_ri_info {
    use core::panic;
    use std::rc::Rc;
    /// Tests the lens with material data from the refractiveindex.info
    /// database.
    use std::{collections::HashMap, io::Read};

    use anyhow::Result;
    use approx::assert_abs_diff_eq;
    use lib_ria::Store;

    use cherry_rs::examples::convexplano_lens::sequential_model;
    use cherry_rs::{Axis, FieldSpec, ParaxialView, PupilSampling};

    pub fn load_store() -> Result<Store> {
        let filename = std::path::PathBuf::from("data/rii.db");
        let file = std::fs::File::open(filename)?;
        let reader = std::io::BufReader::new(file);
        let data = reader.bytes().collect::<Result<Vec<u8>, _>>()?;

        let store: Store = bitcode::deserialize(&data)?;
        Ok(store)
    }

    // Inputs
    const WAVELENGTHS: [f64; 3] = [0.4861, 0.5876, 0.6563]; // Fraunhofer F, d, and C lines
    const FIELD_SPECS: [FieldSpec; 2] = [
        FieldSpec::Angle {
            angle: 0.0,
            pupil_sampling: PupilSampling::TangentialRayFan,
        },
        FieldSpec::Angle {
            angle: 5.0,
            pupil_sampling: PupilSampling::TangentialRayFan,
        },
    ];

    // Paraxial property values
    fn primary_axial_color() -> HashMap<Axis, f64> {
        let mut primary_axial_color = HashMap::new();
        primary_axial_color.insert(Axis::Y, 0.7743);
        primary_axial_color
    }

    #[test]
    fn test_feature_enabled() {
        println!("Feature ri-info is enabled!");
        assert!(cfg!(feature = "ri-info"));
    }

    #[test]
    fn test_paraxial_view_primary_axial_color() {
        let mut store = load_store().unwrap();

        // Remove the item so that we can pass ownership to a Rc.
        let air = Rc::new(store.remove("other:air:Ciddor").unwrap());
        let nbk7 = Rc::new(store.remove("glass:BK7:SCHOTT").unwrap());

        let model = sequential_model(air, nbk7, &WAVELENGTHS);
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        let expected = primary_axial_color();
        let results = view.primary_axial_color();

        assert_eq!(expected.len(), results.len());
        for (axis, expected_value) in expected.iter() {
            let result = results.get(axis).unwrap();
            assert_abs_diff_eq!(expected_value, result, epsilon = 1e-4);
        }
    }
}
