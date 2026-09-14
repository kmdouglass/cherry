use super::model::{
    BoundaryVariant, FieldMode, FieldRow, PathRow, RowId, SolveSpec, SurfaceRow, SurfaceVariant,
    SystemSpecs,
};

/// Simple thin lens: f = 100 mm, object at infinity.
///
/// Carries an M (marginal ray height) solve on the lens-to-image gap with a
/// target height of 0, demonstrating how to keep the image plane at the
/// lens's focus instead of hand-entering the back focal distance.
pub fn thin_lens() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow::new_object("Infinity"),
        SurfaceRow::new_thin_lens("12.5", "100.0", "50.0", "1.0"),
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![FieldRow {
        chi: "0.0".into(),
        phi: "90.0".into(),
        x: "0.0".into(),
    }];
    path.aperture_semi_diameter = "12.5".into();
    path.wavelengths = vec!["0.5876".into()];
    let mut specs = SystemSpecs::new_single(path);
    specs.solves = vec![SolveSpec::MarginalRayHeight {
        gap_index: 1,
        target_height: 0.0,
        wavelength_id: 0,
        path_id: 0,
    }];
    specs
}

/// Figure-Z two-mirror system: two flat mirrors at 30° tilt, separated by 100
/// mm, returning the beam parallel to the z-axis.
pub fn mirrors_figure_z() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow::new_object("Infinity"),
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Reflecting,
            refractive_index: "1.0".into(),
            thickness: "100.0".into(),
            semi_diameter: "12.7".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "30".into(),
            psi: "0".into(),
            material_key: None,
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Reflecting,
            refractive_index: "1.0".into(),
            thickness: "50.0".into(),
            semi_diameter: "12.7".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "30".into(),
            psi: "0".into(),
            material_key: None,
        },
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![FieldRow {
        chi: "0.0".into(),
        phi: "90.0".into(),
        x: "0.0".into(),
    }];
    path.aperture_semi_diameter = "10.9".into();
    path.wavelengths = vec!["0.567".into()];
    let mut specs = SystemSpecs::new_single(path);
    specs.cross_section_n_rays = 11;
    specs
}

/// Petzval lens example (5 glass elements).
pub fn petzval_lens() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow::new_object("Infinity"),
        SurfaceRow::new_sphere("28.478", "99.56266", "13.0", "1.5168"),
        SurfaceRow::new_sphere("26.276", "-86.84002", "4.0", "1.6645"),
        SurfaceRow::new_sphere("21.02", "-1187.63858", "40.0", "1.0"),
        SurfaceRow::new_iris("16.631", "40.0", "1.0"),
        SurfaceRow::new_sphere("20.543", "57.47491", "12.0", "1.6074"),
        SurfaceRow::new_sphere("20.074", "-54.61685", "3.0", "1.6727"),
        SurfaceRow::new_sphere("20.074", "-614.68633", "46.8221", "1.0"),
        SurfaceRow::new_sphere("17.297", "-38.1711", "2.0", "1.6727"),
        SurfaceRow::new_sphere("18.94", "Infinity", "1.87179", "1.0"),
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![
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
    ];
    path.aperture_semi_diameter = "16.631".into();
    path.wavelengths = vec!["0.567".into()];
    let mut specs = SystemSpecs::new_single(path);
    specs.cross_section_n_rays = 11;
    specs
}

/// f = +100 mm biconvex lens (Thorlabs LB1676-A) with a finite object at 200
/// mm.
pub fn biconvex_lens() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow::new_object("200.0"),
        SurfaceRow::new_sphere("12.7", "102.4", "3.6", "1.517"),
        SurfaceRow::new_sphere("12.7", "-102.4", "196.1684", "1.0"),
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![
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
    ];
    path.aperture_semi_diameter = "5.0".into();
    path.wavelengths = vec!["0.5876".into()];
    path.field_mode = FieldMode::PointSource;
    let mut specs = SystemSpecs::new_single(path);
    specs.cross_section_n_rays = 11;
    specs
}

/// f = 50 mm convexplano lens with BK7 glass and Ciddor air (F, d, C
/// wavelengths).
pub fn convexplano_lens_with_materials() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Object,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "Infinity".into(),
            semi_diameter: "12.5".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: "0.0".into(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.515".into(),
            thickness: "5.3".into(),
            semi_diameter: "12.5".into(),
            radius_of_curvature: "25.8".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("popular_glass:BK7:SCHOTT".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "46.6".into(),
            semi_diameter: "12.5".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![
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
    ];
    path.aperture_semi_diameter = "12.0".into();
    path.wavelengths = vec!["0.4861".into(), "0.5876".into(), "0.6563".into()];
    let mut specs = SystemSpecs::new_single(path);
    specs.use_materials = true;
    specs.selected_materials = vec!["other:air:Ciddor".into(), "popular_glass:BK7:SCHOTT".into()];
    specs.cross_section_n_rays = 11;
    specs.background_material_key = Some("other:air:Ciddor".into());
    specs
}

/// Compact f-theta scan lens with three N-SF57 glass elements (F, d, C
/// wavelengths).
///
/// Milton Laikin, *Lens Design*, 4th ed., CRC Press, 2007, p. 251.
pub fn f_theta_scan_lens() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Object,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "Infinity".into(),
            semi_diameter: "12.5".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: "0.0".into(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Iris,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "5".into(),
            semi_diameter: "0.5".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: "0.0".into(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "0.3".into(),
            semi_diameter: "2".into(),
            radius_of_curvature: "-2.2136".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("specs:SCHOTT-optical:N-SF57".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "0.02".into(),
            semi_diameter: "2".into(),
            radius_of_curvature: "-2.6575".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "0.5292".into(),
            semi_diameter: "2".into(),
            radius_of_curvature: "-5.5022".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("specs:SCHOTT-optical:N-SF57".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "4.2927".into(),
            semi_diameter: "2".into(),
            radius_of_curvature: "-3.8129".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "0.59".into(),
            semi_diameter: "3".into(),
            radius_of_curvature: "7.9951".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("specs:SCHOTT-optical:N-SF57".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "17.6".into(),
            semi_diameter: "3".into(),
            radius_of_curvature: "8.3651".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![FieldRow {
        chi: "0".into(),
        phi: "90.0".into(),
        x: "0.0".into(),
    }];
    path.aperture_semi_diameter = "0.49".into();
    path.wavelengths = vec!["0.4861".into(), "0.5876".into(), "0.6563".into()];
    let mut specs = SystemSpecs::new_single(path);
    specs.use_materials = true;
    specs.selected_materials = vec![
        "other:air:Ciddor".into(),
        "specs:SCHOTT-optical:N-SF57".into(),
    ];
    specs.background_material_key = Some("other:air:Ciddor".into());
    specs
}

/// Scan lens designed for use with a galvo mirror (Negrean and Mansvelder).
///
/// A. Negrean and H. D. Mansvelder, "Optimal lens design and use in
/// laser-scanning microscopy," *Biomedical Optics Express*, vol. 5, no. 5, p.
/// 1588, May 2014.
pub fn galvo_scan_lens_negrean_mansvelder() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Object,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "Infinity".into(),
            semi_diameter: String::new(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Reflecting,
            refractive_index: "1.0".into(),
            thickness: "15.1986".into(),
            semi_diameter: "2".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "-45".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "9.0163".into(),
            semi_diameter: "9".into(),
            radius_of_curvature: "21.423".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("specs:SCHOTT-optical:N-KZFS5".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "15.121".into(),
            semi_diameter: "8".into(),
            radius_of_curvature: "13.471".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "13.0006".into(),
            semi_diameter: "15".into(),
            radius_of_curvature: "88.222".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("specs:SCHOTT-optical:N-PK51".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "0.2".into(),
            semi_diameter: "15".into(),
            radius_of_curvature: "-23.392".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "8.6136".into(),
            semi_diameter: "15".into(),
            radius_of_curvature: "211.304".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("specs:OHARA-optical:S-FPM2".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "6.0003".into(),
            semi_diameter: "15".into(),
            radius_of_curvature: "-20.385".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("specs:SCHOTT-optical:N-KZFS11".into()),
        },
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Refracting,
            refractive_index: "1.0".into(),
            thickness: "41.1639".into(),
            semi_diameter: "15".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: Some("other:air:Ciddor".into()),
        },
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![FieldRow {
        chi: "0".into(),
        phi: "90.0".into(),
        x: "0.0".into(),
    }];
    path.aperture_semi_diameter = "1.3".into();
    path.wavelengths = vec![
        "0.405".into(),
        "0.488".into(),
        "0.561".into(),
        "0.647".into(),
    ];
    path.stop_surface = Some(1);
    let mut specs = SystemSpecs::new_single(path);
    specs.use_materials = true;
    specs.selected_materials = vec![
        "other:air:Ciddor".into(),
        "specs:SCHOTT-optical:N-KZFS5".into(),
        "specs:SCHOTT-optical:N-PK51".into(),
        "specs:SCHOTT-optical:N-KZFS11".into(),
        "specs:OHARA-optical:S-FPM2".into(),
    ];
    specs.cross_section_n_rays = 11;
    specs.background_material_key = Some("other:air:Ciddor".into());
    specs
}

/// Reduced widefield epifluorescence microscope excitation path.
///
/// An object, a thin relay lens (f = 40 mm), a flat fold mirror at 45°, and a
/// second thin lens (f = 3.3333 mm) that acts as the aperture stop, modelling
/// a microscope objective back aperture immersed in oil (n = 1.5). The fold
/// mirror contributes no power; only the unfolded track length of 150 mm
/// between the two lenses matters for paraxial calculations.
pub fn wf_epi_excitation() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow::new_object("40.0"),
        SurfaceRow::new_thin_lens("25.0", "40.0", "100.0", "1.0"),
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Reflecting,
            refractive_index: "1.0".into(),
            thickness: "50.0".into(),
            semi_diameter: "36.0".into(),
            radius_of_curvature: "Infinity".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "45".into(),
            psi: "0".into(),
            material_key: None,
        },
        SurfaceRow::new_thin_lens("4.25", "3.3333", "1.0", "1.5"),
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![FieldRow {
        chi: "1.5".into(),
        phi: "90.0".into(),
        x: "0.0".into(),
    }];
    path.aperture_semi_diameter = "1.5454".into();
    path.wavelengths = vec!["0.5876".into()];
    path.field_mode = FieldMode::PointSource;
    path.stop_surface = Some(3);
    let mut specs = SystemSpecs::new_single(path);
    specs.cross_section_n_rays = 11;
    specs.solves = vec![SolveSpec::MarginalRayHeight {
        gap_index: 3,
        target_height: 0.0,
        wavelength_id: 0,
        path_id: 0,
    }];
    specs
}

/// f = +100 mm concave mirror.
pub fn concave_mirror() -> SystemSpecs {
    let mut path = PathRow::from_surface_rows(vec![
        SurfaceRow::new_object("Infinity"),
        SurfaceRow {
            id: RowId::default(),
            variant: SurfaceVariant::Sphere,
            boundary_variant: BoundaryVariant::Reflecting,
            refractive_index: "1.0".into(),
            thickness: "100.0".into(),
            semi_diameter: "12.5".into(),
            radius_of_curvature: "-200.0".into(),
            conic_constant: String::new(),
            focal_length: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        },
        SurfaceRow::new_image(),
    ]);
    path.fields = vec![
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
    ];
    path.aperture_semi_diameter = "12.5".into();
    path.wavelengths = vec!["0.567".into()];
    let mut specs = SystemSpecs::new_single(path);
    specs.cross_section_n_rays = 11;
    specs.solves = vec![SolveSpec::MarginalRayHeight {
        gap_index: 1,
        target_height: 0.0,
        wavelength_id: 0,
        path_id: 0,
    }];
    specs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SequentialModelBuilder, SequentialSubModel, gui::convert};

    fn parse(specs: &SystemSpecs) -> convert::ParsedSpecs {
        #[cfg(not(feature = "ri-info"))]
        return convert::convert_specs(specs).expect("convert");
        #[cfg(feature = "ri-info")]
        return convert::convert_specs(specs, &Default::default()).expect("convert");
    }

    /// The thin lens preset must parse into a valid, buildable model — the
    /// bug this guards against: a stubbed-out field (e.g. an empty
    /// `focal_length`) that compiles but fails at conversion time.
    #[test]
    fn thin_lens_example_converts_to_valid_model() {
        let specs = thin_lens();
        let parsed = parse(&specs);
        SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .build()
            .expect("model");
    }

    #[test]
    fn wf_epi_excitation_example_converts_to_valid_model() {
        let specs = wf_epi_excitation();
        let parsed = parse(&specs);
        SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .build()
            .expect("model");
    }

    /// The M solve on the lens-to-image gap must resolve to the lens's back
    /// focal distance (== focal length, for a thin lens in air with an
    /// object at infinity), demonstrating that the image plane tracks the
    /// lens's focus rather than a hand-entered distance.
    #[test]
    fn thin_lens_example_solve_places_image_at_focus() {
        let specs = thin_lens();
        let parsed = parse(&specs);
        let model = SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .solves(parsed.solves)
            .build()
            .expect("model with solve applied")
            .model;

        let solved_thickness = model.submodels_for_path(0)[0].gaps()[1].thickness;
        approx::assert_abs_diff_eq!(solved_thickness, 100.0, epsilon = 1e-6);
    }
}
