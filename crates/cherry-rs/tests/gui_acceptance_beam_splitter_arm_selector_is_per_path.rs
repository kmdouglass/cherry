#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// VT-SURF-3: the beam-splitter arm selector is a per-path control — each
/// path's traversal of a shared `BeamSplitter` surface gets its own
/// `Transmitting`/`Reflecting` selector, independent of every other path's.
///
/// Note on what this test can and can't observe: egui's `ComboBox` paints
/// its *current* selection directly (a `Galley`, not an AccessKit-exposed
/// value — confirmed by dumping the real AccessKit tree, where a closed
/// `ComboBox` node has an empty `label` and no `value`), and its option
/// list's `selectable_value` items don't surface a `toggled`/`selected`
/// AccessKit attribute either (`egui::Button`'s `.selected()` only affects
/// paint style, not `WidgetInfo`). So this test can't read "which arm is
/// currently selected" for either path — it instead exercises the control
/// that FR-SURF-6 requires: both paths expose their *own* independent
/// Arm combo box (one per path's traversal of the shared beam splitter,
/// each reachable and clickable from that path's own Surfaces tab, scoped
/// to `wf_epi_microscope`'s known fixture: path 0 reflects, path 1
/// transmits — verified at the unit-test layer by
/// `gui/convert.rs::wf_epi_microscope_topology_round_trips` and
/// `at_beamsplitter_in_both_paths`), and that opening/clicking one has no
/// effect on the tab rendering for the other path.
#[test]
fn each_path_has_its_own_independently_clickable_arm_selector() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    // Path 0's beam-splitter row is the 3rd combo box on its Surfaces tab
    // (Variant combos for the relay lens and beam splitter, then the Arm
    // combo) — found by dumping the real tree and matching rects to row
    // labels, the same way as the sibling Shared-picker tests. It renders
    // far enough right in the table to be clipped out of the Specs
    // window's visible scroll area, so a plain `.click()` (which requires
    // the node's screen position to be actually hit-testable) silently
    // does nothing; `.click_accesskit()` dispatches an AccessKit
    // `Action::Click` directly and works regardless of scroll/clipping.
    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(2)
        .unwrap()
        .click_accesskit();
    harness.run();
    harness.get_by_label("Transmitting");
    harness.get_by_label("Reflecting");
    // Close the popup without changing anything, by picking the option
    // that mirrors the fixture's own path-0 default (Reflecting).
    harness.get_by_label("Reflecting").click_accesskit();
    harness.run();

    // Switch to path 1 and open its own Arm combo — the Shared row at
    // step 2 referencing the same physical beam splitter. This is combo
    // #7 on path 1's tab (see
    // `gui_acceptance_shared_picker_lists_out_of_order_targets.rs` for how
    // path 1's combo indices were derived; the Arm combo immediately
    // follows that row's Shared-target picker, combo #6).
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

    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(7)
        .unwrap()
        .click_accesskit();
    harness.run();
    // Path 1's own Arm combo offers the same two options, independent of
    // path 0's — clicking it (and path 0's, above) never errors or panics,
    // and each path's own Surfaces tab remains fully rendered afterward.
    harness.get_by_label("Transmitting");
    harness.get_by_label("Reflecting");
    harness.get_by_label("Transmitting").click_accesskit();
    harness.run();

    harness.get_by_label("Surfaces");
    harness.get_by_label("Fields");
}
