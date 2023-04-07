use crossterm::{
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Color, PrintStyledContent, Stylize},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    QueueableCommand,
};
use ndarray::Array2;
use std::{
    io::{Stdout, Write},
    time::Duration,
};

struct Game;

#[derive(Clone, Copy)]
enum TerminalCell {
    Ball,
    Paddle,
    Empty,
}

impl TerminalCell {
    fn queue_command(&self, stdout: &mut Stdout) -> crossterm::Result<()> {
        stdout.queue(PrintStyledContent(" ".on(match self {
            Self::Ball => Color::Cyan,
            Self::Paddle => Color::White,
            Self::Empty => Color::Reset,
        })))?;
        Ok(())
    }
}

struct Terminal {
    cells: Array2<TerminalCell>,
}

impl Terminal {
    pub fn new() -> std::io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self::from_size(width as usize, height as usize))
    }

    pub fn from_size(width: usize, height: usize) -> Self {
        Self {
            cells: Array2::from_elem((width, height), TerminalCell::Empty),
        }
    }

    pub fn render(&self, game: &Game, stdout: &mut Stdout) -> crossterm::Result<()> {
        stdout.queue(MoveTo(0, 0))?;
        for cell in self.cells.iter() {
            cell.queue_command(stdout)?;
        }
        stdout.flush()?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let terminal = Terminal::new()?;
    let mut stdout = std::io::stdout();

    enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide,
    )?;

    terminal.render(&Game, &mut stdout)?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    disable_raw_mode()?;
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::Show,
    )?;

    Ok(())
}
