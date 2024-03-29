#![warn(clippy::pedantic)]

use clap::Parser;
use crossterm::event::{EventStream, KeyCode};
use futures::{FutureExt, StreamExt};
use game_of_life_core::prelude::*;
use std::time::Duration;
use tokio::select;

mod terminal;

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
    #[command(flatten)]
    backend_config: terminal::Config,
}

#[tokio::main]
async fn main() -> crossterm::Result<()> {
    let Args {
        delay,
        backend_config,
    } = Args::parse();

    let mut event_stream = EventStream::new();

    let mut ui = terminal::Backend::renderer(backend_config)?;

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
