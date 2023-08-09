#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod template_app;

fn main() {
    // eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "dev-mount", // hardcode it
                web_options,
                Box::new(|cc| Box::new(crate::template_app::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
