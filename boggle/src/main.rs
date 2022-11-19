use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    io::{self, Stdout},
    sync::Mutex,
};
use tui::{backend::CrosstermBackend, Terminal};
use ui::WordEntryResult;

mod ui;

// Dictionary API >> https://dictionaryapi.dev/

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

lazy_static! {
    pub static ref TERMINAL: Mutex<Terminal<CrosstermBackend<Stdout>>> = {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend).unwrap();
        Mutex::new(terminal)
    };
}

pub struct GameState {
    word_entry_buffer: String,
    words_scroll: usize,
    words: HashSet<String>,
}

impl GameState {
    fn new() -> Self {
        Self {
            word_entry_buffer: String::new(),
            words_scroll: 0,
            words: HashSet::new(),
        }
    }
}

fn main() -> AnyResult<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide
    )?;

    let mut state = GameState::new();

    loop {
        let render_result = ui::ui(
            ui::EventHandlers {
                word_entered: |word, state| {
                    if word.contains('s') {
                        WordEntryResult::InvalidWord
                    } else {
                        state.words.insert(word);
                        WordEntryResult::Valid
                    }
                },
            },
            &mut state,
        )?;
        match render_result {
            ui::RenderResult::None => {}
            ui::RenderResult::Exit => break,
        }
    }

    {
        let mut terminal = TERMINAL.lock()?;
        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
    }

    Ok(())
}
