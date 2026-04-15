#[cfg(feature = "ri-info")]
mod test_ri_info {
    use core::panic;
    /// Tests the lens with material data from the refractiveindex.info
    /// database.
    use std::io::Read;
    use std::rc::Rc;

    use anyhow::Result;
    use approx::assert_abs_diff_eq;
    use lib_ria::Store;

    use cherry_rs::examples::convexplano_lens::sequential_model;
    use cherry_rs::{FieldSpec, ParaxialView};

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
            chi: 0.0,
            phi: 90.0,
        },
        FieldSpec::Angle {
            chi: 5.0,
            phi: 90.0,
        },
    ];

    #[test]
    fn test_feature_enabled() {
        println!("Feature ri-info is enabled!");
        const { assert!(cfg!(feature = "ri-info")) };
    }

    #[test]
    fn test_paraxial_view_primary_axial_color() {
        let mut store = load_store().unwrap();

        // Remove the item so that we can pass ownership to a Rc.
        let air = Rc::new(store.remove("other:air:Ciddor").unwrap());
        let nbk7 = Rc::new(store.remove("popular_glass:BK7:SCHOTT").unwrap());

        let model = sequential_model(air, nbk7, &WAVELENGTHS);
        let view =
            ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

        // For a single phi=90° field there is one tangential direction
        // (tangential_vec_id=0).
        let results = view.primary_axial_color();
        assert_eq!(results.len(), 1);
        assert_abs_diff_eq!(results[0], 0.7743, epsilon = 1e-4);
    }
}
