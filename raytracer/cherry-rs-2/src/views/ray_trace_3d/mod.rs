/// Performs a 3D ray trace on the system.
mod rays;
mod trace;

use std::collections::HashMap;

use anyhow::Result;

use crate::{
    core::{
        sequential_model::{SequentialModel, SequentialSubModel, SubModelID, Surface},
        Float, PI,
    },
    specs::{
        aperture::ApertureSpec,
        fields::{FieldSpec, PupilSampling},
    },
    Pupil,
};

use rays::Ray;
use trace::{trace, TraceResults};

use super::paraxial::{self, ParaxialSubView, ParaxialView};

#[derive(Debug)]
pub struct RayTrace3DView {
    aperture_spec: ApertureSpec,
    fields: Vec<FieldSpec>,

    subviews: HashMap<SubModelID, RayTrace3DSubView>,
}

#[derive(Debug)]
pub struct RayTrace3DSubView {
    results: TraceResults,
}

impl RayTrace3DView {
    pub fn new(
        aperture_spec: ApertureSpec,
        field_specs: Vec<FieldSpec>,
        sequential_model: &SequentialModel,
        paraxial_view: ParaxialView,
    ) -> Self {
        let subviews = sequential_model
            .submodels()
            .iter()
            .map(|(id, submodel)| {
                let surfaces = sequential_model.surfaces();
                let paraxial_sub_view = paraxial_view.subviews.get(id).unwrap();
                (
                    *id,
                    RayTrace3DSubView::new(
                        &aperture_spec,
                        &field_specs,
                        submodel,
                        surfaces,
                        paraxial_sub_view,
                    ),
                )
            })
            .collect();

        Self {
            aperture_spec,
            fields: field_specs,

            subviews,
        }
    }
}

impl RayTrace3DSubView {
    pub fn new(
        aperture_spec: &ApertureSpec,
        field_specs: &[FieldSpec],
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        paraxial_sub_view: &ParaxialSubView,
    ) -> Self {
        let rays = Self::rays(
            aperture_spec,
            sequential_sub_model,
            surfaces,
            paraxial_sub_view,
            field_specs,
            None,
        )
        .unwrap();

        let mut sequential_sub_model_iter = sequential_sub_model.iter(surfaces);
        let results = trace(&mut sequential_sub_model_iter, rays);

        Self { results }
    }

    /// Returns the rays to trace through the system as defined by the fields.
    ///
    /// # Arguments
    ///
    /// * `sampling` - The pupil sampling method. This will override the
    ///   sampling method specified
    ///  in the field specs for every field if provided.
    fn rays(
        aperture_spec: &ApertureSpec,
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        paraxial_sub_view: &ParaxialSubView,
        field_specs: &[FieldSpec],
        sampling: Option<PupilSampling>,
    ) -> Result<Vec<Ray>> {
        let mut rays = Vec::new();

        for (field_id, field) in field_specs.iter().enumerate() {
            match field {
                FieldSpec::Angle {
                    angle,
                    pupil_sampling,
                } => {
                    let angle = angle.to_radians();

                    let pupil_sampling = match sampling {
                        Some(sampling) => sampling,
                        None => *pupil_sampling,
                    };

                    let rays_field = match pupil_sampling {
                        PupilSampling::SquareGrid { spacing } => Self::pupil_ray_sq_grid(
                            aperture_spec,
                            sequential_sub_model,
                            surfaces,
                            paraxial_sub_view,
                            spacing,
                            angle,
                            field_id,
                        )?,
                        PupilSampling::ChiefMarginalRays => {
                            // 3 rays -> two diametrically-opposed marginal rays at the pupil edge
                            // and a chief ray in the center
                            Self::pupil_ray_fan(
                                aperture_spec,
                                sequential_sub_model,
                                surfaces,
                                paraxial_sub_view,
                                3,
                                PI / 2.0,
                                angle,
                                field_id,
                            )?
                        }
                    };

                    rays.extend(rays_field);
                }
                _ => unimplemented!(),
            }
        }

        Ok(rays)
    }

    /// Create a linear ray fan that passes through the entrance pupil.
    ///
    /// # Arguments
    ///
    /// * `num_rays` - The number of rays in the fan.
    /// * `theta` - The polar angle of the ray fan in the x-y plane.
    /// * `phi` - The angle of the ray w.r.t. the z-axis.
    /// * `field_id` - The ID of the field.
    fn pupil_ray_fan(
        aperture_spec: &ApertureSpec,
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        paraxial_sub_view: &ParaxialSubView,
        num_rays: usize,
        theta: Float,
        phi: Float,
        field_id: usize,
    ) -> Result<Vec<Ray>> {
        let ep = Self::entrance_pupil(
            aperture_spec,
            sequential_sub_model,
            surfaces,
            paraxial_sub_view,
        )?;
        let obj_z = surfaces[0].pos().z();
        let sur_z = surfaces[1].pos().z();
        let enp_z = ep.pos().z();

        let launch_point_z = Self::axial_launch_point(obj_z, sur_z, enp_z);

        // Determine the radial distance from the axis at the launch point for the
        // center of the ray fan.
        let dz = enp_z - launch_point_z;
        let dy = -dz * phi.tan();

        let rays = Ray::fan(
            num_rays,
            ep.semi_diameter,
            theta,
            launch_point_z,
            phi,
            0.0,
            dy,
            field_id,
        );

        Ok(rays)
    }

    /// Create a square grid of rays that passes through the entrance pupil.
    ///
    /// # Arguments
    ///
    /// * `spacing` - The spacing between rays in the grid in normalized pupil
    ///   distances, i.e. [0, 1]. A spacing of 1.0 means that one ray will lie
    ///   at the pupil center (the chief ray) and the others will lie at the
    ///   pupil edge (marginal rays).
    /// * `phi` - The angle of the ray w.r.t. the z-axis in radians.
    /// * `field_id` - The field ID.
    fn pupil_ray_sq_grid(
        aperture_spec: &ApertureSpec,
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        paraxial_sub_view: &ParaxialSubView,
        spacing: Float,
        phi: Float,
        field_id: usize,
    ) -> Result<Vec<Ray>> {
        let ep = Self::entrance_pupil(
            aperture_spec,
            sequential_sub_model,
            surfaces,
            paraxial_sub_view,
        )?;
        let obj_z = surfaces[0].pos().z();
        let sur_z = surfaces[1].pos().z();
        let enp_z = ep.pos().z();

        let launch_point_z = Self::axial_launch_point(obj_z, sur_z, enp_z);

        let enp_diam = ep.semi_diameter;
        let abs_spacing = enp_diam / 2.0 * spacing;

        // Determine the radial distance from the axis at the launch point for the
        // center of the ray fan.
        let dz = enp_z - launch_point_z;
        let dy = -dz * phi.tan();

        let rays = Ray::sq_grid_in_circ(
            enp_diam / 2.0,
            abs_spacing,
            launch_point_z,
            phi,
            0.0,
            dy,
            field_id,
        );

        Ok(rays)
    }

    /// Determines the entrance pupil of the subview.
    fn entrance_pupil(
        aperture_spec: &ApertureSpec,
        sequential_sub_model: &SequentialSubModel,
        surfaces: &[Surface],
        paraxial_sub_view: &ParaxialSubView,
    ) -> Result<Pupil> {
        let semi_diameter = match aperture_spec {
            ApertureSpec::EntrancePupil { semi_diameter } => *semi_diameter,
        };

        let entrance_pupil = paraxial_sub_view.entrance_pupil(sequential_sub_model, surfaces)?;
        let z = entrance_pupil.pos().z();

        Ok(Pupil {
            location: z,
            semi_diameter,
        })
    }

    /// Determine the axial launch point for the rays.
    ///
    /// If the object plane is at infinity, and if the first surface lies before
    /// the entrance pupil, then launch the rays from one unit to the left
    /// of the first surface. If the object plane is at infinity, and if it
    /// comes after the entrance pupil, then launch the rays from
    /// one unit in front of the entrance pupil. Otherwise, launch the rays from
    /// the object plane.
    fn axial_launch_point(obj_z: Float, sur_z: Float, enp_z: Float) -> Float {
        if obj_z == Float::NEG_INFINITY && sur_z <= enp_z {
            sur_z - 1.0
        } else if obj_z == Float::NEG_INFINITY && sur_z > enp_z {
            enp_z - 1.0
        } else {
            obj_z
        }
    }
}
