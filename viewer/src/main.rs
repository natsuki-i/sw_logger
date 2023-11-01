use ewebsock::{WsMessage, WsReceiver, WsSender};
use std::collections::{HashMap, VecDeque};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        min_window_size: Some(egui::vec2(640.0, 480.0)),
        ..Default::default()
    };

    eframe::run_native(
        "sw_logger",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start("canvas", web_options, Box::new(|cc| Box::new(App::new(cc))))
            .await
            .expect("failed to start")
    });
}

pub struct App {
    server: String,
    ws: Option<(WsSender, WsReceiver)>,
    values: HashMap<String, VecDeque<f32>>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            server: "ws://127.0.0.1:8080/socket".into(),
            ws: None,
            values: Default::default(),
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
                        match serde_json::from_str::<HashMap<String, f32>>(&m) {
                            Ok(v) => {
                                for (k, v) in v {
                                    let q = self.values.entry(k).or_default();
                                    q.push_back(v);
                                    while q.len() > 600 {
                                        q.pop_front();
                                    }
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
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    });
                    ui.separator();
                }
                if self.ws.is_none() {
                    if ui.button("connect").clicked() {
                        let ctx = ctx.clone();
                        let wakeup = move || ctx.request_repaint();
                        self.ws = ewebsock::connect_with_wakeup(&self.server, wakeup)
                            .map_err(|e| log::error!("failed to init websocket {}", e))
                            .ok();
                    }
                } else {
                    if ui.button("disconnect").clicked() {
                        self.ws = None;
                    }
                }
                egui::widgets::reset_button(ui, &mut self.values);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.text_edit_singleline(&mut self.server);
            for (k, v) in &self.values {
                ui.label(format!("{}: {}", k, v.back().unwrap_or(&0f32)));
            }
        });
    }
}
