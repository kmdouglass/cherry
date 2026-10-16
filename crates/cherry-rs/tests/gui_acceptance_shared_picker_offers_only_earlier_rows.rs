#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{By, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

use cherry_rs::gui::CherryApp;

/// Load the Michelson example and open path 1's ("Arm 2 (M2)") Surfaces tab.
fn michelson_path_1_surfaces() -> Harness<'static, CherryApp> {
    let mut harness = launch();
    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Michelson Interferometer").click();
    harness.run();

    harness.get_by_label("\u{25b6}").click();
    harness.run();
    wait_until(
        &mut harness,
        "switched to path 1",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Arm 2 (M2)"),
    );
    harness.get_by_label("Surfaces").click();
    harness.run();
    harness
}

/// Open the `nth` combo box on the current Surfaces tab. Indices were
/// derived by dumping the AccessKit tree and matching combo rects to table
/// rows: on the Michelson's path 1 every row has a Ref combo, Shared rows
/// add a target picker, beam-splitter rows add an Arm combo, and M2 (row 3)
/// adds Variant and Kind combos. Row 2's target picker is combo #5 and row
/// 4's is combo #11.
fn open_combo(harness: &mut Harness<'static, CherryApp>, nth: usize) {
    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(nth)
        .unwrap()
        .click_accesskit();
    harness.run();
}

/// VT-SURF-2 (amended by FR-7, `20261016_michelson_interferometer_
/// requirements.md`): the Shared-surface picker offers rows introduced at an
/// *earlier* step of the same path. Path 1's row 4 revisits the beam
/// splitter after path 1's own M2 (row 3), so M2 is a candidate.
#[test]
fn shared_picker_offers_earlier_rows_of_the_same_path() {
    let mut harness = michelson_path_1_surfaces();
    open_combo(&mut harness, 11);

    harness.get_by_label("(1, 3) Conic");
    harness.get_by_label("(0, 2) Beam Splitter");
    // Boundary surfaces never sit mid-path (FR-9).
    for boundary in [") Object", ") Image"] {
        assert!(
            harness
                .query_all_by_label_contains(boundary)
                .next()
                .is_none(),
            "an interior row's picker must not offer {boundary:?} surfaces"
        );
    }
}

/// FR-7: rows at the same or a later step of the same path are never
/// offered. Path 1's row 2 comes before M2 (row 3), so M2 is not a
/// candidate there.
#[test]
fn shared_picker_excludes_later_rows_of_the_same_path() {
    let mut harness = michelson_path_1_surfaces();
    open_combo(&mut harness, 5);

    harness.get_by_label("(0, 2) Beam Splitter");
    assert!(
        harness.query_all_by_label_contains("(1, ").next().is_none(),
        "the picker must not offer the same path's later rows"
    );
}
