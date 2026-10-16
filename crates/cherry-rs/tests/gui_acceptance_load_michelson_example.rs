#![cfg(feature = "gui")]

mod gui_support;

use std::time::Duration;

use egui_kittest::kittest::Queryable;
use gui_support::{has_label_contains, launch, wait_until};

/// AT-10: loading "Michelson Interferometer" computes a valid result for
/// both arms. Each arm focuses through the f = 100 mm detector lens, so the
/// Paraxial Summary shows an EFL of 100 for the active path and again after
/// switching to the other arm.
#[test]
fn load_michelson_shows_efl_for_both_arms() {
    let mut harness = launch();
    harness.get_by_label("Examples").click();
    harness.run();
    harness.get_by_label("Michelson Interferometer").click();
    harness.run();

    harness.get_by_label("Paraxial Summary").click();
    harness.run();

    for arm in ["Arm 1 (M1)", "Arm 2 (M2)"] {
        wait_until(
            &mut harness,
            &format!("{arm}: Paraxial window shows the EFL"),
            Duration::from_secs(2),
            |h| {
                has_label_contains(h, arm)
                    && has_label_contains(h, "Effective focal length")
                    && !has_label_contains(h, "No data yet")
            },
        );
        assert!(
            harness.get_all_by_label_contains("100.0000").count() > 0,
            "{arm}: expected the paraxial window to show 100.0000"
        );
        harness.get_by_label("\u{25b6}").click();
        harness.run();
    }
}
