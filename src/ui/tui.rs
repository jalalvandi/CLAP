// src/ui/tui.rs

use crossterm::{execute, terminal::{self, ClearType}};
use std::io::{self, Write};

pub fn init_terminal() -> Result<(), io::Error> {
    execute!(io::stdout(), terminal::Clear(ClearType::All), terminal::EnterAlternateScreen)?;
    Ok(())
}

pub fn cleanup_terminal() -> Result<(), io::Error> {
    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
    terminal::enable_raw_mode()?;
    Ok(())
}

pub fn render_current_track(track_name: &str) -> Result<(), io::Error> {
    println!("Now Playing: {}", track_name);
    Ok(())
}

pub fn render_controls() -> Result<(), io::Error> {
    println!("Controls:");
    println!("  [P] Play");
    println!("  [S] Stop");
    println!("  [Q] Quit");
    Ok(())
}