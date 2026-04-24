use super::model::{FieldMode, FieldRow, SurfaceKind, SurfaceRow, SurfaceVariant, SystemSpecs};

/// Figure-Z two-mirror system: two flat mirrors at 30° tilt, separated by 100
/// mm, returning the beam parallel to the z-axis.
pub fn mirrors_figure_z() -> SystemSpecs {
    SystemSpecs {
        surfaces: vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Reflecting,
                refractive_index: "1.0".into(),
                thickness: "100.0".into(),
                semi_diameter: "12.7".into(),
                radius_of_curvature: "Infinity".into(),
                conic_constant: "0.0".into(),
                theta: "30".into(),
                psi: "0".into(),
                material_key: None,
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Reflecting,
                refractive_index: "1.0".into(),
                thickness: "50.0".into(),
                semi_diameter: "12.7".into(),
                radius_of_curvature: "Infinity".into(),
                conic_constant: "0.0".into(),
                theta: "30".into(),
                psi: "0".into(),
                material_key: None,
            },
            SurfaceRow::new_image(),
        ],
        fields: vec![FieldRow {
            chi: "0.0".into(),
            phi: "90.0".into(),
            x: "0.0".into(),
        }],
        aperture_semi_diameter: "10.9".into(),
        wavelengths: vec!["0.567".into()],
        field_mode: FieldMode::Angle,
        use_materials: false,
        selected_materials: Vec::new(),
        cross_section_n_rays: 11,
        full_pupil_spacing: "0.1".into(),
        n_fan_rays: 65,
        background_n: "1.0".into(),
        background_material_key: None,
        stop_surface: None,
    }
}

/// Petzval lens example (5 glass elements).
pub fn petzval_lens() -> SystemSpecs {
    SystemSpecs {
        surfaces: vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_conic("28.478", "99.56266", "0.0", "13.0", "1.5168"),
            SurfaceRow::new_conic("26.276", "-86.84002", "0.0", "4.0", "1.6645"),
            SurfaceRow::new_conic("21.02", "-1187.63858", "0.0", "40.0", "1.0"),
            SurfaceRow::new_iris("16.631", "40.0", "1.0"),
            SurfaceRow::new_conic("20.543", "57.47491", "0.0", "12.0", "1.6074"),
            SurfaceRow::new_conic("20.074", "-54.61685", "0.0", "3.0", "1.6727"),
            SurfaceRow::new_conic("20.074", "-614.68633", "0.0", "46.8221", "1.0"),
            SurfaceRow::new_conic("17.297", "-38.1711", "0.0", "2.0", "1.6727"),
            SurfaceRow::new_conic("18.94", "Infinity", "0.0", "1.87179", "1.0"),
            SurfaceRow::new_image(),
        ],
        fields: vec![
            FieldRow {
                chi: "0.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            },
            FieldRow {
                chi: "5.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            },
        ],
        aperture_semi_diameter: "16.631".into(),
        wavelengths: vec!["0.567".into()],
        field_mode: FieldMode::Angle,
        use_materials: false,
        selected_materials: Vec::new(),
        cross_section_n_rays: 11,
        full_pupil_spacing: "0.1".into(),
        n_fan_rays: 65,
        background_n: "1.0".into(),
        background_material_key: None,
        stop_surface: None,
    }
}

/// f = +100 mm biconvex lens (Thorlabs LB1676-A) with a finite object at 200
/// mm.
pub fn biconvex_lens() -> SystemSpecs {
    SystemSpecs {
        surfaces: vec![
            SurfaceRow::new_object("200.0"),
            SurfaceRow::new_conic("12.7", "102.4", "0.0", "3.6", "1.517"),
            SurfaceRow::new_conic("12.7", "-102.4", "0.0", "196.1684", "1.0"),
            SurfaceRow::new_image(),
        ],
        fields: vec![
            FieldRow {
                chi: "0.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            },
            FieldRow {
                chi: "5.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            },
        ],
        aperture_semi_diameter: "5.0".into(),
        wavelengths: vec!["0.5876".into()],
        field_mode: FieldMode::PointSource,
        use_materials: false,
        selected_materials: Vec::new(),
        cross_section_n_rays: 11,
        full_pupil_spacing: "0.1".into(),
        n_fan_rays: 65,
        background_n: "1.0".into(),
        background_material_key: None,
        stop_surface: None,
    }
}

/// f = 50 mm convexplano lens with BK7 glass and Ciddor air (F, d, C
/// wavelengths).
pub fn convexplano_lens_with_materials() -> SystemSpecs {
    SystemSpecs {
        surfaces: vec![
            SurfaceRow {
                variant: SurfaceVariant::Object,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "Infinity".into(),
                semi_diameter: "12.5".into(),
                radius_of_curvature: "Infinity".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("other:air:Ciddor".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.515".into(),
                thickness: "5.3".into(),
                semi_diameter: "12.5".into(),
                radius_of_curvature: "25.8".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("popular_glass:BK7:SCHOTT".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "46.6".into(),
                semi_diameter: "12.5".into(),
                radius_of_curvature: "Infinity".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("other:air:Ciddor".into()),
            },
            SurfaceRow::new_image(),
        ],
        fields: vec![
            FieldRow {
                chi: "0.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            },
            FieldRow {
                chi: "5.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            },
        ],
        aperture_semi_diameter: "5.0".into(),
        wavelengths: vec!["0.4861".into(), "0.5876".into(), "0.6563".into()],
        field_mode: FieldMode::Angle,
        use_materials: true,
        selected_materials: vec!["other:air:Ciddor".into(), "popular_glass:BK7:SCHOTT".into()],
        cross_section_n_rays: 11,
        full_pupil_spacing: "0.1".into(),
        n_fan_rays: 65,
        background_n: "1.0".into(),
        background_material_key: Some("other:air:Ciddor".into()),
        stop_surface: None,
    }
}

/// Compact f-theta scan lens with three N-SF57 glass elements (F, d, C
/// wavelengths).
///
/// Milton Laikin, *Lens Design*, 4th ed., CRC Press, 2007, p. 251.
pub fn f_theta_scan_lens() -> SystemSpecs {
    SystemSpecs {
        surfaces: vec![
            SurfaceRow {
                variant: SurfaceVariant::Object,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "Infinity".into(),
                semi_diameter: "12.5".into(),
                radius_of_curvature: "Infinity".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("other:air:Ciddor".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Iris,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "5".into(),
                semi_diameter: "0.5".into(),
                radius_of_curvature: "Infinity".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("other:air:Ciddor".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "0.3".into(),
                semi_diameter: "2".into(),
                radius_of_curvature: "-2.2136".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("specs:SCHOTT-optical:N-SF57".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "0.02".into(),
                semi_diameter: "2".into(),
                radius_of_curvature: "-2.6575".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("other:air:Ciddor".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "0.5292".into(),
                semi_diameter: "2".into(),
                radius_of_curvature: "-5.5022".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("specs:SCHOTT-optical:N-SF57".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "4.2927".into(),
                semi_diameter: "2".into(),
                radius_of_curvature: "-3.8129".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("other:air:Ciddor".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "0.59".into(),
                semi_diameter: "3".into(),
                radius_of_curvature: "7.9951".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("specs:SCHOTT-optical:N-SF57".into()),
            },
            SurfaceRow {
                variant: SurfaceVariant::Conic,
                surface_kind: SurfaceKind::Refracting,
                refractive_index: "1.0".into(),
                thickness: "17.6".into(),
                semi_diameter: "3".into(),
                radius_of_curvature: "8.3651".into(),
                conic_constant: "0.0".into(),
                theta: "0".into(),
                psi: "0".into(),
                material_key: Some("other:air:Ciddor".into()),
            },
            SurfaceRow::new_image(),
        ],
        fields: vec![FieldRow {
            chi: "0".into(),
            phi: "90.0".into(),
            x: "0.0".into(),
        }],
        aperture_semi_diameter: "0.49".into(),
        wavelengths: vec!["0.4861".into(), "0.5876".into(), "0.6563".into()],
        field_mode: FieldMode::Angle,
        use_materials: true,
        selected_materials: vec![
            "other:air:Ciddor".into(),
            "specs:SCHOTT-optical:N-SF57".into(),
        ],
        cross_section_n_rays: 3,
        full_pupil_spacing: "0.1".into(),
        n_fan_rays: 65,
        background_n: "1.0".into(),
        background_material_key: Some("other:air:Ciddor".into()),
        stop_surface: None,
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
                theta: "0".into(),
                psi: "0".into(),
                material_key: None,
            },
            SurfaceRow::new_image(),
        ],
        fields: vec![
            FieldRow {
                chi: "0.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            },
            FieldRow {
                chi: "5.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            },
        ],
        aperture_semi_diameter: "12.5".into(),
        wavelengths: vec!["0.567".into()],
        field_mode: FieldMode::Angle,
        use_materials: false,
        selected_materials: Vec::new(),
        cross_section_n_rays: 11,
        full_pupil_spacing: "0.1".into(),
        n_fan_rays: 65,
        background_n: "1.0".into(),
        background_material_key: None,
        stop_surface: None,
    }
}
