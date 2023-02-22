use crate::prelude::*;
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
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

        let width = *width as i128;
        let index = index as i128;

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
        } = self;

        let Coordinates { x, y } = coordinates;

        let width = *width as i128;
        let height = *height as i128;

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

    /// Tick will return an iterator containing the coordinates and state of each cell
    pub fn next_state(&mut self) -> impl Iterator<Item = (Coordinates, CellState)> {
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

                (coordinates, (state, next_state))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(coordinates, (state, next_state))| (next_state, (coordinates, state)))
            .unzip();

        self.cells = internal_state;

        state.into_iter()
    }
}
