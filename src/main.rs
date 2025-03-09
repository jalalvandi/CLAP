mod player;
mod ui;
mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use player::MusicPlayer;
use std::{
    error::Error,
    io,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::env;
use utils::scan_music_directory;

enum InputEvent<I> {
    Input(I),
    Tick,
}

struct App {
    music_player: MusicPlayer,
    list_state: ListState,
    search_query: String,
    is_searching: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    tx.send(InputEvent::Input(key)).unwrap();
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(InputEvent::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let mut music_player = MusicPlayer::new();
    
    // Get music directory path
    let music_dir = if let Ok(home) = env::var("USERPROFILE") {
        PathBuf::from(home).join("Music")
    } else {
        PathBuf::from(".")
    };

    // Scan for music files
    let music_files = scan_music_directory(&music_dir);
    for track in music_files {
        music_player.add_track(track);
    }

    let mut list_state = ListState::default();
    if !music_player.tracks.is_empty() {
        list_state.select(Some(0));
    }

    let mut app = App {
        music_player,
        list_state,
        search_query: String::new(),
        is_searching: false,
    };

    loop {
        terminal.draw(|f| ui::draw(f, &app.music_player, &mut app.list_state))?;

        match rx.recv()? {
            InputEvent::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('p') => {
                    if app.music_player.is_playing() {
                        app.music_player.pause();
                    } else {
                        app.music_player.play();
                    }
                }
                KeyCode::Char('s') => {
                    app.music_player.stop();
                }
                KeyCode::Down => {
                    let i = match app.list_state.selected() {
                        Some(i) => {
                            if i >= app.music_player.tracks.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    app.list_state.select(Some(i));
                    app.music_player.play_track(i)?;
                }
                KeyCode::Up => {
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
                    app.music_player.play_track(i)?;
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
                KeyCode::Char('/') => {
                    app.is_searching = true;
                    app.search_query.clear();
                }
                KeyCode::Esc => {
                    app.is_searching = false;
                    app.search_query.clear();
                }
                KeyCode::Char(c) if app.is_searching => {
                    app.search_query.push(c.to_lowercase().next().unwrap());
                    // Filter the track list based on search
                    if let Some(index) = app.music_player.tracks.iter().position(|track| {
                        track.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_lowercase()
                            .contains(&app.search_query)
                    }) {
                        app.list_state.select(Some(index));
                    }
                }
                _ => {}
            },
            InputEvent::Tick => {
                // This will redraw the UI every tick (250ms)
                terminal.draw(|f| ui::draw(f, &app.music_player, &mut app.list_state))?;
            }
        }
    }

    Ok(())
}