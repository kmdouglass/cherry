use anyhow::{bail, Result};

use crate::vec3::Vec3;

pub(crate) struct Rays {
    pos: Vec<Vec3>,
    dir: Vec<Vec3>,
}

impl Rays {
    pub fn new(pos: Vec<Vec3>, dir: Vec<Vec3>) -> Result<Self> {
        if pos.len() != dir.len() {
            bail!("position and direction must have the same length");
        }

        // Verify that each direction is a unit vector
        for dir in &dir {
            if !dir.is_unit() {
                bail!("direction must be a unit vector");
            }
        }

        Ok(Self { pos, dir })
    }
}

#[cfg(test)]
mod test {
    // Test the constructor of Rays
    #[test]
    fn test_rays_new() {
        use super::*;

        let pos = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, -2.0, 0.0),
        ];
        let dir = vec![
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];

        let rays = Rays::new(pos, dir).unwrap();

        assert_eq!(rays.pos.len(), 3);
        assert_eq!(rays.dir.len(), 3);
    }

    // Test the constructor of Rays with mismatched lengths
    #[test]
    fn test_rays_new_mismatched_lengths() {
        use super::*;

        let pos = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, -2.0, 0.0),
        ];
        let dir = vec![Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0)];

        let rays = Rays::new(pos, dir);

        assert!(rays.is_err());
    }

    // Test the constructor of Rays with a non-unit direction
    #[test]
    fn test_rays_new_non_unit_direction() {
        use super::*;

        let pos = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, -2.0, 0.0),
        ];
        let dir = vec![
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];

        let rays = Rays::new(pos, dir);

        assert!(rays.is_err());
    }
}