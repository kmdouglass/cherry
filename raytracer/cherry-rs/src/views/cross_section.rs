/// A 2D cross section view through the optical system.
use anyhow::Result;

use crate::core::{
    math::{
        geometry::surfaces::{
            GeometricSurface, parametric_plane::ParametricPlane, quadric::Quadric,
        },
        vec3::Vec3,
    },
    sequential_model::{Axis, SequentialModel, Surface},
};

// The orientation of the cutting plane in the global reference system.
#[derive(Debug)]
pub enum CuttingPlane {
    XY,
    XZ,
    YZ,
}

pub type Path = Vec<Vec3>;
pub type Paths = Vec<Path>;

pub fn cross_section_view(
    sequential_model: &SequentialModel,
    cutting_plane: CuttingPlane,
    num_samples: usize,
) -> Result<Paths> {
    let num_surfaces = sequential_model.surfaces().len();
    let mut paths = Vec::with_capacity(num_surfaces);

    for surf in sequential_model.surfaces() {
        // Skip object or image planes at infinity
        match surf {
            Surface::Object(_) | Surface::Image(_) => {
                if surf.is_infinite() {
                    paths.push(Path::default());
                    continue;
                }
            }
            _ => {}
        }

        let geometric_surface = surf.to_geometric_surface()?;

        // Transform the cutting plane into the local CRS
        let mut parametric_plane = cutting_plane.to_parametric_plane()?;
        parametric_plane.rotate(surf.rot_mat());
        parametric_plane.translate(surf.pos());

        // Compute the intersection curve between the surface and the cutting plane

        // Sample from the curve to create a Path
        todo!("Implement cross section view generation");
    }

    Ok(paths)
}

impl CuttingPlane {
    fn to_parametric_plane(&self) -> Result<ParametricPlane> {
        match self {
            Self::XY => ParametricPlane::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            Self::XZ => ParametricPlane::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            Self::YZ => ParametricPlane::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ),
        }
    }
}

impl Surface {
    fn to_geometric_surface(&self) -> Result<GeometricSurface> {
        let k = self.conic_constant().unwrap_or(0.0);
        let roc = self.roc(&Axis::Y); // Assume rotational symmetry for now
        match self {
            Self::Conic(_s) => {
                let a = 1.0;
                let b = 1.0;
                let c = k + 1.0;
                let i = -2.0 * roc;

                Ok(GeometricSurface::Quadric(Quadric::new(
                    a, b, c, 0.0, 0.0, 0.0, 0.0, 0.0, i, 0.0,
                )))
            }
            Self::Object(_) | Self::Image(_) | Self::Probe(_) | Self::Stop(_) => {
                // The plane is just the z=0 plane in the local CRS
                Ok(GeometricSurface::ParametricPlane(ParametricPlane::new(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                )?))
            }
        }
    }
}
