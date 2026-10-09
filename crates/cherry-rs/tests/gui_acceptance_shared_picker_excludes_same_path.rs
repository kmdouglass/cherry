#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// VT-SURF-2: the Shared-surface picker never offers a row introduced by
/// the *same* path as a candidate (FR-SURF-3) — only strictly earlier
/// paths. `wf_epi_microscope`'s path 1 introduces its own fold mirror and
/// tube lens (both `New` rows); neither should ever appear in path 1's own
/// Shared-picker dropdown.
#[test]
fn shared_picker_excludes_candidates_from_the_same_path() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    harness.get_by_label("\u{25b6}").click();
    harness.run();
    wait_until(
        &mut harness,
        "switched to path 1",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 1"),
    );
    harness.get_by_label("Surfaces").click();
    harness.run();

    // Combo box #6 is path 1's row-2 Shared-surface picker — see the
    // sibling VT-SURF-1 test for how this index was derived.
    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(6)
        .unwrap()
        .click();
    harness.run();

    // Path 0's candidates are offered (sanity: the dropdown isn't just
    // empty).
    harness.get_by_label("Beam Splitter \u{2014} 0");

    // Path 1's own rows — its fold mirror (`Conic \u{2014} 1`) and tube
    // lens (`Thin Lens \u{2014} 1`) — are never offered, regardless of
    // which row's picker is open, since `shared_candidates` only ever
    // considers `specs.paths[..active_path]`.
    assert!(
        harness
            .query_all_by_label_contains("\u{2014} 1")
            .next()
            .is_none(),
        "the picker must not offer any candidate introduced by the same path"
    );
}
