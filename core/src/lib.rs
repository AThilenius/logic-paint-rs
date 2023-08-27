pub mod template_app;

use js_sys::{Function, Object, Reflect, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Only added if the 'web' feature is enabled
#[wasm_bindgen]
pub fn web_main() {
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "root",
                web_options,
                Box::new(|cc| Box::new(crate::template_app::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
