use crate::{app::Window, values::Values};
use egui::{vec2, Context, Id, Layout, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};
use egui_file::FileDialog;
use std::hash::Hash;

pub struct TableWindow {
    id: Id,
    title: String,
    keys: Vec<String>,
    save_dialog: Option<FileDialog>,
}

impl Window for TableWindow {
    fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new(&self.title)
            .id(self.id)
            .default_size(vec2(100.0, 200.0))
            .vscroll(true)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }
}

impl TableWindow {
    pub fn new(id: impl Hash, key: String) -> Self {
        Self {
            id: Id::new(id),
            title: key.clone(),
            keys: vec![key],
            save_dialog: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, values: &Values) {
        ScrollArea::horizontal()
            .id_source(self.id.with("header"))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for key in values.keys() {
                        if ui.selectable_label(self.keys.contains(key), key).clicked() {
                            if let Some(index) = self.keys.iter().position(|k| k == key) {
                                self.keys.remove(index);
                            } else {
                                self.keys.push(key.to_owned());
                            }
                            self.title = self.keys.join(",");
                        }
                    }
                });
            });
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Save CSV").clicked() {
            let mut fd = FileDialog::save_file(None)
                .default_filename(format!("{}.csv", self.title))
                .title("Save as CSV");
            fd.open();
            self.save_dialog = Some(fd);
        }
        ui.separator();
        let table = TableBuilder::new(ui)
            .cell_layout(Layout::left_to_right(egui::Align::Center))
            .columns(Column::auto(), self.keys.len())
            .stick_to_bottom(true);
        table
            .header(20.0, |mut header| {
                for key in &self.keys {
                    header.col(|ui| {
                        ui.strong(key);
                    });
                }
            })
            .body(|body| {
                let mut values: Vec<_> = self
                    .keys
                    .iter()
                    .map(|key| values.values_for_key(key))
                    .collect();
                let max_len = values
                    .iter()
                    .map(|v| v.as_ref().map(|v| v.len()).unwrap_or_default())
                    .max()
                    .unwrap_or_default();
                body.rows(20.0, max_len, |index, mut row| {
                    for iter in values.iter_mut() {
                        row.col(|ui| {
                            if let Some(it) = iter.as_mut() {
                                let offset = max_len - it.len();
                                if offset <= index {
                                    if let Some(v) = it.get(index - offset) {
                                        ui.label(v.to_string());
                                    } else {
                                        *iter = None;
                                    }
                                }
                            }
                        });
                    }
                });
            });
        if let Some(save_dialog) = self.save_dialog.as_mut() {
            if save_dialog.show(ui.ctx()).selected() {
                if let Some(path) = save_dialog.path() {
                    let _ = values.save_csv(path, self.keys.iter());
                }
                self.save_dialog = None;
            }
        }
    }
}
