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
}