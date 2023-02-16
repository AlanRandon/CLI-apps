use crate::{
    prelude::*,
    state::{State, ToTable},
};
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Clear},
    Terminal,
};

pub struct Ui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub state: State,
}

impl Ui {
    pub fn new(state: State) -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend).unwrap();

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;

        Ok(Self { terminal, state })
    }

    pub fn render(&mut self) -> Result<()> {
        let Self { terminal, state } = self;

        let root_block = Block::default().title("Game Of Life").borders(Borders::ALL);
        let ToTable {
            table,
            width_constraints,
        } = state.into();
        let table = table.widths(&width_constraints).block(root_block);

        terminal.draw(|frame| {
            let size = frame.size();

            frame.render_widget(Clear, size);
            frame.render_widget(table.clone(), size);
            frame.set_cursor(size.width, size.height);
        })?;

        Ok(())
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        let Self { terminal, .. } = self;

        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        terminal.show_cursor().unwrap();
    }
}
