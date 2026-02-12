mod app;
mod event;
mod ui;

use std::io;
use std::panic;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use event::{poll_event, AppEvent};

fn main() -> anyhow::Result<()> {
    // Panic hook to restore terminal
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen, cursor::Show);
        default_hook(info);
    }));

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    while app.running {
        terminal.draw(|f| ui::draw(f, &app))?;

        match poll_event(Duration::from_millis(250))? {
            AppEvent::Key(key) => handle_key(&mut app, key),
            AppEvent::Tick => app.tick(),
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.running = false,
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => app.running = false,
        (KeyCode::Char(' '), _) => app.toggle_play_pause(),
        (KeyCode::Char('n'), _) => app.next_track(),
        (KeyCode::Char('p'), _) => app.prev_track(),
        (KeyCode::Char('+') | KeyCode::Char('='), _) => app.volume_up(),
        (KeyCode::Char('-'), _) => app.volume_down(),
        (KeyCode::Left, _) => app.seek_backward(),
        (KeyCode::Right, _) => app.seek_forward(),
        (KeyCode::Tab, _) => app.next_player(),
        (KeyCode::BackTab, _) => app.prev_player(),
        (KeyCode::Char('l'), _) => app.cycle_loop(),
        (KeyCode::Char('s'), _) => app.toggle_shuffle(),
        _ => {}
    }
}
