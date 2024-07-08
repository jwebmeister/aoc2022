#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use day12::AppDay12;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "AoC 2022 app",
        native_options,
        Box::new(|cc| Ok(Box::new(AppDay12::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Ok(Box::new(AppDay12::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
