use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use game::GameState;
use std::time::Duration;
use terminal::Terminal;

mod game;
mod terminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new()?;
    let game_state = GameState::new(&terminal);
    let mut stdout = std::io::stdout();

    enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide,
    )?;

    terminal.render(&game_state);
    terminal.render_to_stdout(&mut stdout)?;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    disable_raw_mode()?;
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::Show,
    )?;

    dbg!(&game_state);

    Ok(())
}
