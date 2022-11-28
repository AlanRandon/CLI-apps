use crate::{AnyResult, GameState, TERMINAL};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use futures::future::BoxFuture;
use std::time::Duration;
use tokio::time::Instant;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Table},
};

pub enum WordEntryResult {
    InvalidWord,
    Valid,
}

type WordEnteredCallback =
    dyn for<'a> FnMut(String, &'a mut GameState) -> BoxFuture<'a, WordEntryResult>;

pub struct EventHandlers<'a> {
    pub word_entered: &'a mut WordEnteredCallback,
}

impl<'a> EventHandlers<'a> {
    pub fn new(word_entered: &'a mut WordEnteredCallback) -> Self {
        Self { word_entered }
    }
}

pub enum RenderResult {
    None,
    Exit,
}

pub async fn handle_events<'a>(
    event_handlers: EventHandlers<'a>,
    state: &mut GameState,
) -> (Option<WordEntryResult>, RenderResult) {
    let GameState {
        word_entry_buffer,
        words_scroll,
        ..
    } = state;

    let mut result = (None, RenderResult::None);
    if let Ok(true) = crossterm::event::poll(Duration::from_millis(500)) {
        if let Ok(Event::Key(key_event)) = crossterm::event::read() {
            if key_event.modifiers.contains(KeyModifiers::NONE) {
                match key_event.code {
                    KeyCode::Char(character) => {
                        word_entry_buffer.push(character);
                    }
                    KeyCode::Backspace => {
                        word_entry_buffer.pop();
                    }
                    KeyCode::Esc => result.1 = RenderResult::Exit,
                    KeyCode::Enter | KeyCode::Tab => {
                        result.0 = Some(
                            (event_handlers.word_entered)(
                                std::mem::take(word_entry_buffer).to_uppercase(),
                                state,
                            )
                            .as_mut()
                            .await,
                        );
                    }
                    KeyCode::Down => {
                        *words_scroll = words_scroll.saturating_add(1);
                    }
                    KeyCode::Up => {
                        *words_scroll = words_scroll.saturating_sub(1_usize);
                    }
                    _ => (),
                }
            }
        }
    }

    result
}

pub async fn ui<'a>(
    event_handlers: EventHandlers<'a>,
    state: &mut GameState,
) -> AnyResult<RenderResult> {
    let mut terminal = TERMINAL.lock().await;

    terminal.hide_cursor()?;

    let root = Block::default()
        .title(format!(
            "Boggle: {} seconds remaining",
            state.deadline.duration_since(Instant::now()).as_secs(),
        ))
        .borders(Borders::ALL);

    let (word_validation_result, result) = handle_events(event_handlers, state).await;

    let GameState {
        word_entry_buffer,
        words_scroll,
        words,
        board,
        ..
    } = state;

    let game_grid = Table::new(board.to_rows())
        .widths(&[
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ])
        .column_spacing(1)
        .block(Block::default().borders(Borders::ALL));

    let words_list = {
        List::new(if words.is_empty() {
            Vec::new()
        } else {
            let mut words: Vec<_> = words.iter().collect();

            words.sort();

            words
                .get((*words_scroll).clamp(0, words.len().saturating_sub(1))..words.len())
                .unwrap()
                .iter()
                .map(|&word| ListItem::new(word.clone()))
                .collect()
        })
        .block(
            Block::default()
                .title(format!("Words | {} discovered", words.len()))
                .borders(Borders::ALL),
        )
    };

    let input = Paragraph::new(Spans::from(vec![
        Span::raw(word_entry_buffer.clone()),
        Span::styled(" ", Style::default().bg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .title(match word_validation_result {
                Some(WordEntryResult::Valid) => "Valid Word Found",
                Some(WordEntryResult::InvalidWord) => "Word Invalid",
                None => "Enter a valid word",
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(match word_validation_result {
                Some(WordEntryResult::Valid) => Color::Green,
                Some(WordEntryResult::InvalidWord) => Color::Red,
                None => Color::Reset,
            })),
    );

    terminal.draw(|frame| {
        let size = frame.size();
        let root_inner_rect = root.inner(size);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .horizontal_margin(1)
            .split(root_inner_rect);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);

        frame.render_widget(Clear, size);
        frame.render_widget(root, size);
        frame.render_widget(game_grid, top_layout[0]);
        frame.render_widget(words_list, top_layout[1]);
        frame.render_widget(input, layout[1]);
        frame.set_cursor(size.width, size.height);
    })?;

    terminal.autoresize()?;

    Ok(result)
}
