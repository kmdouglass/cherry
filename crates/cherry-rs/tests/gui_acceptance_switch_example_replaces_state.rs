#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui_kittest::kittest::Queryable;
use gui_support::{has_label_contains, launch, wait_until};

/// Loading a second example through the `Examples` menu replaces the first
/// example's computed results in an already-open window, rather than
/// leaving stale data visible (REQ-GAT-06 scenario 4) — guards against the
/// `latest_result`/`input_id` staleness class of bug `app.rs` itself calls
/// out in comments on `active_path`/result validation.
#[test]
fn switching_examples_replaces_paraxial_window_contents() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Thin Lens").click();
    harness.run();

    harness.get_by_label("Paraxial Summary").click();
    harness.run();
    wait_until(
        &mut harness,
        "Paraxial window shows Thin Lens's EFL (100)",
        Duration::from_secs(2),
        |h| has_label_contains(h, "100.0000"),
    );

    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Biconvex Lens").click();
    harness.run();

    // Biconvex Lens's EFL (~99.6297mm) deliberately differs from Thin
    // Lens's exact 100.0000mm, so a stale-result bug (old value lingering)
    // is distinguishable from a correct re-render.
    wait_until(
        &mut harness,
        "Paraxial window no longer shows Thin Lens's stale EFL (100) after switching to Biconvex Lens",
        Duration::from_secs(2),
        |h| !has_label_contains(h, "100.0000") && has_label_contains(h, "Effective focal length"),
    );
}
