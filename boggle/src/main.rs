use board::BoggleBoard;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use std::{
    cell::RefCell,
    collections::HashSet,
    io::{self, Stdout},
    rc::Rc,
    sync::Mutex,
    time::Duration,
};
use tokio::{sync::oneshot::channel, time::Instant};
use tui::{backend::CrosstermBackend, Terminal};
use ui::WordEntryResult;

mod board;
mod ui;

type AnyError = Box<dyn std::error::Error + Sync + Send>;
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
    deadline: Instant,
}

impl GameState {
    fn new(deadline: Instant) -> Self {
        Self {
            word_entry_buffer: String::new(),
            words_scroll: 0,
            words: HashSet::new(),
            board: BoggleBoard::new(),
            deadline,
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

#[tokio::main]
async fn main() -> AnyResult<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide
    )?;

    let timer = tokio::time::sleep(Duration::from_secs(180));

    let mut state = GameState::new(timer.deadline());

    let gameloop = async {
        loop {
            let render_result = ui::ui(
                ui::EventHandlers::new(|word, board, words| async {
                    let Ok(is_valid) = board.is_valid_word(&word.clone()).await else {
                        return WordEntryResult::InvalidWord;
                    };

                    if is_valid {
                        words.insert(word);
                        WordEntryResult::Valid
                    } else {
                        WordEntryResult::InvalidWord
                    }
                }),
                &mut state,
            )
            .await?;
            match render_result {
                ui::RenderResult::None => {}
                ui::RenderResult::Exit => break,
            }
            tokio::task::yield_now().await;
        }
        Ok::<_, AnyError>(())
    };

    tokio::select! {
        _ = timer => Ok(()),
        result = gameloop => result
    }?;

    {
        let mut terminal = TERMINAL.lock().unwrap();
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
