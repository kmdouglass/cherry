use std::collections::{HashMap, HashSet};

use egui_extras::{Column, TableBuilder};

use crate::{
    gui::{
        model::{LensGroupSpec, SystemSpecs},
        result_package::ResultPackage,
    },
    views::components::{Component, PathComponent},
};

/// Floating panel that groups auto-detected optical components and exposes
/// collective decenter / rotation controls.
#[derive(Default)]
pub struct LensOverlayPanel {
    selected: HashSet<usize>,
    stale_names: Vec<String>,
    merge_counter: usize,
    last_result_id: Option<u64>,
    /// Last `active_path` the stale-group validation ran against. Switching
    /// `active_path` alone produces no new `ResultPackage` (§3.3), so
    /// `last_result_id` staying the same must not suppress re-validation —
    /// see FR-OUT-3a.
    last_active_path: Option<usize>,
}

fn component_first_idx(c: &Component) -> usize {
    match c {
        Component::Element { surf_idxs } => *surf_idxs
            .first()
            .expect("Element must have at least one surface"),
        Component::Iris { stop_idx } => *stop_idx,
        Component::Mirror { surf_idx }
        | Component::ThinLens { surf_idx }
        | Component::BeamSplitter { surf_idx }
        | Component::UnpairedSurface { surf_idx } => *surf_idx,
    }
}

/// Maps each store index reachable from `active_path` to its **step**
/// index within that path's own `surface_refs` — the indexing
/// `cross_section.rs` actually uses for `surface_frames[active_path]`
/// (mirrors how `surfaces.rs`'s own hover handler writes a step index, not
/// a store index). `LensGroupSpec.component_first_surfs` stores store
/// indices, so the RUF-axis hover annotation must convert through this map
/// rather than using a store index directly — store and step indices only
/// coincide by accident in a single-path system.
fn store_index_to_step_map(specs: &SystemSpecs, active_path: usize) -> HashMap<usize, usize> {
    let store_table = specs.store_index_table();
    let Some(path) = specs.paths.get(active_path) else {
        return HashMap::new();
    };
    path.surface_refs
        .iter()
        .enumerate()
        .filter_map(|(step, r)| Some((store_table.resolve(r)?, step)))
        .collect()
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
        Component::ThinLens { surf_idx } => format!("Thin Lens ({surf_idx})"),
        Component::BeamSplitter { surf_idx } => format!("Beam Splitter ({surf_idx})"),
        Component::UnpairedSurface { surf_idx } => format!("Surface ({surf_idx})"),
    }
}

/// Validates `path_id`'s lens groups against that path's detected
/// `components`. Discards stale groups (whose component_first_surfs no longer
/// match any detected component) and promotes unassigned components to new
/// default single-component groups on `path_id`. Other paths' groups are left
/// untouched: they describe components of their own path, not this one.
///
/// Returns `(stale_names, n_new)` where `stale_names` are the names of the
/// discarded groups and `n_new` is the number of newly added groups.
fn validate_and_sync(
    lens_groups: &mut Vec<LensGroupSpec>,
    components: &[PathComponent],
    path_id: usize,
) -> (Vec<String>, usize) {
    let known: HashSet<usize> = components
        .iter()
        .map(|pc| component_first_idx(&pc.component))
        .collect();

    let mut stale_names = Vec::new();
    lens_groups.retain(|g| {
        if g.path_id != path_id {
            return true;
        }
        let ok = g
            .component_first_surfs
            .iter()
            .all(|&fs| known.contains(&fs));
        if !ok {
            stale_names.push(g.name.clone());
        }
        ok
    });

    let mut covered: HashSet<usize> = lens_groups
        .iter()
        .filter(|g| g.path_id == path_id)
        .flat_map(|g| g.component_first_surfs.iter().copied())
        .collect();

    let mut n_new = 0usize;
    for pc in components {
        let first = component_first_idx(&pc.component);
        // `insert` also guards against a component listed twice by a path
        // that revisits it.
        if covered.insert(first) {
            let mut g = LensGroupSpec::new(default_group_name(&pc.component), path_id);
            g.component_first_surfs = vec![first];
            lens_groups.push(g);
            n_new += 1;
        }
    }

    lens_groups.sort_by_key(|g| {
        (
            g.path_id,
            g.component_first_surfs
                .first()
                .copied()
                .unwrap_or(usize::MAX),
        )
    });

    (stale_names, n_new)
}

/// Indices into `specs.lens_groups` of the groups belonging to
/// `active_path`, in list order — the rows of the Lens Overlay table.
fn visible_groups(specs: &SystemSpecs, active_path: usize) -> Vec<usize> {
    specs
        .lens_groups
        .iter()
        .enumerate()
        .filter(|(_, g)| g.path_id == active_path)
        .map(|(i, _)| i)
        .collect()
}

/// Replace the `selected` groups (indices into `groups`) with one group named
/// `name` holding all their components, inserted where the first selected
/// group was. Only groups on the first selected group's path are merged; a
/// group never spans paths.
fn merge_groups(groups: &mut Vec<LensGroupSpec>, selected: &[usize], name: String) {
    let mut sel = selected.to_vec();
    sel.sort_unstable();
    sel.dedup();
    let Some(&insert_pos) = sel.first() else {
        return;
    };
    let path_id = groups[insert_pos].path_id;
    sel.retain(|&i| groups[i].path_id == path_id);

    let mut merged_surfs: Vec<usize> = sel
        .iter()
        .flat_map(|&i| groups[i].component_first_surfs.iter().copied())
        .collect();
    merged_surfs.sort_unstable();
    merged_surfs.dedup();

    for &i in sel.iter().rev() {
        groups.remove(i);
    }
    let mut new_group = LensGroupSpec::new(name, path_id);
    new_group.component_first_surfs = merged_surfs;
    groups.insert(insert_pos, new_group);
}

/// Split each selected multi-component group (indices into `groups`) into
/// single-component groups on the same path, named by `name_for(first_surf)`.
fn split_groups(
    groups: &mut Vec<LensGroupSpec>,
    selected: &[usize],
    name_for: impl Fn(usize) -> String,
) {
    let mut sel = selected.to_vec();
    sel.sort_unstable_by(|a, b| b.cmp(a)); // descending — stable indices
    sel.dedup();
    for i in sel {
        if groups[i].component_first_surfs.len() <= 1 {
            continue;
        }
        let group = groups.remove(i);
        for (offset, &key) in group.component_first_surfs.iter().enumerate() {
            let mut new_group = LensGroupSpec::new(name_for(key), group.path_id);
            new_group.component_first_surfs = vec![key];
            groups.insert(i + offset, new_group);
        }
    }
}

/// One decenter or rotation cell. Disabled, with an explanation on hover,
/// when group transforms don't apply. Returns `true` if the value changed.
fn transform_cell(ui: &mut egui::Ui, value: &mut f64, enabled: bool) -> bool {
    ui.add_enabled(enabled, egui::DragValue::new(value).speed(0.01))
        .on_disabled_hover_text("Group transforms apply only to single-path systems")
        .changed()
}

impl LensOverlayPanel {
    /// Show the lens overlay window, filtered to `active_path` (FR-OUT-2).
    /// Returns `true` if `specs` changed.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        specs: &mut SystemSpecs,
        result: Option<&ResultPackage>,
        active_path: usize,
    ) -> bool {
        // Sync whenever a new ResultPackage arrives *or* active_path changes
        // (FR-OUT-3a) — switching paths alone produces no new ResultPackage,
        // but a group can become stale purely because its members were never
        // part of the newly active path's own components.
        let mut changed = false;
        if let Some(r) = result
            && (self.last_result_id != Some(r.id) || self.last_active_path != Some(active_path))
        {
            let path_switched = self.last_active_path.is_some_and(|p| p != active_path);
            self.last_result_id = Some(r.id);
            self.last_active_path = Some(active_path);
            // The warning and the selection both describe the previous path's
            // groups; neither applies once that view is gone. Keeping the
            // selection would let Merge reach into another path's groups.
            if path_switched {
                self.stale_names.clear();
                self.selected.clear();
            }
            let active_components: Vec<PathComponent> = r
                .components
                .iter()
                .filter(|pc| pc.path_id == active_path)
                .cloned()
                .collect();
            let (stale, n_new) =
                validate_and_sync(&mut specs.lens_groups, &active_components, active_path);
            // Only this path's groups are checked, so a discard here is a
            // genuine one — possibly from an edit made while another path was
            // active, surfacing now that this path is validated again.
            // Either change can reorder `lens_groups`, invalidating the
            // selected indices.
            if !stale.is_empty() || n_new > 0 {
                self.stale_names.extend(stale);
                self.selected.clear();
                changed = true;
            }
        }

        let inner = egui::Window::new("Lens Overlay")
            .open(open)
            .default_width(720.0)
            .show(ctx, |ui| self.show_contents(ui, specs, result, active_path))
            .and_then(|r| r.inner)
            .unwrap_or(false);

        changed || inner
    }

    fn show_contents(
        &mut self,
        ui: &mut egui::Ui,
        specs: &mut SystemSpecs,
        result: Option<&ResultPackage>,
        active_path: usize,
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

        // Store index -> step index within active_path's own surface_refs,
        // for the RUF-axis hover annotation. Computed eagerly (not as a
        // closure over `specs`) so it doesn't hold a borrow across the
        // row-rendering loop below, which mutates `specs.lens_groups`.
        let step_for_store_index = store_index_to_step_map(specs, active_path);

        let comp_lookup: std::collections::HashMap<usize, &Component> = result
            .components
            .iter()
            .filter(|pc| pc.path_id == active_path)
            .map(|pc| (component_first_idx(&pc.component), &pc.component))
            .collect();

        let rows = visible_groups(specs, active_path);
        // Group transforms are only applied to single-path systems
        // (`convert.rs`); elsewhere the fields would silently do nothing.
        let transforms_apply = specs.paths.len() == 1;
        let selected_snapshot = self.selected.clone();

        // Collect selection changes inside the table (deferred to avoid
        // simultaneous mutable borrows).
        let mut select_add: Vec<usize> = Vec::new();
        let mut select_remove: Vec<usize> = Vec::new();

        egui::ScrollArea::horizontal().show(ui, |ui| {
            let ctx = ui.ctx().clone();
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .sense(egui::Sense::click())
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
                    // `row_idx` indexes `specs.lens_groups`, not the table.
                    for &row_idx in &rows {
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

                            // Name (editable)
                            row.col(|ui| {
                                let group = &mut specs.lens_groups[row_idx];
                                if ui.text_edit_singleline(&mut group.name).changed() {
                                    changed = true;
                                }
                            });

                            // Components summary (read-only)
                            row.col(|ui| {
                                let summary: Vec<String> = specs.lens_groups[row_idx]
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
                                changed |= transform_cell(
                                    ui,
                                    &mut specs.lens_groups[row_idx].decenter[0],
                                    transforms_apply,
                                );
                            });
                            row.col(|ui| {
                                changed |= transform_cell(
                                    ui,
                                    &mut specs.lens_groups[row_idx].decenter[1],
                                    transforms_apply,
                                );
                            });
                            row.col(|ui| {
                                changed |= transform_cell(
                                    ui,
                                    &mut specs.lens_groups[row_idx].decenter[2],
                                    transforms_apply,
                                );
                            });

                            // Euler angles θ / ψ / φ
                            row.col(|ui| {
                                changed |= transform_cell(
                                    ui,
                                    &mut specs.lens_groups[row_idx].rotation[0],
                                    transforms_apply,
                                );
                            });
                            row.col(|ui| {
                                changed |= transform_cell(
                                    ui,
                                    &mut specs.lens_groups[row_idx].rotation[1],
                                    transforms_apply,
                                );
                            });
                            row.col(|ui| {
                                changed |= transform_cell(
                                    ui,
                                    &mut specs.lens_groups[row_idx].rotation[2],
                                    transforms_apply,
                                );
                            });

                            if row.response().hovered() {
                                let store_idx = specs.lens_groups[row_idx]
                                    .component_first_surfs
                                    .first()
                                    .copied();
                                if let Some(&step_idx) =
                                    store_idx.and_then(|si| step_for_store_index.get(&si))
                                {
                                    ctx.data_mut(|d| {
                                        d.insert_temp(
                                            egui::Id::new("annotation_hover_surface_idx"),
                                            step_idx,
                                        );
                                    });
                                }
                            }
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
                let sel: Vec<usize> = self.selected.iter().copied().collect();
                merge_groups(&mut specs.lens_groups, &sel, new_name);
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
                let sel: Vec<usize> = self.selected.iter().copied().collect();
                split_groups(&mut specs.lens_groups, &sel, |key| {
                    comp_lookup
                        .get(&key)
                        .map(|c| default_group_name(c))
                        .unwrap_or_else(|| format!("Surface ({key})"))
                });

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
    use crate::gui::model::{
        FieldMode, FieldRow, GapRow, LensGroupSpec, LinkedObjectOrientationRow, PathRow, RowId,
        SurfaceRefRow, SurfaceRow,
    };

    fn make_element(surfs: Vec<usize>) -> Component {
        Component::Element { surf_idxs: surfs }
    }

    fn make_mirror(idx: usize) -> Component {
        Component::Mirror { surf_idx: idx }
    }

    fn as_path_components(components: Vec<Component>) -> Vec<PathComponent> {
        components
            .into_iter()
            .map(|component| PathComponent {
                path_id: 0,
                component,
            })
            .collect()
    }

    fn make_group(name: &str, first_surfs: Vec<usize>) -> LensGroupSpec {
        make_path_group(name, 0, first_surfs)
    }

    fn make_path_group(name: &str, path_id: usize, first_surfs: Vec<usize>) -> LensGroupSpec {
        let mut g = LensGroupSpec::new(name, path_id);
        g.component_first_surfs = first_surfs;
        g
    }

    /// A path that revisits a surface (a Michelson arm crossing its beam
    /// splitter twice) lists that component twice; it still gets one group.
    #[test]
    fn validate_adds_one_group_for_a_revisited_component() {
        let mut groups: Vec<LensGroupSpec> = Vec::new();
        let components = as_path_components(vec![
            Component::BeamSplitter { surf_idx: 2 },
            make_mirror(3),
            Component::BeamSplitter { surf_idx: 2 },
        ]);
        let (_, n_new) = validate_and_sync(&mut groups, &components, 0);
        assert_eq!(n_new, 2);
        let names: Vec<&str> = groups.iter().map(|g| g.name.as_str()).collect();
        assert_eq!(names, vec!["Beam Splitter (2)", "Mirror (3)"]);
    }

    #[test]
    fn validate_adds_default_groups_when_empty() {
        let mut groups: Vec<LensGroupSpec> = Vec::new();
        let components = as_path_components(vec![make_element(vec![1, 2]), make_mirror(3)]);
        let (stale, n_new) = validate_and_sync(&mut groups, &components, 0);
        assert!(stale.is_empty());
        assert_eq!(n_new, 2);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "Element (1\u{2013}2)");
        assert_eq!(groups[1].name, "Mirror (3)");
    }

    #[test]
    fn validate_discards_stale_group() {
        let mut groups = vec![make_group("OldGroup", vec![5])];
        let components = as_path_components(vec![make_element(vec![1, 2])]);
        let (stale, n_new) = validate_and_sync(&mut groups, &components, 0);
        assert_eq!(stale, vec!["OldGroup"]);
        assert_eq!(n_new, 1);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Element (1\u{2013}2)");
    }

    #[test]
    fn validate_keeps_valid_groups_and_adds_new_component() {
        let mut groups = vec![make_group("MyLens", vec![1])];
        let components = as_path_components(vec![make_element(vec![1, 2]), make_mirror(3)]);
        let (stale, n_new) = validate_and_sync(&mut groups, &components, 0);
        assert!(stale.is_empty());
        assert_eq!(n_new, 1);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "MyLens");
        assert_eq!(groups[1].name, "Mirror (3)");
    }

    #[test]
    fn validate_groups_sorted_by_first_surf() {
        let mut groups: Vec<LensGroupSpec> = Vec::new();
        let components = as_path_components(vec![make_mirror(5), make_element(vec![1, 2])]);
        let (_, _) = validate_and_sync(&mut groups, &components, 0);
        assert_eq!(groups[0].component_first_surfs[0], 1);
        assert_eq!(groups[1].component_first_surfs[0], 5);
    }

    fn do_split(
        groups: &mut Vec<LensGroupSpec>,
        selected: &[usize],
        comp_lookup: &std::collections::HashMap<usize, Component>,
    ) {
        split_groups(groups, selected, |key| {
            comp_lookup
                .get(&key)
                .map(default_group_name)
                .unwrap_or_else(|| format!("Surface ({key})"))
        });
    }

    /// Validation only touches the given path's groups; another path's
    /// groups survive even though their components aren't in `components`.
    #[test]
    fn validate_leaves_other_paths_groups_untouched() {
        let mut groups = vec![
            make_path_group("Path0Lens", 0, vec![1]),
            make_path_group("Path1Doublet", 1, vec![6, 8]),
        ];
        let components = as_path_components(vec![make_element(vec![1, 2])]);
        let (stale, n_new) = validate_and_sync(&mut groups, &components, 0);
        assert!(stale.is_empty());
        assert_eq!(n_new, 0);
        let names: Vec<&str> = groups.iter().map(|g| g.name.as_str()).collect();
        assert_eq!(names, vec!["Path0Lens", "Path1Doublet"]);
    }

    /// New default groups belong to the path being validated, and a path
    /// already covered elsewhere doesn't suppress them: the shared component
    /// at store index 3 gets a group of its own on each path.
    #[test]
    fn validate_creates_groups_for_the_validated_path() {
        let mut groups = vec![make_path_group("Path0Objective", 0, vec![3])];
        let components = vec![PathComponent {
            path_id: 1,
            component: Component::ThinLens { surf_idx: 3 },
        }];
        let (_, n_new) = validate_and_sync(&mut groups, &components, 1);
        assert_eq!(n_new, 1);
        let tagged: Vec<(usize, &str)> = groups
            .iter()
            .map(|g| (g.path_id, g.name.as_str()))
            .collect();
        assert_eq!(tagged, vec![(0, "Path0Objective"), (1, "Thin Lens (3)")]);
    }

    /// Merging keeps the merged group on its path, and ignores a selected
    /// index that belongs to a different path.
    #[test]
    fn merge_stays_within_one_path() {
        let mut groups = vec![
            make_path_group("A", 0, vec![1]),
            make_path_group("B", 0, vec![3]),
            make_path_group("C", 1, vec![6]),
        ];
        merge_groups(&mut groups, &[0, 1, 2], "Group 1".into());
        let tagged: Vec<(usize, &str, Vec<usize>)> = groups
            .iter()
            .map(|g| (g.path_id, g.name.as_str(), g.component_first_surfs.clone()))
            .collect();
        assert_eq!(tagged, vec![(0, "Group 1", vec![1, 3]), (1, "C", vec![6])]);
    }

    /// Split groups inherit the original group's path.
    #[test]
    fn split_keeps_the_path() {
        let mut groups = vec![make_path_group("Big", 1, vec![6, 8])];
        split_groups(&mut groups, &[0], |key| format!("Surface ({key})"));
        assert!(groups.iter().all(|g| g.path_id == 1));
        assert_eq!(groups.len(), 2);
    }

    /// Only the active path's groups are listed in the table.
    #[test]
    fn visible_groups_are_the_active_paths() {
        let specs = SystemSpecs {
            lens_groups: vec![
                make_path_group("A", 0, vec![1]),
                make_path_group("B", 1, vec![6]),
                make_path_group("C", 0, vec![3]),
            ],
            ..Default::default()
        };
        assert_eq!(visible_groups(&specs, 0), vec![0, 2]);
        assert_eq!(visible_groups(&specs, 1), vec![1]);
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

    /// FR-OUT-3a / VT-OUT-2: switching `active_path` alone (no new
    /// `ResultPackage`) must still re-run stale-group discard. A group
    /// created while `active_path == 0`, referencing a path-0-exclusive
    /// component (e.g. the tube lens), must be discarded the moment
    /// `active_path` switches to 1 — even though `result.id` never changes.
    #[test]
    fn switching_active_path_without_new_result_revalidates_groups() {
        use crate::gui::result_package::ResultPackage;
        use std::cell::Cell;
        use std::rc::Rc;

        // Path 0 has an exclusive Element at surf 1 (the "tube lens"); the
        // objective (surf 3) is shared by both paths 0 and 1.
        let components = vec![
            PathComponent {
                path_id: 0,
                component: make_element(vec![1, 2]),
            },
            PathComponent {
                path_id: 0,
                component: Component::ThinLens { surf_idx: 3 },
            },
            PathComponent {
                path_id: 1,
                component: Component::ThinLens { surf_idx: 3 },
            },
        ];
        let result = ResultPackage {
            error: None,
            components,
            ..ResultPackage::error(1, String::new())
        };

        let specs = SystemSpecs::default();
        let panel = LensOverlayPanel::default();
        let active_path = Rc::new(Cell::new(0usize));

        let active_path_for_closure = Rc::clone(&active_path);
        let mut harness =
            egui_kittest::Harness::new_state(
                move |ctx,
                      (panel, specs, result): &mut (
                    LensOverlayPanel,
                    SystemSpecs,
                    ResultPackage,
                )| {
                    let mut open = true;
                    panel.show(
                        ctx,
                        &mut open,
                        specs,
                        Some(result),
                        active_path_for_closure.get(),
                    );
                },
                (panel, specs, result),
            );
        harness.run();
        let (_, s, _) = harness.state();
        // With active_path == 0, the tube-lens group (surf 1, path-0
        // exclusive) exists and is not stale.
        assert!(
            s.lens_groups
                .iter()
                .any(|g| g.component_first_surfs == vec![1]),
            "path-0-exclusive component should have its own group while active_path == 0"
        );

        // Switch active_path to 1 without changing the ResultPackage id, and
        // re-run — this must trigger re-validation (FR-OUT-3a), which gives
        // path 1 its own group for the shared objective...
        active_path.set(1);
        harness.run();
        let (_, s, _) = harness.state();
        assert!(
            s.lens_groups
                .iter()
                .any(|g| g.path_id == 1 && g.component_first_surfs == vec![3]),
            "path 1 should get its own group for the shared objective"
        );
        // ...while path 0's groups, including the path-0-exclusive one, are
        // kept rather than discarded (they are path 0's, not path 1's).
        assert!(
            s.lens_groups
                .iter()
                .any(|g| g.path_id == 0 && g.component_first_surfs == vec![1]),
            "path 0's tube-lens group must survive switching to path 1"
        );
    }

    /// Render the panel for `specs` and report, for each decenter/rotation
    /// drag value, whether it is enabled.
    fn transform_fields_enabled(specs: SystemSpecs) -> Vec<bool> {
        use crate::gui::result_package::ResultPackage;
        use egui::accesskit::Role;
        use egui_kittest::kittest::{By, NodeT, Queryable};

        let result = ResultPackage {
            error: None,
            components: vec![PathComponent {
                path_id: 0,
                component: make_element(vec![1, 2]),
            }],
            ..ResultPackage::error(1, String::new())
        };
        let mut harness =
            egui_kittest::Harness::builder()
                .with_size(egui::vec2(1400.0, 600.0))
                .build_state(
                    |ctx,
                     (panel, specs, result): &mut (
                        LensOverlayPanel,
                        SystemSpecs,
                        ResultPackage,
                    )| {
                        let mut open = true;
                        panel.show(ctx, &mut open, specs, Some(result), 0);
                    },
                    (LensOverlayPanel::default(), specs, result),
                );
        harness.run();
        harness
            .get_all(By::new().role(Role::SpinButton))
            .map(|n| !n.accesskit_node().is_disabled())
            .collect()
    }

    /// Group transforms only apply to single-path systems (`convert.rs`), so
    /// in a multipath system the decenter/rotation fields are disabled
    /// rather than silently ignored.
    #[test]
    fn transform_fields_disabled_for_multipath_systems() {
        let mut specs = SystemSpecs::default();
        specs.add_path();
        let enabled = transform_fields_enabled(specs);
        assert_eq!(enabled.len(), 6, "one group row: 3 decenters + 3 angles");
        assert!(enabled.iter().all(|&e| !e), "{enabled:?}");
    }

    #[test]
    fn transform_fields_enabled_for_single_path_systems() {
        let enabled = transform_fields_enabled(SystemSpecs::default());
        assert_eq!(enabled.len(), 6, "one group row: 3 decenters + 3 angles");
        assert!(enabled.iter().all(|&e| e), "{enabled:?}");
    }

    /// A user's merged group on path 0 survives a round trip through path 1.
    #[test]
    fn merged_group_survives_switching_paths_and_back() {
        use crate::gui::result_package::ResultPackage;
        use std::cell::Cell;
        use std::rc::Rc;

        let components = vec![
            PathComponent {
                path_id: 0,
                component: make_element(vec![1, 2]),
            },
            PathComponent {
                path_id: 0,
                component: Component::ThinLens { surf_idx: 3 },
            },
            PathComponent {
                path_id: 1,
                component: Component::ThinLens { surf_idx: 3 },
            },
        ];
        let result = ResultPackage {
            error: None,
            components,
            ..ResultPackage::error(1, String::new())
        };
        let specs = SystemSpecs {
            lens_groups: vec![make_path_group("Relay + objective", 0, vec![1, 3])],
            ..Default::default()
        };

        let active_path = Rc::new(Cell::new(0usize));
        let active_path_for_closure = Rc::clone(&active_path);
        let mut harness =
            egui_kittest::Harness::new_state(
                move |ctx,
                      (panel, specs, result): &mut (
                    LensOverlayPanel,
                    SystemSpecs,
                    ResultPackage,
                )| {
                    let mut open = true;
                    panel.show(
                        ctx,
                        &mut open,
                        specs,
                        Some(result),
                        active_path_for_closure.get(),
                    );
                },
                (LensOverlayPanel::default(), specs, result),
            );
        harness.run();
        for path in [1, 0] {
            active_path.set(path);
            harness.run();
        }

        let (panel, s, _) = harness.state();
        let path_0: Vec<(&str, Vec<usize>)> = s
            .lens_groups
            .iter()
            .filter(|g| g.path_id == 0)
            .map(|g| (g.name.as_str(), g.component_first_surfs.clone()))
            .collect();
        assert_eq!(path_0, vec![("Relay + objective", vec![1, 3])]);
        assert!(panel.stale_names.is_empty());
    }

    /// Switching `active_path` back and forth discards the other path's
    /// groups (FR-OUT-3a) but must not warn: nothing went stale, so the
    /// "Discarded stale groups" banner stays empty instead of growing on every
    /// switch.
    #[test]
    fn switching_active_path_does_not_report_stale_groups() {
        use crate::gui::result_package::ResultPackage;
        use std::cell::Cell;
        use std::rc::Rc;

        let components = vec![
            PathComponent {
                path_id: 0,
                component: make_element(vec![1, 2]),
            },
            PathComponent {
                path_id: 1,
                component: make_mirror(4),
            },
        ];
        let result = ResultPackage {
            error: None,
            components,
            ..ResultPackage::error(1, String::new())
        };

        let active_path = Rc::new(Cell::new(0usize));
        let active_path_for_closure = Rc::clone(&active_path);
        let mut harness =
            egui_kittest::Harness::new_state(
                move |ctx,
                      (panel, specs, result): &mut (
                    LensOverlayPanel,
                    SystemSpecs,
                    ResultPackage,
                )| {
                    let mut open = true;
                    panel.show(
                        ctx,
                        &mut open,
                        specs,
                        Some(result),
                        active_path_for_closure.get(),
                    );
                },
                (LensOverlayPanel::default(), SystemSpecs::default(), result),
            );
        harness.run();

        for path in [1, 0, 1, 0] {
            active_path.set(path);
            harness.run();
            let (panel, _, _) = harness.state();
            assert!(
                panel.stale_names.is_empty(),
                "switching to path {path} reported stale groups: {:?}",
                panel.stale_names
            );
        }
    }

    /// A discard caused by a new `ResultPackage` (an actual spec change)
    /// still reports the stale group.
    #[test]
    fn new_result_that_drops_a_component_reports_stale_group() {
        use crate::gui::result_package::ResultPackage;

        let make_result = |id, components| ResultPackage {
            error: None,
            components,
            ..ResultPackage::error(id, String::new())
        };
        let first = make_result(
            1,
            vec![PathComponent {
                path_id: 0,
                component: make_element(vec![1, 2]),
            }],
        );
        let second = make_result(
            2,
            vec![PathComponent {
                path_id: 0,
                component: make_mirror(3),
            }],
        );

        let mut panel = LensOverlayPanel::default();
        let mut specs = SystemSpecs::default();
        let ctx = egui::Context::default();
        for result in [&first, &second] {
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                let mut open = true;
                panel.show(ctx, &mut open, &mut specs, Some(result), 0);
            });
        }
        assert_eq!(panel.stale_names, vec!["Element (1\u{2013}2)"]);
    }

    /// Regression: the RUF-axis hover annotation must convert a lens
    /// group's `component_first_surfs` (a **store** index) into the
    /// **step** index within `active_path`'s own `surface_refs`, since
    /// `cross_section.rs` indexes `surface_frames[active_path]` by step.
    /// Store and step indices only coincide by accident in a single-path
    /// system; this fixture deliberately mismatches them: path 1's step 1
    /// (`Shared`) resolves to store index 2, not step index 2.
    #[test]
    fn store_index_to_step_map_converts_correctly_for_mismatched_multipath_indices() {
        let mut specs = SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"), // path 0 step 0 -> store 0
            SurfaceRow::new_sphere("10.0", "50.0", "5.0", "1.5"), // step 1 -> store 1
            SurfaceRow::new_sphere("10.0", "Infinity", "5.0", "1.0"), // step 2 -> store 2
            SurfaceRow::new_image(),            // step 3 -> store 3
        ]));

        let linked_id = specs.mint_row_id();
        let img_id = specs.mint_row_id();
        specs.paths.push(PathRow {
            name: None,
            surface_refs: vec![
                // path 1 step 0 -> store 4 (new Object, linked to path 0).
                SurfaceRefRow::ObjectLinkedTo {
                    id: linked_id,
                    path: 0,
                    orientation: LinkedObjectOrientationRow::SameDirection,
                    gap_after: GapRow::default(),
                },
                // path 1 step 1 -> store 2 (mismatched: step != store).
                SurfaceRefRow::Shared {
                    target: RowId(2),
                    gap_after: GapRow::default(),
                },
                // path 1 step 2 -> store 5.
                SurfaceRefRow::New(SurfaceRow::new_image().with_id(img_id)),
            ],
            stop_surface: None,
            fields: vec![FieldRow {
                chi: "0.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            }],
            field_mode: FieldMode::Angle,
            aperture_semi_diameter: "10.0".into(),
            wavelengths: vec!["0.520".into()],
            beam_splitter_arms: Vec::new(),
        });

        let map = store_index_to_step_map(&specs, 1);
        assert_eq!(
            map.get(&2),
            Some(&1),
            "store index 2 must map to step 1 within path 1, not step 2"
        );
        assert_eq!(map.get(&4), Some(&0), "the linked Object is step 0");
        assert_eq!(map.get(&5), Some(&2), "the trailing New Image is step 2");
        // Path 0's own store indices (0, 1, 3) are not reachable as steps
        // within path 1's own surface_refs.
        assert_eq!(map.get(&0), None);
        assert_eq!(map.get(&1), None);
        assert_eq!(map.get(&3), None);
    }
}
