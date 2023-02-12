use crate::cell::Cell;
use crate::prelude::*;
use rayon::prelude::*;
use std::sync::Arc;

pub struct State {
    cells: Vec<Arc<Cell>>,
    width: usize,
    height: usize,
}

impl State {
    pub fn new(width: usize, height: usize) -> Self {
        let length = width * height;
        let cells = (0..length).map(|_| Arc::new(Cell::new(false))).collect();
        Self {
            height,
            width,
            cells,
        }
    }

    fn get_adjacent(&self, index: usize) -> [Arc<Cell>; 8] {
        let Self {
            cells,
            width,
            height,
        } = self;

        let (y, x) = (index / width, index % width);
        [
            cells.get((y + 1 % height) * width + x).unwrap(),
            cells.get((y + 1 % height) * width + x - 1 % width).unwrap(),
            cells.get((y + 1 % height) * width + x + 1 % width).unwrap(),
            cells.get(y * width - x + 1 % width).unwrap(),
            cells.get(y * width + x + 1 % width).unwrap(),
            cells.get((y - 1 % height) * width + x - 1 % width).unwrap(),
            cells.get((y - 1 % height) * width + x).unwrap(),
            cells.get((y - 1 % height) * width + x + 1 % width).unwrap(),
        ]
        .map(Arc::clone)
    }

    pub fn tick(&mut self) {
        let new_cells = self
            .cells
            .par_iter()
            .enumerate()
            .map(|(index, cell)| {
                let adjacency_count = self
                    .get_adjacent(index)
                    .iter()
                    .fold(0, |acc, cell| acc + cell.alive as usize);

                let next_state = match (cell.alive, adjacency_count) {};

                cell.clone()
            })
            .collect::<Vec<_>>();

        self.cells = new_cells;
    }
}
