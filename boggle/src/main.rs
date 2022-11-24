use board::BoggleBoard;
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
    time::Duration,
};
use tokio::runtime::Handle;
use tui::{backend::CrosstermBackend, Terminal};
use ui::WordEntryResult;

mod board;
mod ui;

type AnyError = Box<dyn std::error::Error>;
type AnyResult<T> = Result<T, AnyError>;

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
    board: BoggleBoard,
}

impl GameState {
    fn new() -> Self {
        Self {
            word_entry_buffer: String::new(),
            words_scroll: 0,
            words: HashSet::new(),
            board: BoggleBoard::new(),
        }
    }

    fn get_score(&self) -> u32 {
        self.words
            .iter()
            .map(|word| match word.len() {
                0..=2 => 0,
                3..=4 => 1,
                5 => 2,
                6 => 3,
                7 => 4,
                _ => 11,
            })
            .sum()
    }
}

fn main() -> AnyResult<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

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

    let gameloop = async {
        let handle = Handle::current();

        loop {
            let render_result = ui::ui(
                ui::EventHandlers {
                    word_entered: |word, state| {
                        let is_valid = handle
                            .spawn_blocking(|| state.board.is_valid_word(&word))
                            .unwrap();
                        if is_valid {
                            state.words.insert(word);
                            WordEntryResult::Valid
                        } else {
                            WordEntryResult::InvalidWord
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
        Ok::<_, AnyError>(())
    };

    let timer = async {
        tokio::time::sleep(Duration::from_secs(/* 180 */ 30)).await;
    };

    runtime.block_on(async {
        tokio::select! {
            result = gameloop => result,
            _ = timer => Ok(())
        }
    })?;

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

    println!("Score: {}", state.get_score());

    Ok(())
}
