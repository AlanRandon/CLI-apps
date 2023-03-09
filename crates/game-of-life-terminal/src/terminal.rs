pub use config::Config;
use crossterm::{
    cursor::{self, MoveToColumn, MoveToRow},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Color, Print, SetBackgroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use game_of_life_core::prelude::*;
use std::io::{stdout, Stdout, Write};

mod config;

pub struct Backend {
    terminal: Stdout,
    colors: CellColors,
    has_rendered: bool,
}

struct CellColors {
    alive_color: Color,
    dead_color: Color,
}

impl CellColors {
    fn get_color(&self, state: CellState) -> Color {
        let Self {
            alive_color,
            dead_color,
        } = self;

        if state == CellState::Alive {
            *alive_color
        } else {
            *dead_color
        }
    }
}

impl Backend {
    fn new(alive_color: Color, dead_color: Color) -> crossterm::Result<Self> {
        enable_raw_mode()?;
        let mut terminal = stdout();
        execute!(
            terminal,
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide,
        )?;

        Ok(Self {
            terminal,
            has_rendered: false,
            colors: CellColors {
                alive_color,
                dead_color,
            },
        })
    }
}

impl RendererBackend<crossterm::ErrorKind> for Backend {
    type Config = Config;

    fn render<I>(&mut self, state: I) -> crossterm::Result<()>
    where
        I: Iterator<Item = CellRenderInfo>,
    {
        let Self {
            terminal,
            colors,
            has_rendered,
        } = self;

        let mut previous_y = -10_i32;

        for CellRenderInfo {
            coordinates: Coordinates { x, y },
            state,
            needs_rerender,
        } in state
        {
            if needs_rerender || !*has_rendered {
                let color = colors.get_color(state);

                if previous_y != y {
                    queue!(
                        terminal,
                        MoveToRow(
                            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                            {
                                y as _
                            }
                        )
                    )?;
                    previous_y = y;
                }

                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                {
                    queue!(
                        terminal,
                        MoveToColumn(
                            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                            {
                                x as _
                            }
                        ),
                        SetBackgroundColor(color),
                        Print(" "),
                    )?;
                }
            }
        }

        *has_rendered = true;

        terminal.flush()?;

        Ok(())
    }

    fn renderer(
        Config {
            rows,
            columns,
            alive_color,
            dead_color,
        }: Config,
    ) -> crossterm::Result<Renderer<Self, crossterm::ErrorKind>> {
        let (columns, rows) = if let (Some(columns), Some(rows)) = (columns, rows) {
            (columns, rows)
        } else {
            let (terminal_columns, terminal_rows) = crossterm::terminal::size()?;
            (
                columns.unwrap_or(terminal_columns as _),
                rows.unwrap_or(terminal_rows as _),
            )
        };

        let state = State::new(columns, rows);
        let backend = Self::new(alive_color.into_color(), dead_color.into_color())?;

        Ok(Renderer::new(state, backend))
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        let Self { terminal, .. } = self;

        disable_raw_mode().unwrap();
        execute!(
            terminal,
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show,
        )
        .unwrap();
    }
}
