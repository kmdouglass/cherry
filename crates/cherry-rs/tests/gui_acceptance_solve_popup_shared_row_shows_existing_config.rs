#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, Queryable};
use gui_support::{has_label_contains, launch, wait_until};

/// VT-SOLVE-2: opening the RoC-solve popup from a `Shared` row shows the
/// *existing* solve configuration, not a blank "create new" state — a
/// `Curvature`-kind solve's identity is keyed by store index (FR-SOLVE-5),
/// not by which path's row happened to open the popup (FR-SOLVE-4).
///
/// Builds the scenario from scratch rather than `wf_epi_microscope` (whose
/// shared surfaces are thin lenses, which don't expose an RoC column at
/// all): "Concave Mirror" gives a single curved (reflecting `Sphere`)
/// surface to configure an `F/#` solve on; a second path is then added and
/// switched to `Shared`, targeting that same mirror.
#[test]
fn solve_popup_opened_from_a_shared_row_shows_the_underlying_solve() {
    let mut harness = launch();

    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Concave Mirror").click();
    harness.run();
    harness.get_by_label("Surfaces").click();
    harness.run();

    // Configure an F/# solve on the mirror's RoC cell (path 0). There are
    // two inactive "○" solve buttons on this fixture (the object-to-mirror
    // gap's thickness, and the mirror's RoC); the RoC one is the second —
    // see the sibling multipath Solve tests for how indices like this were
    // derived by dumping the real AccessKit tree.
    harness.get_all_by_label("\u{25cb}").nth(1).unwrap().click();
    harness.run();

    let n = harness.get_all(By::new().role(Role::ComboBox)).count();
    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(n - 1)
        .unwrap()
        .click();
    harness.run();
    harness.get_by_label("F/#").click();
    harness.run();
    harness
        .get_all(By::new().role(Role::TextInput))
        .last()
        .unwrap()
        .click();
    harness.run();
    harness
        .get_all(By::new().role(Role::TextInput))
        .last()
        .unwrap()
        .type_text("4");
    harness.run();
    harness.get_by_label("OK").click();
    harness.run();

    // Add a second path and switch its one non-boundary row to `Shared`,
    // targeting the mirror.
    harness.get_by_label("+ Add Path").click();
    harness.run();
    wait_until(
        &mut harness,
        "path 1 created",
        Duration::from_secs(2),
        |h| has_label_contains(h, "Path 1"),
    );
    harness.get_by_label("Surfaces").click();
    harness.run();
    harness.get_all_by_label("+").next().unwrap().click();
    harness.run();

    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(1)
        .unwrap()
        .click();
    harness.run();
    harness.get_by_label("Shared").click();
    harness.run();

    harness
        .get_all(By::new().role(Role::ComboBox))
        .nth(2)
        .unwrap()
        .click();
    harness.run();
    harness.get_by_label("Sphere \u{2014} 0").click();
    harness.run();

    // The Shared row now shows the solve as active ("●") — open it from
    // *this* path's row.
    harness.get_all_by_label("\u{25cf}").next().unwrap().click();
    harness.run();

    // FR-SOLVE-4: the popup shows the existing config, not a blank "None".
    harness.get_by_label("Target F/#:");
    harness.get(By::new().role(Role::TextInput).value("4"));
    assert!(
        harness.query_by_label("None").is_none(),
        "the popup must show the existing F/# solve, not a blank create-new state"
    );
}
