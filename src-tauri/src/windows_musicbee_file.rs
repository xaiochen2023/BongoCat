#![cfg(target_os = "windows")] // Ensure this module only compiles on Windows

use std::fs;
use std::path::PathBuf;
use crate::local_player_manager::LocalSongInfo; // Assumes local_player_manager is at crate root

// Helper function to get the path to the NowPlaying.txt file
fn get_now_playing_file_path() -> Option<PathBuf> {
    // Try to get the APPDATA environment variable
    match std::env::var("APPDATA") {
        Ok(appdata_path) => {
            let mut path = PathBuf::from(appdata_path);
            path.push("MusicBee");
            path.push("NowPlaying.txt");
            Some(path)
        }
        Err(_) => {
            // APPDATA not found, could log this. For now, return None.
            eprintln!("APPDATA environment variable not found.");
            None
        }
    }
}

pub fn get_musicbee_song_info() -> Result<Option<LocalSongInfo>, String> {
    let path = match get_now_playing_file_path() {
        Some(p) => p,
        None => return Ok(None), // APPDATA path couldn't be determined
    };

    if !path.exists() {
        // File doesn't exist, so MusicBee is likely not running or plugin not configured
        return Ok(None);
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            // Error reading file (e.g., permissions, locked)
            return Err(format!("Failed to read MusicBee NowPlaying.txt: {}", e));
        }
    };

    if content.trim().is_empty() {
        // File is empty or only whitespace
        return Ok(None);
    }

    let lines: Vec<&str> = content.lines().collect();

    // Line-by-Line Format:
    // Line 0: Title
    // Line 1: Artist
    // Line 2: Album (optional)
    // Other lines (duration, position, path) are ignored for now.

    let title = lines.get(0).map_or("", |s| s.trim()).to_string();
    let artist = lines.get(1).map_or("", |s| s.trim()).to_string();
    
    // If title or artist is empty, consider it invalid or not playing
    if title.is_empty() || artist.is_empty() {
        return Ok(None);
    }

    let album = lines.get(2).map_or(None, |s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    Ok(Some(LocalSongInfo {
        title,
        artist,
        album,
        player_source: "MusicBee".to_string(),
    }))
}

// Example usage (for testing within this module)
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_read_musicbee_info() {
//         // To test this, you'd need to mock the file system or create a dummy NowPlaying.txt
//         // For example, if you create a dummy file:
//         // let dummy_content = "My Song Title\nMy Artist\nMy Album\n300\n120\nC:\\path\\to\\song.mp3";
//         // let appdata_path = std::env::var("APPDATA").unwrap();
//         // let mut path = PathBuf::from(appdata_path);
//         // path.push("MusicBee");
//         // fs::create_dir_all(&path).unwrap();
//         // path.push("NowPlaying.txt");
//         // fs::write(&path, dummy_content).unwrap();
        
//         match get_musicbee_song_info() {
//             Ok(Some(info)) => {
//                 println!("Currently Playing: {} by {} (Album: {:?}) from {}", info.title, info.artist, info.album, info.player_source);
//                 // assert_eq!(info.title, "My Song Title");
//             }
//             Ok(None) => println!("Nothing detected from MusicBee."),
//             Err(e) => eprintln!("Error: {}", e),
//         }
//         // fs::remove_file(path).unwrap(); // Clean up dummy file
//     }
// }
