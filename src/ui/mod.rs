use crate::player::MusicPlayer;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Sparkline},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn draw<B: Backend>(f: &mut Frame<B>, music_player: &MusicPlayer, list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Playlist
            Constraint::Length(3),       // Progress bar
            Constraint::Length(3),       // Status
            Constraint::Length(3),       // Controls
        ].as_ref())
        .margin(1)
        .split(f.size());

    draw_playlist(f, music_player, list_state, chunks[0]);
    draw_progress(f, music_player, chunks[1]);
    draw_status(f, music_player, chunks[2]);
    draw_controls(f, chunks[3]);
}

fn draw_playlist<B: Backend>(
    f: &mut Frame<B>,
    music_player: &MusicPlayer,
    list_state: &mut ListState,
    area: Rect,
) {
    let items: Vec<ListItem> = music_player
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let filename = track.file_name().unwrap().to_str().unwrap();
            let prefix = if Some(i) == music_player.current_track {
                if music_player.is_playing() { "▶ ".to_string() } else { "■ ".to_string() }
            } else {
                format!("{:2} ", i + 1)
            };
            
            // Get file size
            let size = if let Ok(metadata) = std::fs::metadata(track) {
                let size_mb = metadata.len() as f64 / 1_048_576.0;
                format!(" ({:.1}MB)", size_mb)
            } else {
                String::new()
            };
            
            // Truncate filename if it's too long
            let max_width = area.width.saturating_sub(15 + size.len() as u16) as usize;
            let display_name = if filename.len() > max_width {
                format!("{}...", &filename[..max_width - 3])
            } else {
                filename.to_string()
            };

            ListItem::new(format!("{}{}{}", prefix, display_name, size))
                .style(Style::default().fg(if Some(i) == music_player.current_track {
                    Color::Cyan
                } else {
                    Color::White
                }))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title(" Playlist ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .highlight_style(Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

fn draw_progress<B: Backend>(f: &mut Frame<B>, music_player: &MusicPlayer, area: Rect) {
    let (progress_text, duration_text) = if let Some(progress) = music_player.get_progress() {
        let percentage = (progress * 100.0) as u8;
        let bar_width = area.width as usize - 20;
        let filled = (bar_width as f32 * progress) as usize;
        
        let progress_bar = format!(
            "{}{} {}%",
            "━".repeat(filled),
            "─".repeat(bar_width - filled),
            percentage
        );

        let time = music_player.get_elapsed_time();
        let total = music_player.get_total_time();
        let time_text = format!("{} / {}", time, total);

        (progress_bar, time_text)
    } else {
        ("Not playing".to_string(), "00:00 / 00:00".to_string())
    };

    let progress_block = Block::default()
        .title(" Progress ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let progress_widget = Paragraph::new(format!("{}\n{}", progress_text, duration_text))
        .block(progress_block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));

    f.render_widget(progress_widget, area);
}

fn draw_status<B: Backend>(f: &mut Frame<B>, music_player: &MusicPlayer, area: Rect) {
    let status = if let Some(current) = music_player.current_track {
        let track_name = music_player.tracks[current]
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        
        format!(
            "Playing: {} | Vol: {:.0}% | {}",
            track_name,
            music_player.volume * 100.0,
            if music_player.is_playing() { 
                "▶ Playing" 
            } else { 
                "⏸ Paused" 
            }
        )
    } else {
        "No track selected".to_string()
    };

    let status_widget = Paragraph::new(status)
        .block(Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left);

    f.render_widget(status_widget, area);
}

fn draw_controls<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let controls = "↑/↓: Select | Enter: Play | Space: Pause | ←/→: Prev/Next | +/-: Volume | q: Quit";
    
    let controls_widget = Paragraph::new(controls)
        .block(Block::default()
            .title(" Controls ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(controls_widget, area);
}