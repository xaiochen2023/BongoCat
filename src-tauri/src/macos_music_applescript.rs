#![cfg(target_os = "macos")] // Ensure this module only compiles on macOS

use std::process::Command;
// No longer need serde::Serialize here if LocalSongInfo is defined elsewhere and constructed here.
// However, if we construct LocalSongInfo here, we might still need it for derive if not pub fields.
// For now, assume LocalSongInfo fields are pub and it's constructed directly.
// use serde::Serialize; 

// Use the centralized LocalSongInfo from the local_player_manager module
use crate::local_player_manager::LocalSongInfo;

pub fn get_apple_music_song_info() -> Result<Option<LocalSongInfo>, String> {
    let script = r#"
        tell application "Music"
            if not (exists player state) then return ""
            if player state is playing then
                set current_track to current track
                set song_name to name of current_track
                set artist_name to artist of current_track
                set album_name to album of current_track
                if album_name is missing value then set album_name to ""
                return song_name & "
" & artist_name & "
" & album_name
            else
                return ""
            end if
        end tell
    "#;

    let output_result = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();

    match output_result {
        Ok(output) => {
            if output.status.success() {
                let result_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if result_str.is_empty() {
                    Ok(None) // Music.app is not playing or no track info available
                } else {
                    let parts: Vec<&str> = result_str.split('\n').collect();
                    if parts.len() >= 2 { // Title and Artist are essential
                        let title = parts[0].to_string();
                        let artist = parts[1].to_string();
                        let album = if parts.len() >= 3 && !parts[2].is_empty() {
                            Some(parts[2].to_string())
                        } else {
                            None
                        };
                        
                        Ok(Some(LocalSongInfo {
                            title,
                            artist,
                            album,
                            player_source: "AppleMusic".to_string(),
                        }))
                    } else {
                        Err(format!("Failed to parse AppleScript output: Unexpected format - '{}'", result_str))
                    }
                }
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                Err(format!("AppleScript execution failed: {}", error_msg))
            }
        }
        Err(e) => {
            Err(format!("Failed to execute osascript: {}", e))
        }
    }
}

// Example usage (for testing within this module, not part of the final lib usually)
// fn main() {
//     match get_apple_music_song_info() {
//         Ok(Some(info)) => println!("Currently Playing: {} by {} (Album: {:?}) from {}", info.title, info.artist, info.album, info.player_source),
//         Ok(None) => println!("Nothing is currently playing or Music.app is not providing info."),
//         Err(e) => eprintln!("Error: {}", e),
//     }
// }
