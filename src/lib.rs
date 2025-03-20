use wasm_bindgen::prelude::*;
// use wasm_thread as thread;

mod coords;
mod module;
mod project;
mod substrate;
mod tools;
mod upc;
mod utils;
mod viewport;
mod wgl2;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// #[wasm_bindgen]
// pub struct ThreadTest {
//     pub value: u64,
// }
//
// #[wasm_bindgen]
// impl ThreadTest {
//     #[wasm_bindgen(constructor)]
//     pub fn new() -> Self {
//         Self { value: 0 }
//     }
// }
//
// #[wasm_bindgen]
// pub fn run_thread_test() {
//     for _ in 0..2 {
//         thread::spawn(|| {
//             for i in 1..3 {
//                 log!(
//                     "hi number {} from the spawned thread {:?}!",
//                     i,
//                     thread::current().id()
//                 );
//                 thread::sleep(Duration::from_millis(1));
//             }
//         });
//     }
//
//     for i in 1..3 {
//         log!(
//             "hi number {} from the main thread {:?}!",
//             i,
//             thread::current().id()
//         );
//     }
// }
//
// // Threading test (Using shared array buffer)
// #[wasm_bindgen]
// pub fn increment_forever(thread_test: &mut ThreadTest) {
//     loop {
//         thread_test.value += 1;
//     }
// }
