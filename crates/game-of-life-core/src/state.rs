use crate::{CellRenderInfo, Coordinates};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::prelude::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum CellState {
    Alive = 1,
    Dead = 0,
}

impl From<bool> for CellState {
    fn from(alive: bool) -> Self {
        if alive {
            Self::Alive
        } else {
            Self::Dead
        }
    }
}

pub struct State {
    cells: Vec<CellState>,
    width: usize,
    height: usize,
}

pub struct Frame {
    buffer: Vec<CellRenderInfo>,
}

impl State {
    pub fn new(width: usize, height: usize) -> Self {
        let length = width * height;

        let cells = SmallRng::from_entropy()
            .sample_iter::<bool, _>(Standard)
            .take(length)
            .par_bridge()
            .map(CellState::from)
            .collect();

        Self {
            cells,
            width,
            height,
        }
    }

    fn get_coordinates(&self, index: usize) -> Coordinates {
        let Self { width, .. } = self;

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let [width, index] = [*width as i32, index as _];

        Coordinates {
            x: index % width,
            y: index / width,
        }
    }

    fn get_neighbours(&self, coordinates: Coordinates) -> [CellState; 8] {
        let Self {
            cells,
            width,
            height,
            ..
        } = self;

        let Coordinates { x, y } = coordinates;

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let (width, height) = (*width as _, *height as _);

        [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ]
        .map(|(x_shift, y_shift)| {
            let x = (x + x_shift).rem_euclid(width);
            let y = (y + y_shift).rem_euclid(height);
            y * width + x
        })
        .map(
            #[allow(clippy::cast_sign_loss)]
            {
                |index| cells[index as usize]
            },
        )
    }

    fn get_alive_neighbours_count(&self, coordinates: Coordinates) -> usize {
        self.get_neighbours(coordinates)
            .iter()
            .fold(0, |acc, &state| acc + state as usize)
    }

    fn next_state_buffer(&mut self) -> Vec<CellRenderInfo> {
        let (internal_state, state): (Vec<_>, Vec<_>) = self
            .cells
            .par_iter()
            .enumerate()
            .map(|(index, &state)| {
                let coordinates = self.get_coordinates(index);

                let next_state = match (state, self.get_alive_neighbours_count(coordinates)) {
                    // Cells with 3 neighbours become populated or stay alive
                    // Cells with 2 neighbours also stay alive
                    (_, 3) | (CellState::Alive, 2) => CellState::Alive,
                    // Cells without 3 neighbours stay dead
                    // Cells with 0, 1 or 4+ neighbours die
                    _ => CellState::Dead,
                };

                (
                    next_state,
                    CellRenderInfo {
                        state: next_state,
                        coordinates,
                        needs_rerender: state != next_state,
                    },
                )
            })
            .collect::<Vec<_>>()
            .into_iter()
            .unzip();

        self.cells = internal_state;

        state
    }
}

impl std::iter::Iterator for State {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Frame {
            buffer: self.next_state_buffer(),
        })
    }
}

impl IntoIterator for Frame {
    type IntoIter = std::vec::IntoIter<CellRenderInfo>;
    type Item = CellRenderInfo;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}

impl Frame {
    pub fn to_buffer<F, R>(self, state_mapping: F) -> Vec<R>
    where
        F: Fn(CellState) -> R,
    {
        self.buffer
            .into_iter()
            .map(|CellRenderInfo { state, .. }| state_mapping(state))
            .collect()
    }
}
