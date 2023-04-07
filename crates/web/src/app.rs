use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

#[derive(Serialize, Deserialize, Clone, Prop)]
pub struct AppProps {
    pub page: Page,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Page {
    Index,
    NotFound { uri: String },
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
    match props.page {
        Page::Index => {
            let valid_range = 0..10;
            let count = create_signal(cx, 0);

            let increment = |_| count.set(*count.get() + 1);
            let decrement = |_| count.set(*count.get() - 1);

            view! {
                cx,
                div(class="flex gap-4 items-center") {
                    button(
                        on:click = decrement,
                        class="btn",
                        disabled=*count.get() <= valid_range.start
                    ) {
                        "-1"
                    }
                    div {
                        (count.get())
                    }
                    button(
                        on:click = increment,
                        class="btn",
                        disabled=*count.get() >= (valid_range.end - 1)
                    ) {
                        "+1"
                    }
                }
            }
        }
        Page::NotFound { uri } => {
            view! {
                cx,
                h1 {
                    (format!("Cannot find page \"{uri}\""))
                }
            }
        }
        Page::Error => {
            view! {
                cx,
                h1 {
                    "An error occurred"
                }
            }
        }
    }
}

pub fn app_string(props: AppProps) -> String {
    sycamore::render_to_string(move |cx| {
        let encoded_props = props.encode();
        view! {
            cx,
            html(data-props = encoded_props) {
                head {
                    title {
                        "Hello World"
                    }
                    script(src="init.js", type="module")
                    link(rel="stylesheet", href="style.css")
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
