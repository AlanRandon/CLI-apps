use game_of_life_core::prelude as game_of_life;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Stylesheet id="leptos" href="/pkg/full-stack-web.css"/>
        <Title text="Welcome to Leptos"/>
        <Router>
            <main class="grid place-items-center">
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let mut state = game_of_life::State::new(3, 3);

    let cells: Vec<_> = state
        .next()
        .unwrap()
        .to_state_iter()
        .enumerate()
        .map(move |(id, state)| {
            let (state, set_state) = create_signal(cx, state);
            (id, state, set_state)
        })
        .collect();

    view! {
        cx,
        <div class="p-4 grid place-items-center bg-slate-500 text-white rounded">
            <For
                each=move || cells.clone().into_iter()
                key=|(id, _, _)| id.clone()
                view=move |cx, (_, state, _)| {
                    view! {
                        cx,
                        <div class="p-4 w-4 h-4 grid place-items-center rounded" class=("bg-slate-500", move || state.get() == game_of_life::CellState::Dead)/>
                    }
                }
            />
        </div>
    }
}
