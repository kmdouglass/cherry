#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui_kittest::kittest::Queryable;
use gui_support::{has_label_contains, has_value, launch, wait_until};

/// VT-NAV-5: loading a system with fewer paths than the current
/// `active_path` clamps `active_path` into range (FR-NAV-6). With path 1 of
/// `wf_epi_microscope` active, loading the single-path Thin Lens example
/// must show the Thin Lens's own path-0 data in the Specs tabs — not an
/// empty panel for a now-nonexistent path 1, and no panic.
#[test]
fn loading_single_path_example_while_path_1_active_clamps_active_path() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    harness.get_by_label("\u{25b6}").click();
    for _ in 0..5 {
        harness.step();
    }
    harness.get_by_label("Wavelengths").click();
    wait_until(
        &mut harness,
        "path 1's wavelength is shown",
        Duration::from_secs(2),
        |h| has_value(h, "0.520"),
    );

    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Thin Lens").click();
    for _ in 0..5 {
        harness.step();
    }

    wait_until(
        &mut harness,
        "Thin Lens path 0 wavelength is shown after loading",
        Duration::from_secs(2),
        |h| has_value(h, "0.5876"),
    );
    assert!(
        !has_label_contains(&harness, "Path 1"),
        "a single-path system must not show a Path 1 label"
    );
}
