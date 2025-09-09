use super::model::{
    FieldMode, FieldRow, SurfaceKind, SurfaceRow, SurfaceVariant, SystemSpecs,
};

/// Petzval lens example (5 glass elements).
pub fn petzval_lens() -> SystemSpecs {
    SystemSpecs {
        surfaces: vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_conic("28.478", "99.56266", "0.0", "13.0", "1.5168"),
            SurfaceRow::new_conic("26.276", "-86.84002", "0.0", "4.0", "1.6645"),
            SurfaceRow::new_conic("21.02", "-1187.63858", "0.0", "40.0", "1.0"),
            SurfaceRow::new_stop("16.631", "40.0", "1.0"),
            SurfaceRow::new_conic("20.543", "57.47491", "0.0", "12.0", "1.6074"),
            SurfaceRow::new_conic("20.074", "-54.61685", "0.0", "3.0", "1.6727"),
            SurfaceRow::new_conic("20.074", "-614.68633", "0.0", "46.8221", "1.0"),
            SurfaceRow::new_conic("17.297", "-38.1711", "0.0", "2.0", "1.6727"),
            SurfaceRow::new_conic("18.94", "Infinity", "0.0", "1.87179", "1.0"),
            SurfaceRow::new_image(),
        ],
        fields: vec![
            FieldRow {
                value: "0.0".into(),
                x: "0.0".into(),
                pupil_spacing: "0.1".into(),
            },
            FieldRow {
                value: "5.0".into(),
                x: "0.0".into(),
                pupil_spacing: "0.1".into(),
            },
        ],
        aperture_semi_diameter: "16.631".into(),
        wavelengths: vec!["0.567".into()],
        field_mode: FieldMode::Angle,
        use_materials: false,
        selected_materials: Vec::new(),
    }
}

/// f = +100 mm concave mirror.
pub fn concave_mirror() -> SystemSpecs {
    SystemSpecs {
        surfaces: vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Reflecting,
                refractive_index: "1.0".into(),
                thickness: "100.0".into(),
                semi_diameter: "12.5".into(),
                radius_of_curvature: "-200.0".into(),
                conic_constant: "0.0".into(),
                material_key: None,
            },
            SurfaceRow::new_image(),
        ],
        fields: vec![FieldRow {
            value: "0.0".into(),
            x: "0.0".into(),
            pupil_spacing: "0.1".into(),
        }],
        aperture_semi_diameter: "12.5".into(),
        wavelengths: vec!["0.567".into()],
        field_mode: FieldMode::Angle,
        use_materials: false,
        selected_materials: Vec::new(),
    }
}
