mod player;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, time::Duration, path::PathBuf};
use tui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::thread;
use std::sync::mpsc;

struct App {
    music_player: player::MusicPlayer,
    list_state: ListState,
}

impl App {
    fn new() -> App {
        App {
            music_player: player::MusicPlayer::new(),
            list_state: ListState::default(),
        }
    }

    fn on_tick(&mut self) {
        // Update UI state on tick
        if self.music_player.is_playing() && self.music_player.get_progress().unwrap_or(0.0) >= 1.0 {
            if let Err(e) = self.music_player.next_track() {
                eprintln!("Error playing next track: {}", e);
            }
        }
    }
}

enum InputEvent<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    // Input handling thread
    thread::spawn(move || {
        let mut last_tick = std::time::Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap() {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        tx.send(InputEvent::Input(key)).unwrap();
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                tx.send(InputEvent::Tick).unwrap();
                last_tick = std::time::Instant::now();
            }
        }
    });

    let mut app = App::new();

    // Scan music directory
    let music_dir = if let Ok(home) = std::env::var("USERPROFILE") {
        PathBuf::from(home).join("Music")
    } else {
        PathBuf::from(".")
    };

    if music_dir.exists() {
        for entry in std::fs::read_dir(music_dir)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "mp3" || ext == "wav" || ext == "flac" {
                        app.music_player.add_track(path);
                    }
                }
            }
        }
    }

    // Select first track by default
    if !app.music_player.tracks.is_empty() {
        app.list_state.select(Some(0));
    }

    // Main event loop
    loop {
        terminal.draw(|f| ui::draw(f, &app.music_player, &mut app.list_state))?;

        match rx.recv()? {
            InputEvent::Input(event) => match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if !app.music_player.tracks.is_empty() {
                        let i = match app.list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.music_player.tracks.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.list_state.select(Some(i));
                    }
                }
                KeyCode::Down => {
                    if !app.music_player.tracks.is_empty() {
                        let i = match app.list_state.selected() {
                            Some(i) => (i + 1) % app.music_player.tracks.len(),
                            None => 0,
                        };
                        app.list_state.select(Some(i));
                    }
                }
                KeyCode::Enter => {
                    if let Some(i) = app.list_state.selected() {
                        app.music_player.play_track(i)?;
                    }
                }
                KeyCode::Char(' ') => {
                    if app.music_player.is_playing() {
                        app.music_player.pause();
                    } else {
                        app.music_player.play();
                    }
                }
                KeyCode::Char('s') => {
                    app.music_player.stop();
                }
                KeyCode::Right => {
                    app.music_player.next_track()?;
                    if let Some(current) = app.music_player.current_track {
                        app.list_state.select(Some(current));
                    }
                }
                KeyCode::Left => {
                    app.music_player.previous_track()?;
                    if let Some(current) = app.music_player.current_track {
                        app.list_state.select(Some(current));
                    }
                }
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    app.music_player.increase_volume();
                }
                KeyCode::Char('-') => {
                    app.music_player.decrease_volume();
                }
                _ => {}
            },
            InputEvent::Tick => {
                app.on_tick();
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}