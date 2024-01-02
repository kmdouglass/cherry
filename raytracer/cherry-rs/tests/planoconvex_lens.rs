use cherry_rs::{ApertureSpec, FieldSpec, Gap, PupilSampling, SurfaceSpec, SystemBuilder, SystemModel};
use cherry_rs::trace::trace;

/// A f = +50.1 mm planoconvex lens: https://www.thorlabs.com/thorproduct.cfm?partnumber=LA1255
///
/// The object is at infinity; aperture stop is the first surface.
/// There are two fields: 0 and +5 degrees.
fn planoconvex_lens_obj_at_inf() -> (SystemModel, SystemBuilder) {
    let surf_0 = SurfaceSpec::ObjectPlane { diam: 25.0 };
    let gap_0 = Gap::new(1.0, f32::INFINITY);
    let surf_1 = SurfaceSpec::RefractingCircularConic {
        diam: 25.0,
        roc: 25.8,
        k: 0.0,
    };
    let gap_1 = Gap::new(1.515, 5.3);
    let surf_2 = SurfaceSpec::RefractingCircularFlat { diam: 25.0 };
    let gap_2 = Gap::new(1.0, 46.6);
    let surf_3 = SurfaceSpec::ImagePlane { diam: 25.0 };

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3];
    let gaps = vec![gap_0, gap_1, gap_2];
    let aperture = ApertureSpec::EntrancePupilDiameter { diam: 25.0 };

    let pupil_sampling = PupilSampling::SqGrid { spacing: 0.1 };
    let fields = vec![
        FieldSpec::new_field_angle(0.0, 0.5867, pupil_sampling),
        FieldSpec::new_field_angle(5.0, 0.5867, pupil_sampling),
    ];

    let mut builder = SystemBuilder::new();
    let model = builder
        .surfaces(surfaces)
        .gaps(gaps)
        .aperture(aperture)
        .fields(fields)
        .build()
        .unwrap();

    (model, builder)
}

#[test]
fn test_planoconvex_lens_obj_at_inf() {
    let (model, _) = planoconvex_lens_obj_at_inf();
    let surfaces = model.surf_model().surfaces();

    let rays = model.rays(None).expect("Failed to generate rays");
    //println!("Rays: {:?}", rays.len());
    //let results = trace(surfaces, rays);
}
