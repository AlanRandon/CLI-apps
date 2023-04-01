use game_of_life_core::{prelude as game_of_life, state::CellState};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::sync::{Arc, Mutex, RwLock};
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

/// A function to toggle the internal and signal state of one cell
type CellToggleFunction = Arc<Mutex<Box<dyn FnMut()>>>;

struct StateWrapper {
    state: Arc<RwLock<game_of_life::State>>,
    signals: Vec<(
        ReadSignal<CellState>,
        WriteSignal<CellState>,
        CellToggleFunction,
    )>,
}

impl StateWrapper {
    fn new(cx: Scope, state: game_of_life::State) -> Self {
        let cells = state.cells();

        let state = Arc::new(RwLock::new(state));

        let signals = create_many_signals(cx, cells)
            .into_iter()
            .enumerate()
            .map(|(index, (cell_state, set_cell_state_internal))| {
                let state = Arc::clone(&state);
                let set_cell_state = move || {
                    let new_cell_state = !cell_state.get();
                    state
                        .write()
                        .unwrap()
                        .replace_at_index(index, new_cell_state);
                    set_cell_state_internal.set(new_cell_state);
                };
                (
                    cell_state,
                    set_cell_state_internal,
                    Arc::from(Mutex::new(Box::new(set_cell_state) as Box<dyn FnMut()>)),
                )
            })
            .collect();

        Self { signals, state }
    }

    fn next_state(&mut self) {
        let Self { state, signals } = self;

        for game_of_life::CellRenderInfo {
            state: new_state,
            coordinates,
            needs_rerender,
        } in state.write().unwrap().next().unwrap().into_iter()
        {
            if !needs_rerender {
                continue;
            }

            let (_, set_state_internal, _) = signals[coordinates.to_index(20) as usize];

            set_state_internal.update(|state| *state = new_state);
        }
    }

    fn cell_signals(&self) -> Vec<(ReadSignal<CellState>, CellToggleFunction)> {
        self.signals
            .iter()
            .map(|(state, _, set_state)| (*state, set_state.clone()))
            .collect()
    }
}

#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let mut state = StateWrapper::new(cx, game_of_life::State::new(20, 20));
    let cells = state.cell_signals();

    let (should_update, set_should_update) = create_signal(cx, false);

    #[allow(unused_variables)]
    let tick = move || {
        if should_update.get() {
            state.next_state();
        }
    };

    #[cfg(target_arch = "wasm32")]
    {
        let callback = Closure::wrap(Box::new(tick) as Box<dyn FnMut()>).into_js_value();
        let interval = std::time::Duration::from_millis(100);
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
                    each=move || cells.clone().into_iter().enumerate()
                    key=|(id, _)| *id
                    view=move |cx, (_, (state, set_state))| {
                        view! {
                            cx,
                            <div
                                class="p-4 w-fit h-fit grid place-items-center rounded-sm transition-colors"
                                class=("bg-slate-500", move || state.get() == game_of_life::CellState::Dead)
                                on:click=move |_| set_state.lock().unwrap()()
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
