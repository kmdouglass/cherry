#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// VT-NAME-1: default path names show as "Path N" in the switcher and the
/// cross-section visibility checklist; renaming a path updates both.
#[test]
fn renaming_a_path_updates_switcher_and_cross_section_checklist() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    // Default names: the switcher (Specs window, open by default) shows
    // "Path 0" for the active path.
    harness.get(By::new().role(Role::Label).label_contains("Path 0"));

    harness.get_by_label("Cross Section").click();
    harness.run();
    wait_until(
        &mut harness,
        "cross-section path checklist is populated with default names",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 0") && has_label_contains(h, "Path 1"),
    );

    // Rename the active path (path 0) via the Specs window's "Name:" field.
    harness.get(By::new().role(Role::TextInput)).click();
    harness.run();
    harness
        .get(By::new().role(Role::TextInput))
        .type_text("Excitation");
    harness.run();

    // The switcher immediately reflects the new name instead of "Path 0".
    wait_until(
        &mut harness,
        "switcher shows the custom name",
        Duration::from_secs(2),
        |h| {
            h.query(By::new().role(Role::Label).label_contains("Excitation"))
                .is_some()
        },
    );
    assert!(
        harness
            .query(By::new().role(Role::Label).label_contains("Path 0"))
            .is_none(),
        "switcher should no longer show the default label once renamed"
    );

    // The cross-section visibility checklist also shows the new name.
    wait_until(
        &mut harness,
        "cross-section checklist shows the custom name",
        Duration::from_secs(2),
        |h| {
            h.query(By::new().role(Role::CheckBox).label("Excitation"))
                .is_some()
        },
    );
}
