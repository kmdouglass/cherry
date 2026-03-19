mod cross_section;
#[cfg(feature = "ri-info")]
mod materials;
mod paraxial;
mod specs;
mod spot_diagram;
mod system;

pub use cross_section::{CrossSectionWindow, CuttingPlane};
#[cfg(feature = "ri-info")]
pub use materials::MaterialsWindow;
pub use paraxial::ParaxialWindow;
pub use specs::SpecsWindow;
pub use spot_diagram::SpotDiagramWindow;
pub use system::SystemWindow;

/// Controls which floating windows are currently open.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct WindowVisibility {
    pub specs: bool,
    pub materials: bool,
    pub paraxial_summary: bool,
    pub spot_diagram: bool,
    pub cross_section: bool,
    pub system: bool,
}

impl Default for WindowVisibility {
    fn default() -> Self {
        Self {
            specs: true,
            materials: false,
            paraxial_summary: false,
            spot_diagram: false,
            cross_section: false,
            system: false,
        }
    }
}
