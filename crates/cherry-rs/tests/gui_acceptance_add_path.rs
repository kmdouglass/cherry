#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui_kittest::kittest::Queryable;
use gui_support::{has_label_contains, launch, wait_until};

/// VT-NAV-1: clicking "+ Add Path" on a single-path system creates a second
/// path and switches to it. The Specs window (where the path switcher
/// lives) is open by default (`WindowVisibility::default().specs == true`),
/// so no window needs to be opened first.
#[test]
fn add_path_creates_second_path_and_switches_to_it() {
    let mut harness = launch();

    // Single-path system: the switcher shows "Path 0" without arrows (only
    // appear once `paths.len() > 1`), and "+ Add Path" is always present.
    harness.get_by_label_contains("Path 0");
    harness.get_by_label("+ Add Path").click();
    harness.run();

    wait_until(
        &mut harness,
        "switcher shows the newly created Path 1 as active",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 1"),
    );
}
