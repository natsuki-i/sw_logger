use crate::{app::Window, values::Values};
use egui::{vec2, Context, Id, Layout, Ui};
use egui_extras::{Column, TableBuilder};
use std::hash::Hash;

pub struct TableWindow {
    id: Id,
    key: String,
}

impl Window for TableWindow {
    fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new(&self.key)
            .id(self.id)
            .default_size(vec2(400.0, 600.0))
            .vscroll(true)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }
}

impl TableWindow {
    pub fn new(id: impl Hash, key: String) -> Self {
        Self {
            id: Id::new(id),
            key,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, values: &Values) {
        let table = TableBuilder::new(ui)
            .cell_layout(Layout::left_to_right(egui::Align::Center))
            .column(Column::auto());
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("value");
                });
            })
            .body(|mut body| {
                if let Some(iter) = values.iter_for_key(&self.key) {
                    for v in iter {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(v.to_string());
                            });
                        });
                    }
                }
            });
    }
}
