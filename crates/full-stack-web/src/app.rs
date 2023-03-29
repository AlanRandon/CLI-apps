use game_of_life_core::prelude as game_of_life;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::time::Duration;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
    let mut state = game_of_life::State::new(20, 20);

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

    let change_state = {
        let cells = cells.clone();
        move || {
            for game_of_life::CellRenderInfo {
                state, coordinates, ..
            } in state.next().unwrap().into_iter()
            {
                cells[coordinates.to_index(20) as usize]
                    .2
                    .update(move |cell_state| *cell_state = state)
            }
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        let callback = Closure::wrap(Box::new(change_state) as Box<dyn FnMut()>).into_js_value();
        let interval = Duration::from_millis(100);
        window()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                interval.as_millis().try_into().unwrap_throw(),
            )
            .unwrap();
    }

    view! {
        cx,
        <div class="p-4 grid place-items-center">
            <div class="p-4 grid place-items-center text-white rounded gap-2 [grid-template-columns:repeat(20,1fr)] h-[90vmin] w-[90vmin]">
                <For
                    each=move || cells.clone().into_iter()
                    key=|(id, _, _)| id.clone()
                    view=move |cx, (_, state, _)| {
                        view! {
                            cx,
                            <div class="p-4 w-fit h-fit grid place-items-center rounded-sm transition-colors" class=("bg-slate-500", move || state.get() == game_of_life::CellState::Dead)/>
                        }
                    }
                />
            </div>
        </div>
    }
}
