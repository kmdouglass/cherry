#![cfg(feature = "gui")]

mod gui_support;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, NodeT, Queryable};
use gui_support::launch;

/// AT-14 (FR-14): a row that another row references via `Shared` can't
/// change its ref mode, since that would leave the referrer dangling. In the
/// Michelson example, path 0's beam splitter (row 2) is referenced by its
/// own revisit row and by path 1, so its Ref combo is disabled; M1 (row 3)
/// is referenced by nothing, so its Ref combo is enabled.
#[test]
fn referenced_rows_have_a_disabled_ref_mode_selector() {
    let mut harness = launch();
    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Michelson Interferometer").click();
    harness.run();
    harness.get_by_label("Surfaces").click();
    harness.run();

    // Path 0 combo order (from the AccessKit tree): #0 Iris Variant; #1 BS
    // Ref, #2 BS Variant, #3 BS Arm; #4 M1 Ref, #5 M1 Variant, #6 M1 Kind;
    // #7 revisit Ref, #8 revisit target, #9 revisit Arm; #10 lens Ref, #11
    // lens Variant. Object, Iris and Detector rows show their Ref mode as a
    // plain label (no other option is valid).
    let disabled: Vec<bool> = harness
        .get_all(By::new().role(Role::ComboBox))
        .map(|c| c.accesskit_node().is_disabled())
        .collect();
    assert!(
        disabled[1],
        "the beam splitter's Ref combo must be disabled"
    );
    assert!(!disabled[4], "M1's Ref combo must be enabled");
    assert!(disabled[10], "the shared lens's Ref combo must be disabled");
}
