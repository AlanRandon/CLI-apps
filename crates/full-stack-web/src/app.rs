use leptos::*;
use leptos_meta::*;
use leptos_router::*;

// TODO: render game of life
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        // <Stylesheet id="leptos" href="/pkg/start-axum.css"/>
        <Title text="Welcome to Leptos"/>
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {move || count.get().to_string()}</button>
    }
}
