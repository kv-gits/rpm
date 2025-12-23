use crate::db::Database;
use crate::crypto::CryptoManager;
use crate::errors::RpmResult;
use crate::tray::TrayHandle;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use ratatui::Terminal;
use std::io;

pub struct TuiState {
    pub should_quit: bool,
    pub selected_index: usize,
}

pub async fn run_tui(
    db: Database,
    crypto: CryptoManager,
    _tray: TrayHandle,
) -> RpmResult<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState {
        should_quit: false,
        selected_index: 0,
    };
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| ui(f, &state, &mut list_state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        state.should_quit = true;
                    }
                    KeyCode::Up => {
                        if state.selected_index > 0 {
                            state.selected_index -= 1;
                            list_state.select(Some(state.selected_index));
                        }
                    }
                    KeyCode::Down => {
                        state.selected_index += 1;
                        list_state.select(Some(state.selected_index));
                    }
                    _ => {}
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, state: &TuiState, list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("RPM - Rust Password Manager")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content area
    let items: Vec<ListItem> = vec![
        ListItem::new("Password Entry 1"),
        ListItem::new("Password Entry 2"),
        ListItem::new("Password Entry 3"),
    ];

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Passwords"))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], list_state);

    // Footer
    let footer = Paragraph::new("Press 'q' to quit | ↑↓ to navigate")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

