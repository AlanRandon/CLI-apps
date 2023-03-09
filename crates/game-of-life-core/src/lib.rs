#![warn(clippy::pedantic)]

use state::CellState;

pub mod state;
pub mod ui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Coordinates<T = i32> {
    pub y: T,
    pub x: T,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CellRenderInfo {
    pub state: CellState,
    pub coordinates: Coordinates,
    pub needs_rerender: bool,
}

pub mod prelude {
    pub use super::{
        state::{CellState, State},
        ui::{Renderer, RendererBackend},
        CellRenderInfo, Coordinates,
    };
}
