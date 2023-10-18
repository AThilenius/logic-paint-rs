use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, MessagePort};

#[wasm_bindgen]
pub fn plugin(port: MessagePort) -> Result<(), JsValue> {
    let send = port.clone();
    let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        let data = event.data();
        let data = data.as_string().unwrap();

        if data == "ping" {
            send.post_message(&JsValue::from_str("pong")).unwrap();
        }
    }) as Box<dyn FnMut(_)>);
    port.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();

    Ok(())
}

#[wasm_bindgen]
pub struct MyPlugin {
    foo: i32,
}

// #[wasm_bindgen]
// impl core::Plugin for MyPlugin {
//     fn ping(&self, msg: Vec<u8>) -> Vec<u8> {
//         // Return "pong"
//         vec![112, 111, 110, 103]
//     }
// }

#[wasm_bindgen]
pub fn get_plugin_impl() -> MyPlugin {
    MyPlugin { foo: 42 }
}

#[wasm_bindgen]
pub fn hello() {}
