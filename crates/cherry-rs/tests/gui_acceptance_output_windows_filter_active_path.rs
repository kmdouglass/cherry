#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui_kittest::kittest::Queryable;
use gui_support::{has_label_contains, launch, wait_until};

/// VT-OUT-1: Paraxial/RayFan/SpotDiagram show only the active path's data,
/// and switching `active_path` updates them. Exercised here with the
/// Paraxial Summary window, whose effective focal length differs sharply
/// between `wf_epi_microscope`'s excitation (path 0) and emission (path 1)
/// arms.
#[test]
fn paraxial_window_shows_only_active_paths_efl() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    harness.get_by_label("Paraxial Summary").click();
    harness.run();

    // Path 0 (excitation): EFL = -1.8750 mm.
    wait_until(
        &mut harness,
        "paraxial window shows path 0's EFL",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Effective focal length") && has_label_contains(h, "-1.8750"),
    );
    assert!(!has_label_contains(&harness, "12.4999"));

    // Switch to path 1 (emission): EFL = 12.4999 mm — a different value, not
    // a stale carry-over from path 0.
    harness.get_by_label("\u{25b6}").click();
    harness.run();

    wait_until(
        &mut harness,
        "paraxial window shows path 1's EFL",
        Duration::from_secs(2),
        |h| has_label_contains(h, "12.4999"),
    );
    assert!(!has_label_contains(&harness, "-1.8750"));
}
