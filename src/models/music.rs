use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AudioFileInfo {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct MusicPlayerState {
    pub current_playing_index: Option<usize>,
    pub playback_state: PlaybackState,
    pub current_position: Option<Duration>,
    pub volume: f32,
}

impl Default for MusicPlayerState {
    fn default() -> Self {
        Self {
            current_playing_index: None,
            playback_state: PlaybackState::Stopped,
            current_position: None,
            volume: 0.8,
        }
    }
}
