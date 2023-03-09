use crate::{state::State, CellRenderInfo};
use std::marker::PhantomData;

pub trait RendererBackend<E>: Sized
where
    E: std::error::Error,
{
    type Config;

    /// Renders the state provided by the iterator.
    ///
    /// # Errors
    /// When the backed experiences an error, such as failing to render to stdout, it will error.
    fn render<I>(&mut self, state: I) -> Result<(), E>
    where
        I: Iterator<Item = CellRenderInfo>;

    /// Creates a renderer given some config.
    ///
    /// # Errors
    /// When it fails to create a renderer, it will error.
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

    /// Renders the next state.
    ///
    /// # Errors
    /// When it fails to a render, it will error.
    pub fn render_next_state(&mut self) -> Result<(), E> {
        let Self { state, backend, .. } = self;
        backend.render(state.next_state())
    }
}
