#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// Editing the Thin Lens example's focal length in the (default-open) Specs
/// window propagates through the background compute thread into the
/// Paraxial Summary window's displayed EFL (REQ-GAT-06 scenario 3) — proof
/// that `bump_input_id` -> channel -> `result_rx` -> re-render works, not
/// just that the initial load works.
#[test]
fn editing_focal_length_updates_paraxial_efl() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Thin Lens").click();
    harness.run();

    harness.get_by_label("Paraxial Summary").click();
    harness.run();
    wait_until(
        &mut harness,
        "Paraxial window shows the initial EFL (100)",
        Duration::from_secs(2),
        |h| has_label_contains(h, "100.0000"),
    );

    // The Specs window's Surfaces tab is open by default
    // (`WindowVisibility::default().specs == true`); the focal-length
    // DragValue has no accessible label of its own (it's a bare table
    // cell), so it's located by its current numeric value instead.
    harness
        .get(By::new().role(Role::SpinButton).value("100"))
        .click();
    harness.run();
    harness
        .get(By::new().role(Role::SpinButton).value("100"))
        .type_text("50");
    harness.run();
    harness.key_press(egui::Key::Tab);
    harness.run();

    wait_until(
        &mut harness,
        "Paraxial window shows the updated EFL (50) and no longer shows 100",
        Duration::from_secs(2),
        |h| has_label_contains(h, "50.0000") && !has_label_contains(h, "100.0000"),
    );
}
