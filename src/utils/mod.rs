// src/utils/mod.rs

pub mod audio;

// Export any additional utility functions here as needed

use std::path::{Path, PathBuf};
use std::fs;

pub fn scan_music_directory(dir: &Path) -> Vec<PathBuf> {
    let mut music_files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                        "mp3" | "wav" | "flac" | "ogg" => music_files.push(path),
                        _ => continue,
                    }
                }
            }
        }
    }
    music_files
}