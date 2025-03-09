# CLAP

CLAP (Command Line Audio Player) is a simple music player that runs right in your terminal.This project provides a simple user interface for playing audio files, with controls for playback and track management.

## Project Structure

```
CLAP
├── src
│   ├── main.rs          # Entry point of the application
│   ├── player           # Module for audio playback functionality
│   │   ├── mod.rs       # Player module exports
│   │   └── controls.rs  # Playback control functions
│   ├── ui               # Module for user interface
│   │   ├── mod.rs       # UI module exports
│   │   └── tui.rs       # Terminal user interface functions
│   └── utils            # Module for utility functions
│       ├── mod.rs       # Utils module exports
│       └── audio.rs     # Audio file handling functions
├── Cargo.toml           # Cargo configuration file
├── Cargo.lock           # Dependency lock file
└── README.md            # Project documentation
```

## Setup Instructions

1. Clone the repository:
   ```bash
   git clone https://github.com/jalalvandi/CLAP
   cd music-cli
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the application:
   ```bash
   cargo run
   ```

## Usage

- Use the terminal interface to navigate through your OS music library 
- Control playback with the provided commands (play, pause, stop).
- Enjoy your music!