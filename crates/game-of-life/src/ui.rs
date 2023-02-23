use crate::{
    prelude::*,
    state::{CellState, State},
};
use crossterm::{
    cursor::{self, MoveToColumn, MoveToRow},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Color, ContentStyle, Print, SetStyle},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Write},
    marker::PhantomData,
};

pub trait RendererBackend<RenderError>: Sized
where
    RenderError: std::error::Error,
{
    fn render_next_state(&mut self, state: &mut State) -> Result<(), RenderError>;
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
    fn new(state: State, backend: B) -> Self {
        Self {
            state,
            backend,
            _phantom: PhantomData,
        }
    }

    pub fn render_next_state(&mut self) -> Result<(), E> {
        let Self { state, backend, .. } = self;
        backend.render_next_state(state)
    }
}

pub struct CrosstermBackend {
    terminal: io::Stdout,
}

#[derive(clap::Args)]
pub struct TerminalSizeArgs {
    /// The height of the board (in rows)
    #[clap(short = 'r', long)]
    rows: Option<usize>,
    /// The width of the board (in columns)
    #[clap(short = 'c', long)]
    columns: Option<usize>,
}

impl CrosstermBackend {
    pub fn new() -> crossterm::Result<Self> {
        enable_raw_mode()?;
        let mut terminal = io::stdout();
        execute!(
            terminal,
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide,
        )?;

        Ok(Self { terminal })
    }

    pub fn new_renderer(
        TerminalSizeArgs { rows, columns }: TerminalSizeArgs,
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
        let backend = Self::new()?;

        Ok(Renderer::new(state, backend))
    }
}

impl RendererBackend<crossterm::ErrorKind> for CrosstermBackend {
    fn render_next_state(&mut self, state: &mut State) -> crossterm::Result<()> {
        let Self { terminal } = self;

        for (Coordinates { x, y }, state) in state.next_state() {
            let style = ContentStyle {
                background_color: Some(if state == CellState::Alive {
                    Color::White
                } else {
                    Color::Black
                }),
                ..Default::default()
            };

            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            {
                queue!(
                    terminal,
                    MoveToColumn(x as _),
                    MoveToRow(y as _),
                    SetStyle(style),
                    Print(" ")
                )?;
            }
        }

        queue!(terminal, cursor::MoveTo(0, 0))?;

        terminal.flush()?;

        Ok(())
    }
}

impl Drop for CrosstermBackend {
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
