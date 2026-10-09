#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// VT-SURF-1: the Shared-surface picker offers every surface introduced by
/// a strictly earlier path as a candidate, in no particular forced order —
/// `wf_epi_microscope`'s emission path (path 1) already picks the shared
/// objective (store index 3) at an earlier row than the shared beam
/// splitter (store index 2), demonstrating the library's out-of-order
/// flexibility. This test opens that same beam-splitter row's picker and
/// confirms the dropdown still lists the beam splitter as a candidate even
/// though a *higher* store index (the objective) was already picked at an
/// earlier row.
#[test]
fn shared_picker_lists_beam_splitter_despite_earlier_out_of_order_pick() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    // Switch to path 1 (emission) and open its Surfaces tab.
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

    // Combo box #6 is path 1's row-2 Shared-surface picker (the beam
    // splitter row, `Shared { target: RowId(2), .. }` in the fixture) — see
    // `gui_support`'s module doc for how indices like this were found by
    // dumping the real AccessKit tree (`println!("{:#?}", harness.root())`)
    // and matching each `ComboBox` node's rect to its row's `#` label rect.
    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(6)
        .unwrap()
        .click();
    harness.run();

    // The beam splitter (introduced by path 0, store index 2) is listed as
    // a candidate even though this same path already picked a *higher*
    // store index (the objective, store index 3) at the previous row.
    harness.get_by_label("Beam Splitter \u{2014} 0");
    // The objective is also still listed (candidates aren't removed once
    // picked elsewhere).
    harness
        .get_all_by_label("Thin Lens \u{2014} 0")
        .next()
        .unwrap();
}
