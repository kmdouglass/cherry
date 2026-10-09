#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui_kittest::kittest::Queryable;
use gui_support::{has_label_contains, launch, wait_until};

/// Loading the "Thin Lens" example and opening the Paraxial Summary window
/// shows the known effective focal length (100mm, from the example's
/// `MarginalRayHeight` solve pinning the image plane at focus).
#[test]
fn load_thin_lens_shows_efl_in_paraxial_window() {
    let mut harness = launch();

    // Open the Examples menu and pick "Thin Lens" — the same click path a
    // user takes (see app.rs, menu_button("Examples", ...)).
    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Thin Lens").click();
    harness.run();

    // Open the Paraxial Summary window via the always-visible window list.
    harness.get_by_label("Paraxial Summary").click();
    harness.run();

    // Wait for the background compute thread to finish and the window to
    // render the expected row.
    wait_until(
        &mut harness,
        "Paraxial window shows a non-placeholder EFL value",
        Duration::from_secs(2),
        |h| {
            has_label_contains(h, "Effective focal length") && !has_label_contains(h, "No data yet")
        },
    );

    // Effective focal length for this fixture is exactly 100mm by
    // construction, formatted to 4 significant figures by `format_value`
    // (gui/windows/paraxial.rs). Front/back focal distances share the same
    // value for this symmetric fixture, so more than one row may show
    // "100.0000" — assert presence, not uniqueness.
    assert!(
        harness.get_all_by_label_contains("100.0000").count() > 0,
        "expected at least one row showing 100.0000"
    );
}
