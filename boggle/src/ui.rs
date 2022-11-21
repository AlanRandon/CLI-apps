use crate::{AnyResult, GameState, TERMINAL};
use crossterm::event::{Event, KeyCode, KeyModifiers};
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

pub struct EventHandlers<F>
where
    F: FnMut(String, &mut GameState) -> WordEntryResult,
{
    pub word_entered: F,
}

pub enum RenderResult {
    None,
    Exit,
}

pub fn ui<F>(
    mut event_handlers: EventHandlers<F>,
    game_state: &mut GameState,
) -> AnyResult<RenderResult>
where
    F: FnMut(String, &mut GameState) -> WordEntryResult,
{
    let mut result = RenderResult::None;
    let mut terminal = TERMINAL.lock().unwrap();

    terminal.hide_cursor()?;

    let root = Block::default().title("Boggle").borders(Borders::ALL);

    terminal.draw(|frame| {
        let size = frame.size();
        let root_inner_rect = root.inner(size);
        let mut word_validation_result = None;

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .horizontal_margin(1)
            .split(root_inner_rect);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);

        let Ok(event) = crossterm::event::read() else {
                return;
            };

        if let Event::Key(key_event) = event {
            if key_event.modifiers.contains(KeyModifiers::NONE) {
                match key_event.code {
                    KeyCode::Char(character) => {
                        game_state.word_entry_buffer.push(character);
                        word_validation_result = None;
                    }
                    KeyCode::Backspace => {
                        game_state.word_entry_buffer.pop();
                    }
                    KeyCode::Esc => result = RenderResult::Exit,
                    KeyCode::Enter | KeyCode::Tab => {
                        word_validation_result = Some((event_handlers.word_entered)(
                            std::mem::take(&mut game_state.word_entry_buffer).to_uppercase(),
                            game_state,
                        ));
                    }
                    KeyCode::Down => {
                        let GameState { words_scroll, .. } = game_state;
                        *words_scroll = words_scroll.saturating_add(1);
                    }
                    KeyCode::Up => {
                        let GameState { words_scroll, .. } = game_state;
                        *words_scroll = words_scroll.saturating_sub(1_usize);
                    }
                    _ => (),
                }
            }
        }

        let game_grid = Table::new(game_state.board.to_rows())
            .widths(&[
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(4),
            ])
            .column_spacing(1)
            .block(Block::default().borders(Borders::ALL));

        let input = Paragraph::new(Spans::from(vec![
            Span::raw(game_state.word_entry_buffer.clone()),
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

        let words_list = {
            let GameState {
                words_scroll,
                words,
                ..
            } = game_state;

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
            .block(Block::default().title("Words").borders(Borders::ALL))
        };

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
