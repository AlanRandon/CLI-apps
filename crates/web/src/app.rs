use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

#[derive(Serialize, Deserialize, Clone, Prop)]
pub struct AppProps {
    pub page: Page,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Page {
    Index,
    NotFound,
    Error,
}

impl AppProps {
    pub fn encode(&self) -> String {
        let bytes = postcard::to_stdvec(self).unwrap();
        general_purpose::STANDARD_NO_PAD.encode(bytes)
    }

    pub fn decode(data: &str) -> Self {
        let bytes = general_purpose::STANDARD_NO_PAD.decode(data).unwrap();
        postcard::from_bytes(&bytes).unwrap()
    }
}

#[component]
pub fn App<G: Html>(cx: Scope, props: AppProps) -> View<G> {
    let count = create_signal(cx, 0);
    view! {
        cx,
        div {
            (count.get())
        }
        button(on:click = |_| count.set(*count.get() + 1)) {
            "Click Me"
        }
    }
}

pub fn app_string(props: AppProps) -> String {
    sycamore::render_to_string(move |cx| {
        let encoded_props = props.clone().encode();
        view! {
            cx,
            html(data-props = encoded_props) {
                head {
                    title {
                        "Hello World"
                    }
                    script(type = "module", src = "init.js")
                }
                body {
                    App(props)
                }
            }
        }
    })
}

#[cfg(target_arch = "wasm32")]
pub fn hydrate() {
    use web_sys::window;

    let encoded_props = window()
        .unwrap()
        .document()
        .unwrap()
        .document_element()
        .unwrap()
        .get_attribute("data-props")
        .unwrap();

    let props = AppProps::decode(&encoded_props);

    sycamore::hydrate(|cx| {
        view! {
            cx,
            App(props)
        }
    })
}
