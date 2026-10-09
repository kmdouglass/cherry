#![cfg(feature = "gui")]

mod gui_support;

use egui_kittest::kittest::{NodeT, Queryable};
use gui_support::launch;

/// VT-NAV-2: "Remove Path" is disabled while path 0 is active, even on a
/// multipath system (path 0 can never be removed).
#[test]
fn remove_path_disabled_while_path_zero_is_active() {
    let mut harness = launch();

    // Create a second path (switches active_path to it, per VT-NAV-1).
    harness.get_by_label("+ Add Path").click();
    harness.run();

    // Switch back to path 0 with the "previous" arrow.
    harness.get_by_label("\u{25c0}").click();
    harness.run();
    harness.get_by_label_contains("Path 0");

    let remove_button = harness.get_by_label("Remove Path");
    assert!(
        remove_button.accesskit_node().is_disabled(),
        "Remove Path must be disabled while path 0 is active"
    );
}
