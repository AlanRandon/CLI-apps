#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod app;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    app::hydrate();
}
