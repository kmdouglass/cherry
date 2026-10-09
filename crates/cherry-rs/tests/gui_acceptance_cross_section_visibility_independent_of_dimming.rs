#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::{Role, Toggled};
use egui_kittest::kittest::{By, NodeT, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// VT-XS-5: the per-path visibility checkbox is independent of active-path
/// dimming. Unchecking an *inactive* path hides it; switching `active_path`
/// to that now-hidden path makes it visible again (FR-XS-7's "no active-but-
/// hidden state", from the switcher-driven direction — the checklist-driven
/// direction is covered separately by VT-XS-6).
#[test]
fn unchecking_inactive_path_hides_it_then_switching_to_it_shows_it_again() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness
        .get_by_label("Widefield Epifluorescence Microscope")
        .click();
    harness.run();

    harness.get_by_label("Cross Section").click();
    harness.run();
    wait_until(
        &mut harness,
        "cross-section path checklist is populated",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 0") && has_label_contains(h, "Path 1"),
    );

    // Path 0 is active by default; path 1's checkbox starts checked
    // (visible), independent of it being the dimmed, inactive path.
    let checked = |h: &egui_kittest::Harness<'_, cherry_rs::gui::CherryApp>, label: &str| {
        h.get(By::new().role(Role::CheckBox).label(label))
            .accesskit_node()
            .toggled()
    };
    assert_eq!(checked(&harness, "Path 1"), Some(Toggled::True));

    // Uncheck path 1 (inactive) directly — this hides it entirely, distinct
    // from merely dimming it.
    harness
        .get(By::new().role(Role::CheckBox).label("Path 1"))
        .click();
    harness.run();
    assert_eq!(checked(&harness, "Path 1"), Some(Toggled::False));
    // Path 0 (active) is untouched by hiding path 1.
    assert_eq!(checked(&harness, "Path 0"), Some(Toggled::True));

    // Switching `active_path` to the now-hidden path 1 makes it visible
    // again (FR-XS-7) — no "active but hidden" state.
    harness.get_by_label("\u{25b6}").click();
    harness.run();

    wait_until(
        &mut harness,
        "path 1's checkbox re-checks itself once it becomes active",
        Duration::from_secs(2),
        |h| checked(h, "Path 1") == Some(Toggled::True),
    );
}
