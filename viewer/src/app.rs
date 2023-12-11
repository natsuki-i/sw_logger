use crate::{
    graph::{LineGraph, XYGraph},
    table::TableWindow,
    values::Values,
};
use egui::ahash::HashMap;
use egui_file::FileDialog;
use ewebsock::{WsMessage, WsReceiver, WsSender};

pub trait Window {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, values: &Values);
}

pub struct App {
    id: usize,
    server: String,
    ws: Option<(WsSender, WsReceiver)>,
    values: Values,
    windows: Vec<(Box<dyn Window>, bool)>,
    save_dialog: Option<FileDialog>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        #[cfg(target_arch = "wasm32")]
        let server = {
            let location = &_cc.integration_info.web_info.location;
            format!("ws://{}/socket", location.host)
        };
        #[cfg(not(target_arch = "wasm32"))]
        let server = "ws://127.0.0.1:8080/socket".into();
        Self {
            id: 0,
            server,
            ws: None,
            values: Default::default(),
            windows: vec![],
            save_dialog: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some((_, rx)) = self.ws.as_ref() {
            while let Some(e) = rx.try_recv() {
                match e {
                    ewebsock::WsEvent::Opened => {}
                    ewebsock::WsEvent::Message(WsMessage::Text(m)) => {
                        match serde_json::from_str::<HashMap<String, Vec<f32>>>(&m) {
                            Ok(v) => {
                                for (k, v) in v {
                                    self.values.push(k, v);
                                }
                            }
                            Err(e) => {
                                log::error!("failed to parse: {}", e);
                            }
                        }
                    }
                    ewebsock::WsEvent::Message(_) => {}
                    ewebsock::WsEvent::Error(e) => log::error!("{}", e),
                    ewebsock::WsEvent::Closed => {
                        let ctx = ctx.clone();
                        let wakeup = move || ctx.request_repaint();
                        self.ws = ewebsock::connect_with_wakeup(&self.server, wakeup)
                            .map_err(|e| log::error!("failed to init websocket {}", e))
                            .ok();
                        break;
                    }
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.separator();
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if ui.button("Save as CSV").clicked() {
                            let mut fd = FileDialog::save_file(None)
                                .default_filename("all.csv")
                                .title("Save as CSV");
                            fd.open();
                            self.save_dialog = Some(fd);
                        }
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    }
                });
                ui.menu_button("Settings", |ui| {
                    ui.menu_button("Retention period", |ui| {
                        for (label, len) in [
                            ("10sec", 60 * 10),
                            ("1min", 60 * 60),
                            ("5min", 60 * 60 * 5),
                            ("10min", 60 * 60 * 10),
                            ("15min", 60 * 60 * 15),
                            ("30min", 60 * 60 * 30),
                        ] {
                            if ui.radio(self.values.max_len() == len, label).clicked() {
                                self.values.set_max_len(len);
                                ui.close_menu();
                            }
                        }
                    });
                });
                egui::widgets::reset_button(ui, &mut self.values);
                ui.separator();
                if ui.button("XY Graph").clicked() {
                    self.windows.push((
                        Box::new(XYGraph::new(format!("xy_graph_{}", self.id))),
                        true,
                    ));
                    self.id += 1;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.server);
                if self.ws.is_none() {
                    if ui.button("connect").clicked() {
                        let ctx = ctx.clone();
                        let wakeup = move || ctx.request_repaint();
                        self.ws = ewebsock::connect_with_wakeup(&self.server, wakeup)
                            .map_err(|e| log::error!("failed to init websocket {}", e))
                            .ok();
                    }
                } else if ui.button("disconnect").clicked() {
                    self.ws = None;
                }
            });
            ui.separator();
            self.table(ui);
        });

        for graph in &mut self.windows {
            graph.0.show(ctx, &mut graph.1, &self.values);
        }
        self.windows.retain(|g| g.1);

        if let Some(save_dialog) = self.save_dialog.as_mut() {
            if save_dialog.show(ctx).selected() {
                if let Some(path) = save_dialog.path() {
                    let _ = self.values.save_csv(path, self.values.keys());
                }
                self.save_dialog = None;
            }
        }
    }
}

impl App {
    fn table(&mut self, ui: &mut egui::Ui) {
        let mut keys: Vec<_> = self.values.keys().collect();
        keys.sort();
        use egui_extras::{Column, TableBuilder};
        let table = TableBuilder::new(ui)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::exact(256.0))
            .column(Column::auto());

        table
            .header(20.0, |mut header| {
                header.col(|_| {});
                header.col(|ui| {
                    ui.strong("Key");
                });
                header.col(|ui| {
                    ui.strong("Last Value");
                });
            })
            .body(|body| {
                body.rows(20.0, keys.len(), |index, mut row| {
                    let key = keys[index];
                    row.col(|ui| {
                        if ui.button("G").clicked() {
                            self.windows
                                .push((Box::new(LineGraph::new(self.id, key.to_owned())), true));
                            self.id += 1;
                        }
                        if ui.button("T").clicked() {
                            self.windows
                                .push((Box::new(TableWindow::new(self.id, key.to_owned())), true));
                            self.id += 1;
                        }
                    });
                    row.col(|ui| {
                        ui.label(key);
                    });
                    row.col(|ui| {
                        if let Some(v) = self.values.get_last_value_for_key(key) {
                            ui.label(v.to_string());
                        }
                    });
                });
            });
    }
}
