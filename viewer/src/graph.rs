use crate::{app::Window, values::Values};
use egui::{vec2, Context, Id, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use std::hash::Hash;

pub struct LineGraph {
    id: Id,
    title: String,
    keys: Vec<String>,
}

impl Window for LineGraph {
    fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new(&self.title)
            .id(self.id)
            .default_size(vec2(400.0, 600.0))
            .vscroll(false)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }
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

pub struct XYGraph {
    id: Id,
    selector: (String, String),
    keys: Vec<(String, String)>,
}

impl Window for XYGraph {
    fn show(&mut self, ctx: &Context, open: &mut bool, values: &Values) {
        egui::Window::new("XY Graph")
            .id(self.id)
            .default_size(vec2(400.0, 600.0))
            .vscroll(false)
            .open(open)
            .show(ctx, |ui| self.ui(ui, values));
    }
}

impl XYGraph {
    pub fn new(id: impl Hash) -> Self {
        let id = Id::new(id);
        Self {
            id,
            selector: Default::default(),
            keys: vec![],
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, values: &Values) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_source(self.id.with("x_selector"))
                .selected_text(&self.selector.0)
                .show_ui(ui, |ui| {
                    for key in values.keys() {
                        ui.selectable_value(&mut self.selector.0, key.to_owned(), key);
                    }
                });
            egui::ComboBox::from_id_source(self.id.with("y_selector"))
                .selected_text(&self.selector.1)
                .show_ui(ui, |ui| {
                    for key in values.keys() {
                        ui.selectable_value(&mut self.selector.1, key.to_owned(), key);
                    }
                });
            if ui.button("Add").clicked()
                && values.contains_key(&self.selector.0)
                && values.contains_key(&self.selector.1)
            {
                self.keys
                    .push(std::mem::replace(&mut self.selector, Default::default()));
            }
        });
        ui.separator();
        {
            let mut delete = None;
            for (index, keys) in self.keys.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{:5} {:5}", keys.0, keys.1));
                    if ui.button("Delete").clicked() {
                        delete = Some(index);
                    }
                });
            }
            if let Some(index) = delete {
                self.keys.remove(index);
            }
        }
        ui.separator();
        Plot::new(self.id.with("plot"))
            .y_axis_width(5)
            .legend(Legend::default())
            .show_axes(true)
            .show_grid(true)
            .data_aspect(1.0)
            .show(ui, |ui| {
                for (x_key, y_key) in &self.keys {
                    if let (Some(x_iter), Some(y_iter)) =
                        (values.iter_for_key(&x_key), values.iter_for_key(&y_key))
                    {
                        ui.line(
                            Line::new(PlotPoints::from_iter(
                                x_iter
                                    .rev()
                                    .zip(y_iter.rev())
                                    .rev()
                                    .map(|(x, y)| [*x as f64, *y as f64]),
                            ))
                            .name(format!("{} {}", x_key, y_key)),
                        );
                    }
                }
            });
    }
}
