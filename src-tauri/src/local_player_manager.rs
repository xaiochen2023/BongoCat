use serde::Serialize;

// Conditional use statements for OS-specific modules
#[cfg(target_os = "macos")]
use crate::macos_music_applescript;

#[cfg(target_os = "windows")]
use crate::windows_musicbee_file;

// Centralized LocalSongInfo struct definition
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalSongInfo {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub player_source: String,
}

// Function to get current song from active local players based on OS
pub fn get_current_song_from_active_players() -> Result<Option<LocalSongInfo>, String> {
    #[cfg(target_os = "macos")]
    {
        match macos_music_applescript::get_apple_music_song_info() {
            Ok(Some(song_info)) => Ok(Some(song_info)),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Error from Apple Music: {}", e)),
        }
    }

    #[cfg(target_os = "windows")]
    {
        match windows_musicbee_file::get_musicbee_song_info() {
            Ok(Some(song_info)) => Ok(Some(song_info)),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Error from MusicBee: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Fallback for other operating systems
        // println!("Local player support not implemented for this OS yet.");
        Ok(None)
    }
}
