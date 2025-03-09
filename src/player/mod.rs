use std::time::{Duration, Instant};
use std::ops::Sub;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{error::Error, fs::File, io::BufReader, path::PathBuf};

pub struct MusicPlayer {
    pub tracks: Vec<PathBuf>,
    pub current_track: Option<usize>,
    sink: Option<Sink>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    _stream: Option<OutputStream>,
    pub volume: f32,
    start_time: Option<Instant>,
    elapsed: Duration,
    paused_time: Option<Instant>,
    duration: Option<Duration>,
}

impl MusicPlayer {
    pub fn new() -> Self {
        MusicPlayer {
            tracks: Vec::new(),
            current_track: None,
            sink: None,
            stream_handle: None,
            _stream: None,
            volume: 1.0,
            start_time: None,
            elapsed: Duration::from_secs(0),
            paused_time: None,
            duration: None,
        }
    }

    pub fn add_track(&mut self, path: PathBuf) {
        self.tracks.push(path);
    }

    pub fn play_track(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if index >= self.tracks.len() {
            return Ok(());
        }

        self.stop();

        if self._stream.is_none() {
            let (stream, handle) = OutputStream::try_default()?;
            self._stream = Some(stream);
            self.stream_handle = Some(handle);
        }

        if let Some(handle) = &self.stream_handle {
            let file = File::open(&self.tracks[index])?;
            let reader = BufReader::new(file);
            let source = Decoder::new(reader)?;
            
            // Get duration using the Source trait
            self.duration = source.total_duration();
            
            let sink = Sink::try_new(handle)?;
            sink.set_volume(self.volume);
            sink.append(source);
            sink.play();
            
            self.current_track = Some(index);
            self.sink = Some(sink);
            self.start_time = Some(Instant::now());
            self.elapsed = Duration::from_secs(0);
            self.paused_time = None;
        }

        Ok(())
    }

    pub fn play(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            if let Some(paused_at) = self.paused_time {
                self.elapsed += paused_at.elapsed();
                // Using checked_sub for safe subtraction
                if let Some(new_start) = Instant::now().checked_sub(self.elapsed) {
                    self.start_time = Some(new_start);
                }
            }
            self.paused_time = None;
        }
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            self.paused_time = Some(Instant::now());
            if let Some(start) = self.start_time {
                self.elapsed = start.elapsed();
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.sink = None;
        self.start_time = None;
        self.duration = None;
        self.elapsed = Duration::from_secs(0);
        self.paused_time = None;
    }

    pub fn is_playing(&self) -> bool {
        if let Some(sink) = &self.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }

    pub fn increase_volume(&mut self) {
        self.volume = (self.volume + 0.1).min(1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn decrease_volume(&mut self) {
        self.volume = (self.volume - 0.1).max(0.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn next_track(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current) = self.current_track {
            let next = (current + 1) % self.tracks.len();
            self.play_track(next)?;
        }
        Ok(())
    }

    pub fn previous_track(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current) = self.current_track {
            let previous = if current == 0 {
                self.tracks.len() - 1
            } else {
                current - 1
            };
            self.play_track(previous)?;
        }
        Ok(())
    }

    pub fn get_progress(&self) -> Option<f32> {
        if let Some(sink) = &self.sink {
            let played = sink.len() as f32;
            let total = sink.len() as f32;
            if total > 0.0 {
                Some((1.0 - (played / total)).min(1.0))
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }

    pub fn get_duration(&self) -> Option<Duration> {
        if let Some(sink) = &self.sink {
            Some(Duration::from_secs_f32(sink.len() as f32))
        } else {
            None
        }
    }

    pub fn get_elapsed_time(&self) -> String {
        if let Some(sink) = &self.sink {
            let seconds = (sink.len() as f32 * (1.0 - sink.speed())) as u64;
            let minutes = seconds / 60;
            let remaining_seconds = seconds % 60;
            format!("{:02}:{:02}", minutes, remaining_seconds)
        } else {
            "00:00".to_string()
        }
    }
}