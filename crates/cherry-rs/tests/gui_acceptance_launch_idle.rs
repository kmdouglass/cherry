#![cfg(feature = "gui")]

mod gui_support;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, Queryable};
use gui_support::launch;

/// The app constructs, renders a frame, and shows the top menu bar and the
/// window-visibility list without panicking (REQ-GAT-06 scenario 1).
#[test]
fn app_launches_with_menu_bar_and_window_list() {
    let harness = launch();

    harness.get_by_label("File");
    harness.get_by_label("Examples");
    harness.get_by_label("Help");

    // The window-visibility list (right-hand side panel) is always present.
    // "Specs" is queried by (Button, label) rather than plain label, because
    // the Specs window is open by default (`WindowVisibility::default()`),
    // so a plain label query would also match the open window itself.
    harness.get(By::new().role(Role::Button).label("Specs"));
    harness.get_by_label("Paraxial Summary");
    harness.get_by_label("Spot Diagram");
    harness.get_by_label("Cross Section");
    harness.get_by_label("Ray Fan Plot");
}
