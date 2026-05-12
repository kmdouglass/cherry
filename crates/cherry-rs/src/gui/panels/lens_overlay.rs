use std::collections::HashSet;

use egui_extras::{Column, TableBuilder};

use crate::{
    gui::{
        model::{LensGroupSpec, SystemSpecs},
        result_package::ResultPackage,
    },
    views::components::Component,
};

/// Floating panel that groups auto-detected optical components and exposes
/// collective decenter / rotation controls.
#[derive(Default)]
pub struct LensOverlayPanel {
    selected: HashSet<usize>,
    stale_names: Vec<String>,
    merge_counter: usize,
    last_result_id: Option<u64>,
}

fn component_first_idx(c: &Component) -> usize {
    match c {
        Component::Element { surf_idxs } => *surf_idxs
            .first()
            .expect("Element must have at least one surface"),
        Component::Iris { stop_idx } => *stop_idx,
        Component::Mirror { surf_idx } | Component::UnpairedSurface { surf_idx } => *surf_idx,
    }
}

fn default_group_name(c: &Component) -> String {
    match c {
        Component::Element { surf_idxs } => {
            let first = surf_idxs
                .first()
                .expect("Element must have at least one surface");
            let last = surf_idxs
                .last()
                .expect("Element must have at least one surface");
            format!("Element ({first}\u{2013}{last})")
        }
        Component::Iris { stop_idx } => format!("Iris ({stop_idx})"),
        Component::Mirror { surf_idx } => format!("Mirror ({surf_idx})"),
        Component::UnpairedSurface { surf_idx } => format!("Surface ({surf_idx})"),
    }
}

/// Validates `lens_groups` against detected `components`. Discards stale groups
/// (whose component_first_surfs no longer match any detected component) and
/// promotes unassigned components to new default single-component groups.
///
/// Returns `(stale_names, n_new)` where `stale_names` are the names of the
/// discarded groups and `n_new` is the number of newly added groups.
fn validate_and_sync(
    lens_groups: &mut Vec<LensGroupSpec>,
    components: &[Component],
) -> (Vec<String>, usize) {
    let known: HashSet<usize> = components.iter().map(component_first_idx).collect();

    let mut stale_names = Vec::new();
    let valid: Vec<LensGroupSpec> = lens_groups
        .drain(..)
        .filter(|g| {
            let ok = g
                .component_first_surfs
                .iter()
                .all(|&fs| known.contains(&fs));
            if !ok {
                stale_names.push(g.name.clone());
            }
            ok
        })
        .collect();
    *lens_groups = valid;

    let covered: HashSet<usize> = lens_groups
        .iter()
        .flat_map(|g| g.component_first_surfs.iter().copied())
        .collect();

    let mut n_new = 0usize;
    for comp in components {
        let first = component_first_idx(comp);
        if !covered.contains(&first) {
            let mut g = LensGroupSpec::new(default_group_name(comp));
            g.component_first_surfs = vec![first];
            lens_groups.push(g);
            n_new += 1;
        }
    }

    lens_groups.sort_by_key(|g| {
        g.component_first_surfs
            .first()
            .copied()
            .unwrap_or(usize::MAX)
    });

    (stale_names, n_new)
}

impl LensOverlayPanel {
    /// Show the lens overlay window. Returns `true` if `specs` changed.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        specs: &mut SystemSpecs,
        result: Option<&ResultPackage>,
    ) -> bool {
        // Sync with any newly arrived ResultPackage, even while the window is
        // closed, so groups stay current and recomputes are triggered.
        let mut changed = false;
        if let Some(r) = result
            && self.last_result_id != Some(r.id)
        {
            self.last_result_id = Some(r.id);
            let (stale, n_new) = validate_and_sync(&mut specs.lens_groups, &r.components);
            if !stale.is_empty() {
                self.stale_names.extend(stale);
                self.selected.clear();
                changed = true;
            }
            if n_new > 0 {
                changed = true;
            }
        }

        let inner = egui::Window::new("Lens Overlay")
            .open(open)
            .default_width(720.0)
            .show(ctx, |ui| self.show_contents(ui, specs, result))
            .and_then(|r| r.inner)
            .unwrap_or(false);

        changed || inner
    }

    fn show_contents(
        &mut self,
        ui: &mut egui::Ui,
        specs: &mut SystemSpecs,
        result: Option<&ResultPackage>,
    ) -> bool {
        let mut changed = false;

        // Stale-groups warning banner.
        if !self.stale_names.is_empty() {
            ui.horizontal(|ui| {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    format!("Discarded stale groups: {}", self.stale_names.join(", ")),
                );
                if ui.small_button("Dismiss").clicked() {
                    self.stale_names.clear();
                }
            });
            ui.separator();
        }

        // Waiting state: no result yet.
        let Some(result) = result else {
            ui.centered_and_justified(|ui| {
                ui.label("Waiting for compute result\u{2026}");
            });
            return changed;
        };

        let components = &result.components;
        let comp_lookup: std::collections::HashMap<usize, &Component> = components
            .iter()
            .map(|c| (component_first_idx(c), c))
            .collect();

        let n_groups = specs.lens_groups.len();
        let selected_snapshot = self.selected.clone();

        // Collect selection changes inside the table (deferred to avoid
        // simultaneous mutable borrows).
        let mut select_add: Vec<usize> = Vec::new();
        let mut select_remove: Vec<usize> = Vec::new();

        egui::ScrollArea::horizontal().show(ui, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto().at_least(16.0)) // select
                .column(Column::initial(130.0).resizable(true)) // Name
                .column(Column::initial(190.0).resizable(true)) // Components
                .column(Column::initial(72.0).resizable(true)) // Dec R
                .column(Column::initial(72.0).resizable(true)) // Dec U
                .column(Column::initial(72.0).resizable(true)) // Dec F
                .column(Column::initial(65.0).resizable(true)) // theta
                .column(Column::initial(65.0).resizable(true)) // psi
                .column(Column::initial(65.0).resizable(true)); // phi

            table
                .header(20.0, |mut header| {
                    header.col(|_ui| {});
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Components");
                    });
                    header.col(|ui| {
                        ui.strong("Dec R (mm)");
                    });
                    header.col(|ui| {
                        ui.strong("Dec U (mm)");
                    });
                    header.col(|ui| {
                        ui.strong("Dec F (mm)");
                    });
                    header.col(|ui| {
                        ui.strong("\u{03b8} (deg)");
                    });
                    header.col(|ui| {
                        ui.strong("\u{03c8} (deg)");
                    });
                    header.col(|ui| {
                        ui.strong("\u{03c6} (deg)");
                    });
                })
                .body(|mut body| {
                    for row_idx in 0..n_groups {
                        let is_selected = selected_snapshot.contains(&row_idx);
                        body.row(22.0, |mut row| {
                            row.set_selected(is_selected);

                            // Select checkbox
                            row.col(|ui| {
                                let mut sel = is_selected;
                                if ui.checkbox(&mut sel, "").changed() {
                                    if sel {
                                        select_add.push(row_idx);
                                    } else {
                                        select_remove.push(row_idx);
                                    }
                                }
                            });

                            let group = &mut specs.lens_groups[row_idx];

                            // Name (editable)
                            row.col(|ui| {
                                if ui.text_edit_singleline(&mut group.name).changed() {
                                    changed = true;
                                }
                            });

                            // Components summary (read-only)
                            row.col(|ui| {
                                let summary: Vec<String> = group
                                    .component_first_surfs
                                    .iter()
                                    .filter_map(|&fs| {
                                        comp_lookup.get(&fs).map(|c| default_group_name(c))
                                    })
                                    .collect();
                                ui.label(summary.join(", "));
                            });

                            // Decenter R / U / F
                            row.col(|ui| {
                                if ui
                                    .add(egui::DragValue::new(&mut group.decenter[0]).speed(0.01))
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                            row.col(|ui| {
                                if ui
                                    .add(egui::DragValue::new(&mut group.decenter[1]).speed(0.01))
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                            row.col(|ui| {
                                if ui
                                    .add(egui::DragValue::new(&mut group.decenter[2]).speed(0.01))
                                    .changed()
                                {
                                    changed = true;
                                }
                            });

                            // Euler angles θ / ψ / φ
                            row.col(|ui| {
                                if ui
                                    .add(egui::DragValue::new(&mut group.rotation[0]).speed(0.01))
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                            row.col(|ui| {
                                if ui
                                    .add(egui::DragValue::new(&mut group.rotation[1]).speed(0.01))
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                            row.col(|ui| {
                                if ui
                                    .add(egui::DragValue::new(&mut group.rotation[2]).speed(0.01))
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                        });
                    }
                });
        });

        // Apply deferred selection changes.
        for idx in select_add {
            self.selected.insert(idx);
        }
        for idx in select_remove {
            self.selected.remove(&idx);
        }

        // Action buttons: Merge and Split.
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let can_merge = self.selected.len() >= 2;
            if ui
                .add_enabled(can_merge, egui::Button::new("Merge selected"))
                .clicked()
            {
                self.merge_counter += 1;
                let new_name = format!("Group {}", self.merge_counter);

                let mut sel: Vec<usize> = self.selected.iter().copied().collect();
                sel.sort_unstable();

                let mut merged_surfs: Vec<usize> = sel
                    .iter()
                    .flat_map(|&i| specs.lens_groups[i].component_first_surfs.iter().copied())
                    .collect();
                merged_surfs.sort_unstable();
                merged_surfs.dedup();

                let insert_pos = *sel.first().expect("selected is non-empty");
                for &i in sel.iter().rev() {
                    specs.lens_groups.remove(i);
                }
                let mut new_group = LensGroupSpec::new(new_name);
                new_group.component_first_surfs = merged_surfs;
                specs.lens_groups.insert(insert_pos, new_group);

                self.selected.clear();
                changed = true;
            }

            let can_split = self
                .selected
                .iter()
                .any(|&i| specs.lens_groups[i].component_first_surfs.len() > 1);
            if ui
                .add_enabled(can_split, egui::Button::new("Split selected"))
                .clicked()
            {
                let mut sel: Vec<usize> = self.selected.iter().copied().collect();
                sel.sort_unstable_by(|a, b| b.cmp(a)); // descending — stable indices

                for i in sel {
                    if specs.lens_groups[i].component_first_surfs.len() <= 1 {
                        continue;
                    }
                    let group = specs.lens_groups.remove(i);
                    for (offset, key) in group.component_first_surfs.iter().enumerate() {
                        let name = comp_lookup
                            .get(key)
                            .map(|c| default_group_name(c))
                            .unwrap_or_else(|| format!("Surface ({key})"));
                        let mut new_group = LensGroupSpec::new(name);
                        new_group.component_first_surfs = vec![*key];
                        specs.lens_groups.insert(i + offset, new_group);
                    }
                }

                self.selected.clear();
                changed = true;
            }
        });

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::LensGroupSpec;

    fn make_element(surfs: Vec<usize>) -> Component {
        Component::Element { surf_idxs: surfs }
    }

    fn make_mirror(idx: usize) -> Component {
        Component::Mirror { surf_idx: idx }
    }

    fn make_group(name: &str, first_surfs: Vec<usize>) -> LensGroupSpec {
        let mut g = LensGroupSpec::new(name);
        g.component_first_surfs = first_surfs;
        g
    }

    #[test]
    fn validate_adds_default_groups_when_empty() {
        let mut groups: Vec<LensGroupSpec> = Vec::new();
        let components = vec![make_element(vec![1, 2]), make_mirror(3)];
        let (stale, n_new) = validate_and_sync(&mut groups, &components);
        assert!(stale.is_empty());
        assert_eq!(n_new, 2);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "Element (1\u{2013}2)");
        assert_eq!(groups[1].name, "Mirror (3)");
    }

    #[test]
    fn validate_discards_stale_group() {
        let mut groups = vec![make_group("OldGroup", vec![5])];
        let components = vec![make_element(vec![1, 2])];
        let (stale, n_new) = validate_and_sync(&mut groups, &components);
        assert_eq!(stale, vec!["OldGroup"]);
        assert_eq!(n_new, 1);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Element (1\u{2013}2)");
    }

    #[test]
    fn validate_keeps_valid_groups_and_adds_new_component() {
        let mut groups = vec![make_group("MyLens", vec![1])];
        let components = vec![make_element(vec![1, 2]), make_mirror(3)];
        let (stale, n_new) = validate_and_sync(&mut groups, &components);
        assert!(stale.is_empty());
        assert_eq!(n_new, 1);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "MyLens");
        assert_eq!(groups[1].name, "Mirror (3)");
    }

    #[test]
    fn validate_groups_sorted_by_first_surf() {
        let mut groups: Vec<LensGroupSpec> = Vec::new();
        let components = vec![make_mirror(5), make_element(vec![1, 2])];
        let (_, _) = validate_and_sync(&mut groups, &components);
        assert_eq!(groups[0].component_first_surfs[0], 1);
        assert_eq!(groups[1].component_first_surfs[0], 5);
    }

    /// Simulates the split logic from `show_contents` without the egui
    /// plumbing.
    fn do_split(
        groups: &mut Vec<LensGroupSpec>,
        selected: &[usize],
        comp_lookup: &std::collections::HashMap<usize, Component>,
    ) {
        let mut sel = selected.to_vec();
        sel.sort_unstable_by(|a, b| b.cmp(a));
        for i in sel {
            if groups[i].component_first_surfs.len() <= 1 {
                continue;
            }
            let group = groups.remove(i);
            for (offset, key) in group.component_first_surfs.iter().enumerate() {
                let name = comp_lookup
                    .get(key)
                    .map(default_group_name)
                    .unwrap_or_else(|| format!("Surface ({key})"));
                let mut new_group = LensGroupSpec::new(name);
                new_group.component_first_surfs = vec![*key];
                groups.insert(i + offset, new_group);
            }
        }
    }

    #[test]
    fn split_merged_group_produces_single_component_groups() {
        let components = [
            make_element(vec![1, 2]),
            make_mirror(3),
            make_element(vec![4, 5]),
        ];
        let comp_lookup: std::collections::HashMap<usize, Component> = components
            .iter()
            .map(|c| (component_first_idx(c), c.clone()))
            .collect();

        // Start with all three merged into one group.
        let mut groups = vec![make_group("Big Group", vec![1, 3, 4])];

        do_split(&mut groups, &[0], &comp_lookup);

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].component_first_surfs, vec![1]);
        assert_eq!(groups[0].name, "Element (1\u{2013}2)");
        assert_eq!(groups[0].decenter, [0.0, 0.0, 0.0]);
        assert_eq!(groups[0].rotation, [0.0, 0.0, 0.0]);
        assert_eq!(groups[1].component_first_surfs, vec![3]);
        assert_eq!(groups[1].name, "Mirror (3)");
        assert_eq!(groups[2].component_first_surfs, vec![4]);
        assert_eq!(groups[2].name, "Element (4\u{2013}5)");
    }

    #[test]
    fn split_skips_single_component_groups() {
        let components = [make_element(vec![1, 2]), make_mirror(3)];
        let comp_lookup: std::collections::HashMap<usize, Component> = components
            .iter()
            .map(|c| (component_first_idx(c), c.clone()))
            .collect();

        // Two groups: one merged (keys 1, 3) and one single-component (key 3 already
        // covered — use a fresh setup: merged group with keys [1] only is
        // single, keys [1,3] is merged).
        let mut groups = vec![
            make_group("Merged", vec![1, 3]),
            make_group("Single", vec![5]),
        ];
        // Add a component for key 5 to comp_lookup.
        let comp5 = make_element(vec![5, 6]);
        let mut comp_lookup2 = comp_lookup;
        comp_lookup2.insert(5, comp5);

        // Select both rows.
        do_split(&mut groups, &[0, 1], &comp_lookup2);

        // Row 0 (merged) splits into 2; row 1 (single) unchanged.
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].component_first_surfs, vec![1]);
        assert_eq!(groups[1].component_first_surfs, vec![3]);
        assert_eq!(groups[2].component_first_surfs, vec![5]); // unchanged
        assert_eq!(groups[2].name, "Single");
    }

    #[test]
    fn default_group_name_element() {
        let c = make_element(vec![2, 3, 4]);
        assert_eq!(default_group_name(&c), "Element (2\u{2013}4)");
    }

    #[test]
    fn default_group_name_mirror() {
        let c = make_mirror(7);
        assert_eq!(default_group_name(&c), "Mirror (7)");
    }

    #[test]
    fn default_group_name_iris() {
        let c = Component::Iris { stop_idx: 4 };
        assert_eq!(default_group_name(&c), "Iris (4)");
    }

    #[test]
    fn default_group_name_unpaired() {
        let c = Component::UnpairedSurface { surf_idx: 9 };
        assert_eq!(default_group_name(&c), "Surface (9)");
    }
}
