#![warn(clippy::pedantic)]

use state::CellState;
use std::ops::{Add, Mul};

pub mod state;
pub mod ui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Coordinates<T = i32> {
    pub y: T,
    pub x: T,
}

impl<T> Coordinates<T> {
    pub fn to_index(&self, width: T) -> T
    where
        T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
    {
        self.y * width + self.x
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CellRenderInfo {
    pub state: CellState,
    pub coordinates: Coordinates,
    pub needs_rerender: bool,
}

pub mod prelude {
    pub use super::{
        state::{self, CellState, State},
        ui::{Renderer, RendererBackend},
        CellRenderInfo, Coordinates,
    };
}
