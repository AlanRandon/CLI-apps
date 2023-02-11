// TODO: Game of life itself
// TODO: Setup a `State` struct to hold the world state
// TODO: Use tokio to deal with events at the same time that calculations are done for the next tick

use crossterm::event::{Event, KeyCode};
use prelude::*;
use std::time::Duration;
use ui::Ui;

mod cell;
mod ui;

mod prelude {
    pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
}

fn main() -> Result<()> {
    let mut ui = Ui::new()?;
    loop {
        ui.render()?;

        if let Ok(true) = crossterm::event::poll(Duration::from_millis(500)) {
            if let Ok(Event::Key(key_event)) = crossterm::event::read() {
                if key_event.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }
    Ok(())
}
