use std::collections::{BTreeMap, BTreeSet};

use super::super::model::SystemSpecs;

/// Pre-computed shelf → book → page hierarchy from the material store keys.
#[derive(Default, Clone)]
pub struct MaterialIndex {
    /// shelf -> set of books on that shelf
    pub shelves: BTreeMap<String, BTreeSet<String>>,
    /// (shelf, book) -> set of pages in that book
    pub pages: BTreeMap<(String, String), BTreeSet<String>>,
}

impl MaterialIndex {
    /// Build the index by parsing store keys of the form "shelf:book:page".
    pub fn build_from_keys<'a>(keys: impl Iterator<Item = &'a String>) -> Self {
        let mut idx = Self::default();
        for key in keys {
            let parts: Vec<&str> = key.splitn(3, ':').collect();
            if parts.len() == 3 {
                let (shelf, book, page) = (parts[0], parts[1], parts[2]);
                idx.shelves
                    .entry(shelf.to_string())
                    .or_default()
                    .insert(book.to_string());
                idx.pages
                    .entry((shelf.to_string(), book.to_string()))
                    .or_default()
                    .insert(page.to_string());
            }
        }
        idx
    }
}

/// Transient UI state for the material browser dropdowns (not serialized).
#[derive(Default)]
pub struct MaterialBrowserState {
    pub selected_shelf: Option<String>,
    pub selected_book: Option<String>,
    pub selected_page: Option<String>,
}

/// Draw the materials browser panel. Returns true if specs were modified.
pub fn materials_panel(
    ui: &mut egui::Ui,
    specs: &mut SystemSpecs,
    index: &MaterialIndex,
    browser: &mut MaterialBrowserState,
) -> bool {
    let mut changed = false;

    // Mode toggle
    ui.horizontal(|ui| {
        ui.label("Refractive index source:");
        if ui
            .radio_value(&mut specs.use_materials, false, "Constant n")
            .changed()
        {
            changed = true;
        }
        if ui
            .radio_value(&mut specs.use_materials, true, "Material database")
            .changed()
        {
            changed = true;
        }
    });
    ui.separator();

    if !specs.use_materials {
        ui.label("Using constant refractive indices. Switch to Material mode to browse.");
        return changed;
    }

    // Browser section
    ui.heading("Browse Materials");
    ui.add_space(4.0);

    // Shelf ComboBox
    let shelf_label = browser
        .selected_shelf
        .as_deref()
        .unwrap_or("Select shelf...");
    egui::ComboBox::from_label("Shelf")
        .selected_text(shelf_label)
        .width(200.0)
        .show_ui(ui, |ui| {
            for shelf in index.shelves.keys() {
                if ui
                    .selectable_label(
                        browser.selected_shelf.as_deref() == Some(shelf),
                        shelf,
                    )
                    .clicked()
                {
                    browser.selected_shelf = Some(shelf.clone());
                    browser.selected_book = None;
                    browser.selected_page = None;
                }
            }
        });

    // Book ComboBox (depends on shelf)
    if let Some(shelf) = &browser.selected_shelf {
        let books = index.shelves.get(shelf);
        let book_label = browser
            .selected_book
            .as_deref()
            .unwrap_or("Select book...");
        egui::ComboBox::from_label("Book")
            .selected_text(book_label)
            .width(200.0)
            .show_ui(ui, |ui| {
                if let Some(books) = books {
                    for book in books {
                        if ui
                            .selectable_label(
                                browser.selected_book.as_deref() == Some(book),
                                book,
                            )
                            .clicked()
                        {
                            browser.selected_book = Some(book.clone());
                            browser.selected_page = None;
                        }
                    }
                }
            });
    }

    // Page ComboBox (depends on shelf + book)
    if let Some(shelf) = &browser.selected_shelf {
        if let Some(book) = &browser.selected_book {
            let pages = index.pages.get(&(shelf.clone(), book.clone()));
            let page_label = browser
                .selected_page
                .as_deref()
                .unwrap_or("Select page...");
            egui::ComboBox::from_label("Page")
                .selected_text(page_label)
                .width(200.0)
                .show_ui(ui, |ui| {
                    if let Some(pages) = pages {
                        for page in pages {
                            if ui
                                .selectable_label(
                                    browser.selected_page.as_deref() == Some(page),
                                    page,
                                )
                                .clicked()
                            {
                                browser.selected_page = Some(page.clone());
                            }
                        }
                    }
                });
        }
    }

    // Add button
    ui.add_space(4.0);
    let can_add = browser.selected_shelf.is_some()
        && browser.selected_book.is_some()
        && browser.selected_page.is_some();
    if ui
        .add_enabled(can_add, egui::Button::new("Add to selected"))
        .clicked()
    {
        if let (Some(shelf), Some(book), Some(page)) = (
            &browser.selected_shelf,
            &browser.selected_book,
            &browser.selected_page,
        ) {
            let key = format!("{shelf}:{book}:{page}");
            if !specs.selected_materials.contains(&key) {
                specs.selected_materials.push(key);
                changed = true;
            }
        }
    }

    ui.separator();
    ui.heading("Selected Materials");
    ui.add_space(4.0);

    let mut remove_idx = None;
    for (i, key) in specs.selected_materials.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(key);
            if ui.small_button("Remove").clicked() {
                remove_idx = Some(i);
            }
        });
    }

    if let Some(idx) = remove_idx {
        let removed_key = specs.selected_materials.remove(idx);
        // Clear material_key from any surfaces referencing this material.
        for surf in &mut specs.surfaces {
            if surf.material_key.as_deref() == Some(removed_key.as_str()) {
                surf.material_key = None;
            }
        }
        changed = true;
    }

    if specs.selected_materials.is_empty() {
        ui.label("No materials selected. Browse above to add.");
    }

    changed
}
