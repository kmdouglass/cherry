#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::{Role, Toggled};
use egui_kittest::{
    Harness,
    kittest::{By, NodeT, Queryable},
};
use gui_support::{has_label_contains, launch, wait_until};

fn path_checkbox_toggled(harness: &Harness<'_, cherry_rs::gui::CherryApp>, label: &str) -> bool {
    harness
        .get(By::new().role(Role::CheckBox).label(label))
        .accesskit_node()
        .toggled()
        == Some(Toggled::True)
}

/// FR-XS-7: a path hidden via the cross-section visibility checklist can be
/// made visible again by re-checking its box. Reproduces the reported bug:
/// with Path 1 active, unchecking it (which reassigns `active_path` to
/// Path 0) and then re-checking it must leave the box checked, rather than
/// snapping back to unchecked on the next frame.
#[test]
fn rechecking_hidden_path_makes_it_visible_again() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    // Make Path 1 active via the Specs window's switcher.
    harness.get_by_label("\u{25b6}").click();
    for _ in 0..5 {
        harness.step();
    }

    harness.get_by_label("Cross Section").click();
    harness.run();
    wait_until(
        &mut harness,
        "cross-section path checklist is populated",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 0") && has_label_contains(h, "Path 1"),
    );
    assert!(path_checkbox_toggled(&harness, "Path 1"));

    // Hide Path 1.
    harness
        .get(By::new().role(Role::CheckBox).label("Path 1"))
        .click();
    for _ in 0..5 {
        harness.step();
    }
    assert!(!path_checkbox_toggled(&harness, "Path 1"));

    // Show it again.
    harness
        .get(By::new().role(Role::CheckBox).label("Path 1"))
        .click();
    for _ in 0..5 {
        harness.step();
    }
    assert!(
        path_checkbox_toggled(&harness, "Path 1"),
        "Path 1 checkbox snapped back to unchecked after re-checking it"
    );
}
