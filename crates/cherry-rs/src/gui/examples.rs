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

/// Reduced widefield epifluorescence microscope: excitation and emission
/// paths sharing an objective and a beam splitter.
///
/// Path 0 (excitation) introduces a relay lens, the beam splitter (store
/// index 2, reflecting for this path), and the objective (store index 3).
/// Path 1 (emission) starts at an `ObjectLinkedTo` the sample plane
/// (reversed orientation), `Shared`-references the objective and beam
/// splitter out of order (3 then 2, matching the physical light path back
/// through the system), then adds its own fold mirror and tube lens. Each
/// path declares its own kind for the shared beam splitter (excitation
/// reflects off it, emission transmits through it).
pub fn wf_epi_microscope() -> SystemSpecs {
    use super::model::{
        BeamSplitterPathKindRow, GapRow, LinkedObjectOrientationRow, SurfaceRefRow,
    };

    let mut specs = SystemSpecs::new_single(PathRow::from_surface_rows(vec![
        SurfaceRow::new_object("40.0"),
        SurfaceRow::new_thin_lens("25.0", "40.0", "100.0", "1.0"),
        SurfaceRow::new_beam_splitter("18.0", "50.0", "1.0"),
        SurfaceRow::new_thin_lens("4.25", "3.3333", "1.0", "1.5"),
        SurfaceRow::new_image(),
    ]));
    specs.paths[0].fields = vec![FieldRow {
        chi: "1.5".into(),
        phi: "90.0".into(),
        x: "0.0".into(),
    }];
    specs.paths[0].field_mode = FieldMode::PointSource;
    specs.paths[0].aperture_semi_diameter = "1.5454".into();
    specs.paths[0].wavelengths = vec!["0.488".into()];
    specs.paths[0].stop_surface = Some(3);
    specs.cross_section_n_rays = 3;
    specs.solves = vec![SolveSpec::MarginalRayHeight {
        gap_index: 3,
        target_height: 0.0,
        wavelength_id: 0,
        path_id: 0,
    }];
    specs.paths[0].beam_splitter_arms = vec![BeamSplitterPathKindRow::Reflecting];
    // BS at 45 degrees.
    if let SurfaceRefRow::New(row) = &mut specs.paths[0].surface_refs[2] {
        row.theta = "45".into();
    }

    let linked_id = specs.mint_row_id();
    let mirror_id = specs.mint_row_id();
    let tube_id = specs.mint_row_id();
    let img_id = specs.mint_row_id();
    specs.paths.push(PathRow {
        name: None,
        surface_refs: vec![
            SurfaceRefRow::ObjectLinkedTo {
                id: linked_id,
                path: 0,
                orientation: LinkedObjectOrientationRow::Reversed,
                gap_after: GapRow::default(),
            },
            SurfaceRefRow::Shared {
                target: RowId(3),
                gap_after: GapRow::default(),
            }, // Objective
            SurfaceRefRow::Shared {
                target: RowId(2),
                gap_after: GapRow::new("50.0", "1.0"),
            }, // BeamSplitter
            SurfaceRefRow::New({
                let mut m = SurfaceRow::new_conic("10.0", "Infinity", "0", "50.0", "1.0");
                m.boundary_variant = BoundaryVariant::Reflecting;
                m.theta = "-45".into();
                m.id = mirror_id;
                m
            }),
            SurfaceRefRow::New(
                SurfaceRow::new_thin_lens("20.0", "200.0", "200.0", "1.0").with_id(tube_id),
            ),
            SurfaceRefRow::New(SurfaceRow::new_image().with_id(img_id)),
        ],
        stop_surface: Some(3),
        fields: vec![FieldRow {
            chi: "0.0".into(),
            phi: "90.0".into(),
            x: "0.0".into(),
        }],
        field_mode: FieldMode::PointSource,
        aperture_semi_diameter: "1.5".into(),
        wavelengths: vec!["0.520".into()],
        beam_splitter_arms: vec![BeamSplitterPathKindRow::Transmitting],
    });

    specs
}

/// Michelson interferometer: two arms (one path each) that each revisit the
/// beam splitter on the return pass and recombine on a shared detector. See
/// `crate::examples::michelson_interferometer` for the topology.
pub fn michelson_interferometer() -> SystemSpecs {
    use super::model::{BeamSplitterPathKindRow, GapRow, SurfaceRefRow};

    let mirror = |thickness: &str| {
        let mut m = SurfaceRow::new_conic("12.7", "Infinity", "0", thickness, "1.0");
        m.boundary_variant = BoundaryVariant::Reflecting;
        m
    };
    let fields = || {
        ["0.0", "1.0"]
            .into_iter()
            .map(|chi| FieldRow {
                chi: chi.into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            })
            .collect::<Vec<_>>()
    };

    // Path 0 (arm 1): ids 0 Object, 1 Iris, 2 BS, 3 M1, 4 Lens, 5 Detector.
    let mut path_0 = PathRow::from_surface_rows(vec![
        SurfaceRow::new_object("Infinity"),
        SurfaceRow::new_iris("2.0", "50.0", "1.0"),
        SurfaceRow::new_beam_splitter("12.7", "100.0", "1.0"),
        mirror("100.0"), // M1
        SurfaceRow::new_thin_lens("12.7", "100.0", "100.0", "1.0"),
        SurfaceRow::new_image(),
    ]);
    let id_of = |path: &PathRow, step: usize| path.surface_refs[step].row_id().unwrap();
    let (object_id, iris_id, bs_id, lens_id, detector_id) = (
        id_of(&path_0, 0),
        id_of(&path_0, 1),
        id_of(&path_0, 2),
        id_of(&path_0, 4),
        id_of(&path_0, 5),
    );
    if let SurfaceRefRow::New(row) = &mut path_0.surface_refs[2] {
        row.theta = "45".into();
    }
    // Return pass: revisit the beam splitter on the way to the lens.
    path_0.surface_refs.insert(
        4,
        SurfaceRefRow::Shared {
            target: bs_id,
            gap_after: GapRow::new("50.0", "1.0"),
        },
    );
    path_0.name = Some("Arm 1 (M1)".into());
    path_0.beam_splitter_arms = vec![
        BeamSplitterPathKindRow::Transmitting,
        BeamSplitterPathKindRow::Reflecting,
    ];
    path_0.stop_surface = Some(1);
    path_0.fields = fields();
    path_0.field_mode = FieldMode::Angle;
    path_0.aperture_semi_diameter = "2.0".into();
    path_0.wavelengths = vec!["0.6328".into()];

    let mut specs = SystemSpecs::new_single(path_0);
    specs.cross_section_n_rays = 3;

    // Path 1 (arm 2): everything shared except its own mirror.
    let m2_id = specs.mint_row_id();
    let shared = |target: RowId, thickness: &str| SurfaceRefRow::Shared {
        target,
        gap_after: GapRow::new(thickness, "1.0"),
    };
    specs.paths.push(PathRow {
        name: Some("Arm 2 (M2)".into()),
        surface_refs: vec![
            SurfaceRefRow::Shared {
                target: object_id,
                gap_after: GapRow::new("Infinity", "1.0"),
            },
            shared(iris_id, "50.0"),
            shared(bs_id, "110.0"),
            SurfaceRefRow::New(mirror("110.0").with_id(m2_id)), // M2
            shared(bs_id, "50.0"),                              // revisit
            shared(lens_id, "100.0"),
            SurfaceRefRow::Shared {
                target: detector_id,
                gap_after: GapRow::default(),
            }, // recombination
        ],
        stop_surface: Some(1),
        fields: fields(),
        field_mode: FieldMode::Angle,
        aperture_semi_diameter: "2.0".into(),
        wavelengths: vec!["0.6328".into()],
        beam_splitter_arms: vec![
            BeamSplitterPathKindRow::Reflecting,
            BeamSplitterPathKindRow::Transmitting,
        ],
    });

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
    fn wf_epi_microscope_cross_section_uses_3_rays() {
        assert_eq!(wf_epi_microscope().cross_section_n_rays, 3);
    }

    #[test]
    fn wf_epi_microscope_fields_are_point_sources() {
        let specs = wf_epi_microscope();
        for (i, path) in specs.paths.iter().enumerate() {
            assert_eq!(path.field_mode, FieldMode::PointSource, "path {i}");
        }
    }

    #[test]
    fn wf_epi_microscope_example_converts_to_valid_model() {
        let specs = wf_epi_microscope();
        let parsed = parse(&specs);
        SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .solves(parsed.solves)
            .build()
            .expect("model");
    }

    /// AT-8: the GUI Michelson converts to the library example's topology and
    /// geometry, store index for store index.
    #[test]
    fn michelson_interferometer_matches_library_example() {
        let parsed = parse(&michelson_interferometer());
        let model = SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .build()
            .expect("model")
            .model;
        let reference = crate::examples::michelson_interferometer::sequential_model(
            crate::n!(1.0),
            &[0.6328],
            100.0,
            110.0,
        );

        assert_eq!(model.path_count(), 2);
        assert_eq!(model.path_surface_indices(0), &[0, 1, 2, 3, 2, 4, 5]);
        assert_eq!(model.path_surface_indices(1), &[0, 1, 2, 6, 2, 4, 5]);
        // Skip the Object at infinity.
        for (gui, lib) in model
            .placements()
            .iter()
            .zip(reference.placements())
            .skip(1)
        {
            approx::assert_abs_diff_eq!(gui.position.x(), lib.position.x(), epsilon = 1e-9);
            approx::assert_abs_diff_eq!(gui.position.y(), lib.position.y(), epsilon = 1e-9);
            approx::assert_abs_diff_eq!(gui.position.z(), lib.position.z(), epsilon = 1e-9,);
        }
        for path in 0..2 {
            assert_eq!(model.stop_surface_for_path(path), Some(1), "path {path}");
        }
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
