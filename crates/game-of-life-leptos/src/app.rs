use game_of_life_core::{prelude as game_of_life, state::CellState};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::{
    cell::Cell,
    sync::{Arc, RwLock, RwLockWriteGuard},
    time::Duration,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Stylesheet id="leptos" href="/pkg/game-of-life-leptos.css"/>
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

struct StateWrapper {
    state: game_of_life::State,
    pub signals: Vec<(ReadSignal<CellState>, WriteSignal<CellState>)>,
}

impl StateWrapper {
    fn new(cx: Scope, state: game_of_life::State) -> Self {
        let signals = create_many_signals(cx, state.cells());

        for (index, (cell_state, set_cell_state)) in signals.iter().enumerate() {
            create_effect(cx, move |_| state.replace_at_index(index, cell_state.get()))
        }

        Self { signals, state }
    }

    fn next_state(&mut self) {
        let Self { state, signals } = self;

        for game_of_life::CellRenderInfo {
            state: new_state,
            coordinates,
            needs_rerender,
        } in state.next().unwrap().into_iter()
        {
            if !needs_rerender {
                continue;
            }

            signals[coordinates.to_index(20) as usize]
                .1
                .update_untracked(|state| *state = new_state)
        }
    }
}

#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let mut state = StateWrapper::new(cx, game_of_life::State::new(20, 20));

    let (state_signal, set_state_signal) = create_signal(cx, ());

    let (should_update, set_should_update) = create_signal(cx, false);

    let change_state = {
        move || {
            if !should_update.get() {
                return;
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
        <div class="p-4 grid place-items-center h-full w-full">
            <div class="p-4 grid place-items-center text-white rounded gap-2 [grid-template-columns:repeat(20,1fr)] aspect-square h-fit">
                <For
                    each=move || cells.clone().into_iter()
                    key=|(id, _, _)| *id
                    view=move |cx, (_, state, set_state)| {
                        view! {
                            cx,
                            <div
                                class="p-4 w-fit h-fit grid place-items-center rounded-sm transition-colors"
                                class=("bg-slate-500", move || state.get() == game_of_life::CellState::Dead)
                                on:click=move |_| with_state(move |state| set_state(, !state.get()))
                            />
                        }
                    }
                />
            </div>
            <div class="p-4 grid place-items-center">
                <button
                    class="p-4 rounded bg-indigo-500 hover:bg-indigo-600 active:opacity-70 text-white shadow transition-colors"
                    on:click=move |_| set_should_update.update(move |value| *value = !*value)
                >
                    "Toggle"
                </button>
            </div>
        </div>
    }
}
