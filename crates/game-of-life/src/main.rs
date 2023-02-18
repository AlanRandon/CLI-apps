#![warn(clippy::pedantic)]

use clap::Parser;
use crossterm::event::{EventStream, KeyCode};
use futures::{FutureExt, StreamExt};
use prelude::*;
use state::State;
use std::time::Duration;
use tokio::select;
use ui::Ui;

mod cell;
mod state;
mod ui;

/// An implementation of the game of life in rust.
///
/// The width and height of the board will default to the size of the terminal - this may be slow as much more must be computed.
/// If you would like to use a smaller 10*10 board with a faster update time, here are some reasonable defaults
/// ```sh
/// game-of-life -r 10 -c 10 -d 100
/// ```
#[derive(Parser)]
struct Args {
    /// The delay to wait before updating the board (in milliseconds)
    #[clap(long, short, default_value_t = 500)]
    delay: u64,
    /// The height of the board (in rows)
    #[clap(short = 'r', long)]
    rows: Option<usize>,
    /// The width of the board (in columns)
    #[clap(short = 'c', long)]
    columns: Option<usize>,
}

mod prelude {
    pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
}

#[tokio::main]
async fn main() -> Result<()> {
    let Args {
        delay,
        columns,
        rows,
    } = Args::parse();

    let (columns, rows) = if let (Some(columns), Some(rows)) = (columns, rows) {
        (columns, rows)
    } else {
        let (terminal_columns, terminal_rows) = crossterm::terminal::size()?;
        (
            columns.unwrap_or(terminal_columns as usize - 2),
            rows.unwrap_or(terminal_rows as usize - 2),
        )
    };

    let mut ui = Ui::new(State::new(columns, rows))?;

    let mut event_stream = EventStream::new();

    loop {
        ui.render()?;

        let event_listener = event_stream.next().fuse();
        let sleeper = tokio::time::sleep(Duration::from_millis(delay));

        select! {
            event = event_listener => {
                let Some(event) = event else {
                    continue;
                };
                let event = event?;
                if event == crossterm::event::Event::Key(KeyCode::Esc.into()) {
                    break
                };
            }
            _ = sleeper => {
                ui.state.tick();
            }
        }
    }
    Ok(())
}
