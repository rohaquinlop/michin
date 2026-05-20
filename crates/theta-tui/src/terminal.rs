//! Terminal setup and teardown.

use std::io::{self, Stdout};

use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

/// Setup raw mode and alternate screen.
pub fn setup() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    io::stdout().execute(EnableMouseCapture)?;
    Ok(())
}

/// Restore terminal to normal mode.
pub fn restore() -> io::Result<()> {
    io::stdout().execute(DisableMouseCapture)?;
    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// Get the terminal size as (cols, rows).
pub fn size() -> io::Result<(u16, u16)> {
    crossterm::terminal::size()
}

/// Create a ratatui Terminal with Crossterm backend on stdout.
pub fn create_terminal() -> io::Result<ratatui::Terminal<ratatui::backend::CrosstermBackend<Stdout>>>
{
    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    ratatui::Terminal::new(backend)
}
