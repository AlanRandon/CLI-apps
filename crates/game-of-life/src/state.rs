use crate::cell::Cell;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{self, Row, Table},
};

pub struct State {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl State {
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = thread_rng();
        let length = width * height;
        let cells = (0..length).map(|_| Cell::new(rng.gen())).collect();
        Self {
            height,
            width,
            cells,
        }
    }

    fn get_adjacent(&self, index: usize) -> [Cell; 8] {
        let Self {
            cells,
            width,
            height,
        } = self;

        let index = index as i128;
        let width = *width as i128;
        let height = *height as i128;

        let (y, x) = (index / width, index % width);

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
        .map(|index| cells[index as usize])
    }

    pub fn tick(&mut self) {
        let new_cells = self
            .cells
            .par_iter()
            .enumerate()
            .map(|(index, &cell)| {
                let adjacent_alive_count = self
                    .get_adjacent(index)
                    .iter()
                    .fold(0, |acc, cell| acc + cell.alive as usize);

                let will_stay_alive = match (cell.alive, adjacent_alive_count) {
                    // Cells with 2 or 3 neighbours live
                    (true, 2 | 3) => true,
                    // Cells with 0, 1 or 4+ neighbours die
                    (true, _) => false,
                    // Cells with 3 neighbours become populated
                    (false, 3) => true,
                    // Cells without 3 neighbours stay dead
                    (false, _) => false,
                };

                Cell {
                    will_stay_alive,
                    ..cell
                }
            })
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(Cell::tick)
            .collect::<Vec<_>>();

        self.cells = new_cells;
    }
}

pub struct ToTable<'a> {
    pub table: Table<'a>,
    pub width_constraints: Vec<Constraint>,
}

impl<'a> From<&mut State> for ToTable<'a> {
    fn from(State { cells, width, .. }: &mut State) -> Self {
        let width_constraints = vec![Constraint::Length(1); *width];
        let table = Table::new(
            cells
                .chunks(*width)
                .map(|row| {
                    row.iter().map(|cell| {
                        widgets::Cell::from(" ").style(Style::default().bg(if cell.alive {
                            Color::White
                        } else {
                            Color::DarkGray
                        }))
                    })
                })
                .map(Row::new),
        )
        .column_spacing(0);
        ToTable {
            table,
            width_constraints,
        }
    }
}
