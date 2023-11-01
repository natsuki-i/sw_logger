mod values;
mod app;

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
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start("canvas", web_options, Box::new(|cc| Box::new(app::App::new(cc))))
            .await
            .expect("failed to start")
    });
}
