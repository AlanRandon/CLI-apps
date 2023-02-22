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
use std::io::{self, Write};

pub struct Ui {
    terminal: io::Stdout,
    pub state: State,
}

impl Ui {
    pub fn new(state: State) -> Result<Self> {
        enable_raw_mode()?;
        let mut terminal = io::stdout();
        execute!(
            terminal,
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide,
        )?;

        Ok(Self { terminal, state })
    }

    pub fn render_next_state(&mut self) -> Result<()> {
        let Self { terminal, state } = self;

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

impl Drop for Ui {
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
