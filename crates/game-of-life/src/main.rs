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

mod prelude {
    pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
}

#[tokio::main]
async fn main() -> Result<()> {
    let (columns, rows) = crossterm::terminal::size()?;
    let mut ui = Ui::new(State::new(columns as usize - 2, rows as usize - 2))?;
    let mut event_stream = EventStream::new();

    loop {
        ui.render()?;

        let event_listener = event_stream.next().fuse();
        let sleeper = tokio::time::sleep(Duration::from_millis(500));

        select! {
            event = event_listener => {
                let Some(event) = event else {
                    continue;
                };
                let event = event?;
                if event == crossterm::event::Event::Key(KeyCode::Esc.into()) {
                    break
                }
            }
            _ = sleeper => {
                ui.state.tick()
            }
        }
    }
    Ok(())
}
