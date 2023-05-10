use crossterm::event::{Event, EventStream, KeyCode};
use futures::{FutureExt, StreamExt};
use game::{GameState, Win};
use std::time::Duration;
use terminal::{enter_alternate_mode, exit_alternate_mode, Terminal};

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
    let mut events = Vec::new();

    enter_alternate_mode(&mut stdout)?;

    'a: loop {
        if let Some(win) = game_state.update(events.drain(..)) {
            exit_alternate_mode(&mut stdout)?;
            let win_message = match win {
                Win::Left => "Left has won.",
                Win::Right => "Right has won.",
            };
            println!("{win_message}");
            break;
        };
        terminal.render(&game_state);
        terminal.render_to_stdout(&mut stdout)?;

        let sleeper = tokio::time::sleep(Duration::from_millis(20)).fuse();

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
                EventPoll::Quit => {
                    exit_alternate_mode(&mut stdout)?;
                    break 'a;
                }
                EventPoll::Event(event) => events.push(event),
            }
        }
    }

    Ok(())
}
