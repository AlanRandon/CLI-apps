use crossterm::{
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use futures::{join, FutureExt, StreamExt, TryStreamExt};
use game::GameState;
use std::time::Duration;
use terminal::Terminal;

mod game;
mod terminal;

enum EventPoll {
    Quit,
    Event(game::Event),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new()?;
    let mut game_state = GameState::new(&terminal);
    let mut stdout = std::io::stdout();
    let mut event_stream = EventStream::new();
    let mut events = Vec::new();

    enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide,
    )?;

    // .(
    //     |event| !matches!(event, Event::Key(key_event) if key_event.code == KeyCode::Char('q')),
    // );

    'a: loop {
        // stdout.execute(crossterm::terminal::Clear(
        //     crossterm::terminal::ClearType::Purge,
        // ));
        game_state.update(events.drain(..));
        terminal.render(&game_state);
        terminal.render_to_stdout(&mut stdout)?;

        let sleeper = tokio::time::sleep(Duration::from_millis(100)).fuse();

        let mut event_stream = EventStream::new()
            .filter_map(|event| async { event.ok() })
            .filter_map(|event| async {
                if let Event::Key(event) = event {
                    if event.code == KeyCode::Char('q') {
                        return Some(EventPoll::Quit);
                    }
                }

                if let Some(event) = game::Event::from_crossterm(event) {
                    return Some(EventPoll::Event(event));
                }

                None
            })
            .take_until(sleeper)
            .boxed();

        while let Some(event) = event_stream.next().await {
            match event {
                EventPoll::Quit => break 'a,
                EventPoll::Event(event) => events.push(event),
            }
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
