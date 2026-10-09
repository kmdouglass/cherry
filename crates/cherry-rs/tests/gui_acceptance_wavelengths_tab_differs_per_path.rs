#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui_kittest::kittest::Queryable;
use gui_support::{has_value, launch, wait_until};

/// VT-TAB-1: the Fields/Aperture/Wavelengths tabs show only the active
/// path's own data, and switching `active_path` updates them. Exercised
/// here with the Wavelengths tab, whose `SpinButton` value differs between
/// `wf_epi_microscope`'s excitation (path 0, "0.488") and emission (path 1,
/// "0.520") arms — AccessKit exposes this as the node's `value`, not its
/// `label` (confirmed by dumping the real tree), so this test checks
/// `value` rather than reusing `has_label_contains`.
#[test]
fn wavelengths_tab_shows_only_active_paths_values() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    harness.get_by_label("Wavelengths").click();
    harness.run();

    // Path 0 (excitation): wavelength 0.488.
    wait_until(
        &mut harness,
        "wavelengths tab shows path 0's value",
        Duration::from_secs(2),
        |h| has_value(h, "0.488"),
    );
    assert!(!has_value(&harness, "0.520"));

    // Switch to path 1 (emission): wavelength 0.520 — a different value,
    // not a stale carry-over from path 0. A plain `.run()` here can exceed
    // `max_steps` while the compute thread's spinner is still mid-repaint
    // from the path switch; a few plain `step()`s are enough to let it
    // settle.
    harness.get_by_label("\u{25b6}").click();
    for _ in 0..5 {
        harness.step();
    }

    wait_until(
        &mut harness,
        "wavelengths tab shows path 1's value",
        Duration::from_secs(2),
        |h| has_value(h, "0.520"),
    );
    assert!(!has_value(&harness, "0.488"));
}
