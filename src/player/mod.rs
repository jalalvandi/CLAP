use rodio::{Decoder, OutputStream, Sink};
use std::time::{Duration, Instant};
use std::{error::Error, fs::File, io::BufReader, path::PathBuf};
use symphonia::core::probe::Hint;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

pub struct MusicPlayer {
    pub tracks: Vec<PathBuf>,
    pub current_track: Option<usize>,
    sink: Option<Sink>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    _stream: Option<OutputStream>,
    pub volume: f32,
    start_time: Option<Instant>,
    duration: Option<Duration>,
    paused_duration: Option<Duration>,
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
            duration: None,
            paused_duration: None,
        }
    }

    pub fn add_track(&mut self, path: PathBuf) {
        self.tracks.push(path);
    }

    fn get_track_duration(path: &PathBuf) -> Option<Duration> {
        let file = File::open(path).ok()?;
        let stream = MediaSourceStream::new(Box::new(file), Default::default());
        let hint = Hint::new();
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        
        let probed = symphonia::default::get_probe()
            .format(&hint, stream, &format_opts, &metadata_opts)
            .ok()?;
        
        let format = probed.format;
        let track = format.tracks().get(0)?;
        let time_base = track.codec_params.time_base?;
        let n_frames = track.codec_params.n_frames?;
        
        Some(Duration::from_secs_f64(n_frames as f64 * time_base.numer as f64 / time_base.denom as f64))
    }

    pub fn play_track(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if index >= self.tracks.len() {
            return Ok(());
        }

        self.stop();

        // Get track duration first
        self.duration = Self::get_track_duration(&self.tracks[index]);

        if self._stream.is_none() {
            let (stream, handle) = OutputStream::try_default()?;
            self._stream = Some(stream);
            self.stream_handle = Some(handle);
        }

        if let Some(handle) = &self.stream_handle {
            let file = File::open(&self.tracks[index])?;
            let reader = BufReader::new(file);
            let source = Decoder::new(reader)?;
            
            let sink = Sink::try_new(handle)?;
            sink.set_volume(self.volume);
            sink.append(source);
            sink.play();
            
            self.current_track = Some(index);
            self.sink = Some(sink);
            self.start_time = Some(Instant::now());
            self.paused_duration = None;
        }
        Ok(())
    }

    pub fn get_progress(&self) -> Option<f32> {
        if let (Some(start), Some(duration)) = (self.start_time, self.duration) {
            if self.is_playing() {
                let elapsed = if let Some(paused) = self.paused_duration {
                    paused
                } else {
                    start.elapsed()
                };
                Some((elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0))
            } else if let Some(paused) = self.paused_duration {
                Some((paused.as_secs_f32() / duration.as_secs_f32()).min(1.0))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_elapsed_time(&self) -> String {
        if let Some(start) = self.start_time {
            let elapsed = if let Some(paused) = self.paused_duration {
                paused
            } else {
                start.elapsed()
            };
            let seconds = elapsed.as_secs();
            let minutes = seconds / 60;
            let remaining_seconds = seconds % 60;
            format!("{:02}:{:02}", minutes, remaining_seconds)
        } else {
            "00:00".to_string()
        }
    }

    pub fn next_track(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(current) = self.current_track {
            let next = (current + 1) % self.tracks.len();
            self.play_track(next)?;
        } else if !self.tracks.is_empty() {
            self.play_track(0)?;
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
        } else if !self.tracks.is_empty() {
            self.play_track(self.tracks.len() - 1)?;
        }
        Ok(())
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

    pub fn play(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            if let Some(paused) = self.paused_duration {
                self.start_time = Some(Instant::now() - paused);
                self.paused_duration = None;
            } else if self.start_time.is_none() {
                self.start_time = Some(Instant::now());
            }
        }
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            if let Some(start) = self.start_time {
                self.paused_duration = Some(start.elapsed());
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
        self.paused_duration = None;
    }

    pub fn is_playing(&self) -> bool {
        if let Some(sink) = &self.sink {
            !sink.is_paused() && !self.is_track_finished()
        } else {
            false
        }
    }

    pub fn get_total_time(&self) -> String {
        if let Some(duration) = self.duration {
            let total_secs = duration.as_secs();
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            format!("{:02}:{:02}", minutes, seconds)
        } else {
            "00:00".to_string()
        }
    }

    // Add a method to get both elapsed and total time in one call
    pub fn get_time_info(&self) -> (String, String) {
        let elapsed = self.get_elapsed_time();
        let total = self.get_total_time();
        (elapsed, total)
    }

    pub fn check_auto_advance(&mut self) -> Result<(), Box<dyn Error>> {
        if let (Some(sink), Some(start), Some(duration)) = (&self.sink, self.start_time, self.duration) {
            if !sink.is_paused() && start.elapsed() >= duration {
                return self.next_track();
            }
        }
        Ok(())
    }

    pub fn is_track_finished(&self) -> bool {
        if let (Some(start), Some(duration)) = (self.start_time, self.duration) {
            start.elapsed() >= duration
        } else {
            false
        }
    }
}