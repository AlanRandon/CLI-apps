// TODO: Use tokio to deal with events at the same time that calculations are done for the next tick

use crossterm::event::{Event, KeyCode};
use prelude::*;
use state::State;
use std::time::Duration;
use ui::Ui;

mod cell;
mod state;
mod ui;

mod prelude {
    pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
}

fn main() -> Result<()> {
    let mut ui = Ui::new(State::new(10, 10))?;
    loop {
        ui.render()?;

        if crossterm::event::poll(Duration::from_millis(500))? {
            let Ok(Event::Key(key_event)) = crossterm::event::read() else {
                 continue;
            };
            match key_event.code {
                KeyCode::Esc => break,
                KeyCode::Tab => {
                    ui.state.tick();
                }
                _ => {}
            };
        }
    }
    Ok(())
}
