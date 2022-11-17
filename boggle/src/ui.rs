use super::WORDS;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use std::{io, sync::RwLock};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Terminal,
};

lazy_static! {
    pub static ref WORDS_SCROLL: RwLock<usize> = RwLock::new(0);
}

pub enum WordEntryResult {
    InvalidWord,
    Valid,
}

pub fn ui<F>(mut on_new_word: F) -> Result<(), io::Error>
where
    F: FnMut(String) -> WordEntryResult,
{
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let mut input_buffer = String::new();
    let mut word_validation = None;

    loop {
        let mut should_exit = false;

        let root = Block::default().title("Boggle").borders(Borders::ALL);
        let game_grid = Table::new(vec![
            Row::new(vec!["a", "b", "c"]),
            Row::new(vec!["d", "e", "qu"]),
            Row::new(vec!["g", "h", "i"]),
        ])
        .widths(&[
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .column_spacing(1)
        .block(Block::default().borders(Borders::ALL));

        terminal.draw(|frame| {
            let size = frame.size();
            let root_inner_rect = root.inner(size);

            frame.render_widget(root, size);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
                .horizontal_margin(1)
                .split(root_inner_rect);

            let top_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Max(14), Constraint::Percentage(50)])
                .split(layout[0]);

            frame.render_widget(game_grid, top_layout[0]);

            let Ok(event) = crossterm::event::read() else {
                return;
            };

            if let Event::Key(key_event) = event {
                if key_event.modifiers.contains(KeyModifiers::NONE) {
                    match key_event.code {
                        KeyCode::Char(character) => {
                            input_buffer.push(character);
                            word_validation = None;
                        }
                        KeyCode::Backspace => {
                            input_buffer.pop();
                        }
                        KeyCode::Esc => should_exit = true,
                        KeyCode::Enter | KeyCode::Tab => {
                            word_validation = Some(on_new_word(std::mem::take(&mut input_buffer)));
                        }
                        KeyCode::Down => {
                            let mut words_scroll = WORDS_SCROLL.write().unwrap();
                            *words_scroll = (words_scroll.saturating_add(1))
                                .clamp(0, WORDS.read().unwrap().len().saturating_sub(1))
                        }
                        KeyCode::Up => {
                            let mut words_scroll = WORDS_SCROLL.write().unwrap();
                            *words_scroll = (words_scroll.saturating_sub(1_usize))
                                .clamp(0, WORDS.read().unwrap().len().saturating_sub(1))
                        }
                        _ => (),
                    }
                }
            }

            let input = Paragraph::new(Spans::from(vec![
                Span::raw(input_buffer.clone()),
                Span::styled(" ", Style::default().bg(Color::DarkGray)),
            ]))
            .block(
                Block::default()
                    .title(match word_validation {
                        Some(WordEntryResult::Valid) => "Valid Word Found",
                        Some(WordEntryResult::InvalidWord) => "Word Invalid",
                        None => "Enter a valid word",
                    })
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(match word_validation {
                        Some(WordEntryResult::Valid) => Color::Green,
                        Some(WordEntryResult::InvalidWord) => Color::Red,
                        None => Color::Reset,
                    })),
            );

            frame.render_widget(input, layout[1]);

            let words_list = {
                let words = WORDS.read().unwrap();
                List::new(if words.is_empty() {
                    Vec::new()
                } else {
                    let mut words: Vec<_> = words.iter().collect();

                    words.sort();

                    words
                        .get(*WORDS_SCROLL.read().unwrap()..words.len())
                        .unwrap()
                        .iter()
                        .map(|&word| ListItem::new(word.clone()))
                        .collect()
                })
                .block(Block::default().title("Words").borders(Borders::ALL))
            };

            frame.render_widget(words_list, top_layout[1]);

            frame.set_cursor(size.width, size.height);
        })?;

        if should_exit {
            break;
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
