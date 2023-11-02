use egui::{vec2, Context, Id, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::hash::Hash;

use crate::values::Values;

pub struct LineGraph {
    id: Id,
    title: String,
    keys: Vec<String>,
}

impl LineGraph {
    pub fn new(id: impl Hash, key: String) -> Self {
        let id = Id::new(id);
        Self {
            id,
            title: key.clone(),
            keys: vec![key],
        }
    }

    pub fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new(&self.title)
            .id(self.id)
            .default_size(vec2(400.0, 600.0))
            .vscroll(false)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }

    pub fn ui(&mut self, ui: &mut Ui, values: &Values) {
        ui.horizontal(|ui| {
            for key in values.keys() {
                if ui.selectable_label(self.keys.contains(key), key).clicked() {
                    if let Some(index) = self.keys.iter().position(|k| k == key) {
                        self.keys.remove(index);
                    } else {
                        self.keys.push(key.to_owned());
                    }
                    self.title = self.keys.join(", ");
                };
            }
        });
        ui.separator();
        Plot::new(self.id.with("plot"))
            .legend(Legend::default())
            .y_axis_width(5)
            .show_axes(true)
            .show_grid(true)
            .show(ui, |ui| {
                for k in &self.keys {
                    if let Some(iter) = values.iter_for_key(k) {
                        let len = iter.len();
                        let line = Line::new(PlotPoints::from_iter(
                            iter.enumerate()
                                .map(|(c, v)| [(c as f64 - len as f64) / 60.0, *v as f64]),
                        ))
                        .name(k);
                        ui.line(line);
                    }
                }
            });
    }
}
