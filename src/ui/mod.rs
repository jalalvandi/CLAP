use crate::player::MusicPlayer;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, music_player: &MusicPlayer, list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ].as_ref())
        .split(f.size());

    // Playlist
    let items: Vec<ListItem> = music_player
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let filename = track.file_name().unwrap().to_str().unwrap();
            let prefix = if Some(i) == music_player.current_track {
                if music_player.is_playing() { "▶ " } else { "■ " }
            } else {
                "  "
            };
            ListItem::new(format!("{}{}", prefix, filename))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Playlist").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], list_state);

    // Progress display
    let progress_text = Paragraph::new(format!(
        "{}\nTime: {}", 
        if let Some(progress) = music_player.get_progress() {
            format!("Progress: {}%", (progress * 100.0) as u8)
        } else {
            String::from("Not playing")
        },
        music_player.get_elapsed_time()
    ))
    .block(Block::default().title("Progress").borders(Borders::ALL))
    .alignment(Alignment::Center);
    
    f.render_widget(progress_text, chunks[1]);

    // Status with current track name
    let status = if let Some(current) = music_player.current_track {
        let track_name = music_player.tracks[current]
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        format!(
            "Now Playing: {} | Volume: {}% | Status: {}",
            track_name,
            (music_player.volume * 100.0) as i32,
            if music_player.is_playing() { "Playing" } else { "Paused" }
        )
    } else {
        String::from("No track selected")
    };

    let status_widget = Paragraph::new(status)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_widget, chunks[2]);

    // Controls
    let controls = Paragraph::new(
        "Controls:\n\
         ↑/↓: Navigate playlist | ←/→: Previous/Next track\n\
         p: Play/Pause | s: Stop | +/-: Volume | q: Quit"
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[3]);
}