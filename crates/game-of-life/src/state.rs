use crate::cell::Cell;
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
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
        let length = width * height;

        let cells = SmallRng::from_entropy()
            .sample_iter(Standard)
            .take(length)
            .par_bridge()
            .map(Cell::new)
            .collect();

        Self {
            cells,
            width,
            height,
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
        .map(
            #[allow(clippy::cast_sign_loss)]
            {
                |index| cells[index as usize]
            },
        )
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
                    .fold(0, |acc, cell| acc + usize::from(cell.alive));

                let will_stay_alive = match (cell.alive, adjacent_alive_count) {
                    // Cells with 3 neighbours become populated or stay alive
                    // Cells with 2 neighbours also stay alive
                    (_, 3) | (true, 2) => true,
                    // Cells without 3 neighbours stay dead
                    // Cells with 0, 1 or 4+ neighbours die
                    _ => false,
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
