use cfg_if::cfg_if;
use leptos::*;
pub mod app;
pub mod error_template;

#[cfg(feature = "ssr")]
pub mod fileserv;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::wasm_bindgen;
        use crate::app::*;

        #[wasm_bindgen]
        pub fn hydrate() {
            _ = console_log::init_with_level(log::Level::Debug);
            console_error_panic_hook::set_once();

            leptos::mount_to_body(move |cx| {
                view! { cx, <App/> }
            });
        }
    }
}
