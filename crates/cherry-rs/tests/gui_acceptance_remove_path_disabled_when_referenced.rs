#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, NodeT, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// VT-NAV-3: Remove Path is disabled when a later path `Shared`-references
/// it, distinct from VT-NAV-2's "path 0 can never be removed" case. Builds
/// a 3-path system on top of `wf_epi_microscope` (paths 0, 1): a new path 2
/// is created and its single surface row is switched to `Shared`,
/// targeting path 1's own tube lens — after which path 1 can no longer be
/// removed, even though nothing referenced it before path 2 was added.
#[test]
fn remove_path_disabled_once_a_later_path_shares_one_of_its_rows() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    // Add path 2 and give it a second (non-Object/Image) row to switch to
    // Shared, via the same "+" insert-after control the Surfaces tab
    // already uses for every path.
    harness.get_by_label("+ Add Path").click();
    harness.run();
    wait_until(
        &mut harness,
        "path 2 created",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 2"),
    );
    harness.get_by_label("Surfaces").click();
    harness.run();
    harness.get_all_by_label("+").next().unwrap().click();
    harness.run();

    // Row 0 = Object (locked, Ref-mode combo only). Row 1 (now a default
    // `New` Sphere) gets combo #1 (Ref-mode) and combo #2 (Variant) — see
    // the sibling Shared-picker tests for how these indices were derived
    // by dumping the real AccessKit tree.
    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(1)
        .unwrap()
        .click();
    harness.run();
    harness.get_by_label("Shared").click();
    harness.run();

    // Row 1 is now `Shared`; its target picker (still combo #2) defaults
    // to the first candidate. Re-open it and pick path 1's tube lens
    // specifically, so path 2 ends up referencing path 1 rather than path
    // 0.
    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(2)
        .unwrap()
        .click();
    harness.run();
    harness.get_by_label("Thin Lens \u{2014} 1").click();
    // A `.run()` here can exceed `max_steps` while the compute thread's
    // spinner is still mid-repaint from the earlier edits; a few plain
    // `step()`s are enough to let the click's `changed()` propagate.
    for _ in 0..5 {
        harness.step();
    }

    // Navigate back to path 1: Remove Path is now disabled, because path
    // 2 references one of its rows.
    harness.get_by_label("\u{25c0}").click();
    harness.run();
    wait_until(
        &mut harness,
        "back on path 1",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 1"),
    );
    assert!(
        harness
            .get_by_label("Remove Path")
            .accesskit_node()
            .is_disabled(),
        "path 1 should not be removable once path 2 shares one of its rows"
    );
}
