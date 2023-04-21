use crossterm::{
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use futures::{join, FutureExt, StreamExt};
use game::GameState;
use std::time::Duration;
use terminal::Terminal;

mod game;
mod terminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new()?;
    let mut game_state = GameState::new(&terminal);
    let mut stdout = std::io::stdout();
    let mut event_stream = EventStream::new();

    enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide,
    )?;

    loop {
        // stdout.execute(crossterm::terminal::Clear(
        //     crossterm::terminal::ClearType::Purge,
        // ));
        let debug_text = game_state.update();
        terminal.render(&game_state);
        terminal.render_to_stdout(&mut stdout)?;

        if let Some(debug_text) = debug_text {
            // debug current state
            std::io::stdout()
                .execute(MoveTo(0, 0))
                .unwrap()
                .execute(Print(format!("Debug: {debug_text}\n")))
                .unwrap();
        }

        let sleeper = tokio::time::sleep(Duration::from_millis(100)).fuse();

        tokio::select! {
            event = event_stream.next().fuse() => {
                let Some(event) = event else {
                    continue;
                };
                let event = event?;
                if event == Event::Key(KeyCode::Char('q').into()) {
                    break
                };
            }
            _ = sleeper => {}
        }
    }

    disable_raw_mode()?;
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::Show,
    )?;

    Ok(())
}
