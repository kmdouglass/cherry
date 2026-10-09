#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// VT-XS-6: unchecking the active path directly in the cross-section
/// window's visibility checklist reassigns `active_path` to the only other
/// visible path, observable via the Specs window's path switcher (role
/// `Label`, distinct from the identically-named `CheckBox` in the
/// checklist) flipping to show the new active path — not via any private
/// `CherryApp` field.
#[test]
fn unchecking_active_path_reassigns_active_path() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    harness.get_by_label("Cross Section").click();
    harness.run();
    wait_until(
        &mut harness,
        "cross-section path checklist is populated",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 0") && has_label_contains(h, "Path 1"),
    );

    // Path 0 is active by default; the switcher shows it.
    harness.get(By::new().role(Role::Label).label_contains("Path 0"));

    // Uncheck path 0 directly in the visibility checklist.
    harness
        .get(By::new().role(Role::CheckBox).label("Path 0"))
        .click();
    harness.run();

    wait_until(
        &mut harness,
        "path switcher shows Path 1 as active after Path 0 was hidden",
        Duration::from_secs(2),
        |h| {
            h.query(By::new().role(Role::Label).label_contains("Path 1"))
                .is_some()
        },
    );
}
