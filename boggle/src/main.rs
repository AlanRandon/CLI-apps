use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
    Terminal,
};
use tui_textarea::{Input, Key, TextArea};

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .title("Enter a valid word")
            .borders(Borders::ALL),
    );
    textarea.set_cursor_style(Style::default().bg(Color::DarkGray));
    textarea.set_cursor_line_style(Style::default());

    loop {
        let mut should_exit = false;

        let root = Block::default().title("Boggle").borders(Borders::ALL);
        let table = Table::new(vec![
            Row::new(vec!["a", "b", "c"]),
            Row::new(vec!["d", "e", "f"]),
            Row::new(vec!["g", "h", "i"]),
        ])
        .widths(&[
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .column_spacing(3);

        terminal.draw(|frame| {
            let size = frame.size();
            let root_inner_rect = root.inner(size);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
                .horizontal_margin(1)
                .split(root_inner_rect);

            let Ok(event) = crossterm::event::read() else {
                return;
            };

            let Ok(event): Result<Input, _> = event.try_into() else {
                return;
            };

            match event {
                Input {
                    key: Key::Esc | Key::Enter,
                    ..
                } => {
                    should_exit = true;
                }
                input => {
                    textarea.input(input);
                }
            }

            frame.render_widget(root, size);
            frame.render_widget(table, layout[0]);
            frame.render_widget(textarea.widget(), layout[1]);
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
