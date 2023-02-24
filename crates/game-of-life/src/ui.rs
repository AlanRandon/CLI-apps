use crate::{prelude::*, state::State};
use std::marker::PhantomData;

pub mod terminal;

pub trait RendererBackend<E>: Sized
where
    E: std::error::Error,
{
    type Config;

    fn render<I>(&mut self, state: I) -> Result<(), E>
    where
        I: Iterator<Item = CellRenderInfo>;

    fn renderer(config: Self::Config) -> Result<Renderer<Self, E>, E>;
}

pub struct Renderer<B, E>
where
    B: RendererBackend<E>,
    E: std::error::Error,
{
    state: State,
    backend: B,
    _phantom: PhantomData<E>,
}

impl<B, E> Renderer<B, E>
where
    B: RendererBackend<E>,
    E: std::error::Error,
{
    pub fn new(state: State, backend: B) -> Self {
        Self {
            state,
            backend,
            _phantom: PhantomData,
        }
    }

    pub fn render_next_state(&mut self) -> Result<(), E> {
        let Self { state, backend, .. } = self;
        backend.render(state.next_state())
    }
}
