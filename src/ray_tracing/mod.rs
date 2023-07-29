pub mod rays;
pub mod sequential_model;
pub mod trace;

use anyhow::{bail, Result};

use sequential_model::{Gap, Surface};

struct SystemModel {
    gaps: Vec<Gap>,
    surfaces: Vec<Surface>,
}

impl SystemModel {
    fn new() -> Self {
        let obj_plane = Surface::new_obj_or_img_plane(0.0, 1.0);
        let obj_gap = Gap::new(1.0, 1.0);
        let img_plane = Surface::new_obj_or_img_plane(1.0, 1.0);

        let mut gaps = Vec::new();
        let mut surfaces = Vec::new();

        gaps.push(obj_gap);
        surfaces.push(obj_plane);
        surfaces.push(img_plane);

        Self {
            gaps,
            surfaces,
        }
    }

    fn add_surface_and_gap(&mut self, idx: usize, surface: Surface, gap: Gap) -> Result<()> {
        if idx == 0 {
            bail!("Cannot add surface before the object plane.");
        }

        if idx > self.surfaces.len() - 1 {
            bail!("Cannot add surface after the image plane.");
        }

        self.surfaces.insert(idx, surface);
        self.gaps.insert(idx, gap);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_system_model() {
        let mut model = SystemModel::new();

        let surf = Surface::new_refr_circ_conic(0.0, 1.0, 1.0, 1.0, 0.0);
        let gap = Gap::new(1.0, 1.0);
        model.add_surface_and_gap(1, surf, gap).unwrap();

        assert_eq!(model.surfaces.len(), 3);
        assert_eq!(model.gaps.len(), 2);
    }

    #[test]
    fn test_system_model_insert_surface_at_object_plane() {
        let mut model = SystemModel::new();

        let surf = Surface::new_refr_circ_conic(0.0, 1.0, 1.0, 1.0, 0.0);
        let gap = Gap::new(1.0, 1.0);
        let res = model.add_surface_and_gap(0, surf, gap);

        assert!(res.is_err());
    }

    #[test]
    fn test_system_model_insert_surface_at_image_plane() {
        let mut model = SystemModel::new();

        let surf = Surface::new_refr_circ_conic(0.0, 1.0, 1.0, 1.0, 0.0);
        let gap = Gap::new(1.0, 1.0);
        let res = model.add_surface_and_gap(2, surf, gap);

        assert!(res.is_err());
    }
}