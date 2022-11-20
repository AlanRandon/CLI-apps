use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use std::{
    collections::HashSet,
    fs,
    io::{self, Stdout},
    sync::Mutex,
};
use tui::{
    backend::CrosstermBackend,
    widgets::{Cell, Row},
    Terminal,
};
use ui::WordEntryResult;

mod ui;

// Dictionary API >> https://dictionaryapi.dev/

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

const ADJACENT_INDICIES: [(i16, i16); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

lazy_static! {
    pub static ref TERMINAL: Mutex<Terminal<CrosstermBackend<Stdout>>> = {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend).unwrap();
        Mutex::new(terminal)
    };
    pub static ref DICE_FACES: Vec<Vec<&'static str>> = {
        include_str!("dice.txt")
            .split('\n')
            .map(|faces| faces.split(',').collect())
            .collect()
    };
}

pub struct BoggleBoard {
    letters: Vec<Vec<String>>,
}

impl BoggleBoard {
    fn new() -> Self {
        let mut dice_faces = DICE_FACES.clone();
        let mut rng = rand::thread_rng();

        dice_faces.shuffle(&mut rng);

        Self {
            letters: dice_faces
                .chunks(4)
                .map(|row| {
                    row.iter()
                        .map(|letters| letters.choose(&mut rng).unwrap().to_string())
                        .collect()
                })
                .collect(),
        }
    }

    fn to_rows(&self) -> Vec<Row> {
        self.letters
            .iter()
            .map(|row| Row::new(row.iter().map(|letter| Cell::from(letter.to_string()))))
            .collect()
    }
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
                word_entered: |mut word, state| {
                    let mut letters = Vec::new();
                    let mut chars = word.chars();
                    while let Some(letter) = chars.next() {
                        if letter == 'q' && chars.next() != Some('u') {
                            return WordEntryResult::InvalidWord;
                        }
                        letters.push(letter);
                    }
                    let mut possible_matches = Vec::new();
                    for (x, row) in state.board.letters.iter().enumerate() {
                        for (y, letter) in row.iter().enumerate() {
                            if word.starts_with(letter) {
                                word = word[letter.len()..].to_string();
                                possible_matches.push((x as i16, y as i16));
                            }
                        }
                    }
                    while !possible_matches.is_empty() {
                        possible_matches = possible_matches
                            .into_iter()
                            .flat_map(|(x, y)| {
                                ADJACENT_INDICIES.iter().filter_map(|(x_diff, y_diff)| {
                                    let index = (x + x_diff, y + y_diff);
                                    None
                                })
                            })
                            .collect()
                    }
                    state.words.insert(word);
                    WordEntryResult::Valid
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
