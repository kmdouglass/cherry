//! Shared scaffolding for GUI acceptance tests (`tests/gui_acceptance_*.rs`).
//!
//! To add a new acceptance test: call [`launch`] to get a running `CherryApp`
//! harness, drive it the way a user would (`harness.get_by_label(...).click()`,
//! `harness.run()`), then use [`wait_until`] to block until the background
//! compute thread has produced the expected result before asserting on it.
//! Never reach into `cherry_rs::gui` internals — only the public `CherryApp`
//! / AccessKit surface a real user has (see
//! `.private/docs/gui/20260929_gui_acceptance_testing_requirements.md`,
//! REQ-GAT-11).
#![cfg(feature = "gui")]
// Each `tests/gui_acceptance_*.rs` file compiles this module as its own
// separate binary and uses only a subset of its helpers; `dead_code` would
// otherwise fire per-binary for whichever helpers that particular test
// doesn't need.
#![allow(dead_code)]

use std::time::{Duration, Instant};

use cherry_rs::gui::CherryApp;
use egui_kittest::Harness;

/// Construct a real `CherryApp` behind a headless `egui_kittest` harness.
///
/// Spawns the same background compute thread the live app uses; callers must
/// use [`wait_until`] rather than a single `step()`/`run()` before asserting
/// on any value that depends on computation, since nothing ties the compute
/// thread's completion to egui's own repaint signal.
pub fn launch() -> Harness<'static, CherryApp> {
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(1280.0, 720.0))
        .build_eframe(|cc| CherryApp::new(cc));
    harness.run();
    harness
}

/// Step the harness until `predicate` returns `true`, or panic after
/// `timeout` naming what was being waited for.
///
/// Polls frequently (every 5ms) rather than sleeping for the full timeout,
/// so passing tests aren't slowed down.
#[track_caller]
pub fn wait_until(
    harness: &mut Harness<'_, CherryApp>,
    what: &str,
    timeout: Duration,
    mut predicate: impl FnMut(&Harness<'_, CherryApp>) -> bool,
) {
    let deadline = Instant::now() + timeout;
    loop {
        harness.step();
        if predicate(harness) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("timed out after {timeout:?} waiting for: {what}");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// True if a node whose accessible name contains `text` currently exists.
///
/// Uses `query_all_by_label_contains` rather than the singular
/// `query_by_label_contains`, which panics (not just returns `None`) when
/// more than one node matches — a common case here, since e.g. a symmetric
/// thin lens shows the same formatted value in more than one paraxial row.
pub fn has_label_contains(harness: &Harness<'_, CherryApp>, text: &str) -> bool {
    use egui_kittest::kittest::Queryable;
    harness.query_all_by_label_contains(text).next().is_some()
}

/// True if a node whose accessible name exactly equals `text` currently
/// exists. See [`has_label_contains`] for why this uses the `_all` query.
pub fn has_label(harness: &Harness<'_, CherryApp>, text: &str) -> bool {
    use egui_kittest::kittest::Queryable;
    harness.query_all_by_label(text).next().is_some()
}

/// True if a node whose accessible *value* (not label) exactly equals
/// `text` currently exists — e.g. a `SpinButton`'s current numeric value,
/// which AccessKit exposes via `value`/`numeric_value` rather than `label`.
/// See [`has_label_contains`] for why this uses the `_all` query.
pub fn has_value(harness: &Harness<'_, CherryApp>, text: &str) -> bool {
    use egui_kittest::kittest::Queryable;
    harness.query_all_by_value(text).next().is_some()
}
