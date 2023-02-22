#![warn(clippy::pedantic)]

use clap::Parser;
use crossterm::{
    event::{EventStream, KeyCode},
    style::Color,
};
use futures::{FutureExt, StreamExt};
use prelude::*;
use state::State;
use std::{fmt, time::Duration};
use tokio::select;
use ui::Ui;

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

    #[derive(Clone, Copy)]
    pub struct Coordinates {
        pub x: i128,
        pub y: i128,
    }
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
            columns.unwrap_or(terminal_columns as _),
            rows.unwrap_or(terminal_rows as _),
        )
    };

    let mut event_stream = EventStream::new();

    let state = State::new(columns, rows);

    let mut ui = Ui::new(state)?;

    loop {
        ui.render_next_state()?;

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
            _ = sleeper => {}
        }
    }
    Ok(())
}
