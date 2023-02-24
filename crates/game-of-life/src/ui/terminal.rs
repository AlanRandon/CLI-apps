use super::{Renderer, RendererBackend};
use crate::{
    prelude::*,
    state::{CellState, State},
};
pub use config::Config;
use crossterm::{
    cursor::{self, MoveToColumn, MoveToRow},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Color, Print, SetBackgroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Stdout, Write};

pub struct Backend {
    terminal: Stdout,
    alive_color: Color,
    dead_color: Color,
}

mod config {
    use clap::Args;
    use crossterm::style::Color;
    use std::{fmt, str::FromStr};

    #[derive(Args)]
    pub struct Config {
        /// The height of the board (in rows)
        #[clap(short = 'r', long)]
        pub rows: Option<usize>,
        /// The width of the board (in columns)
        #[clap(short = 'c', long)]
        pub columns: Option<usize>,
        /// The color of an alive cell as an ANSI color
        #[clap(long, value_parser = color_parser, default_value = "255")]
        pub alive_color: ColorWrapper,
        /// The color of an alive cell as an ANSI color
        #[clap(long, value_parser = color_parser, default_value = "0")]
        pub dead_color: ColorWrapper,
    }

    #[derive(Clone)]
    pub struct ColorWrapper {
        color: Color,
    }

    impl fmt::Display for ColorWrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self.color)
        }
    }

    impl ColorWrapper {
        fn new(color: Color) -> Self {
            Self { color }
        }

        pub fn into_color(self) -> Color {
            self.color
        }
    }

    // TODO: add a better parser to handle hex codes + named colors
    fn color_parser(input: &str) -> Result<ColorWrapper, <u8 as FromStr>::Err> {
        let input = input.parse()?;
        Ok(ColorWrapper::new(Color::AnsiValue(input)))
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
            alive_color,
            dead_color,
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
            alive_color,
            dead_color,
        } = self;

        let mut previous_y = -10_i32;

        for CellRenderInfo {
            coordinates: Coordinates { x, y },
            state,
            needs_rerender,
        } in state
        {
            if needs_rerender {
                // TODO: put this if in a `get_color` method - separate color state
                let color = if state == CellState::Alive {
                    *alive_color
                } else {
                    *dead_color
                };

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
