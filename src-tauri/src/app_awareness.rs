use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::Manager; // Required for app_handle().emit_all

// Define the state that will be shared between threads
#[derive(Clone, Debug, PartialEq)]
pub enum DetectedApp {
    Music,
    Game,
    None,
}

impl DetectedApp {
    fn as_str(&self) -> Option<String> {
        match self {
            DetectedApp::Music => Some("music".to_string()),
            DetectedApp::Game => Some("game".to_string()),
            DetectedApp::None => None,
        }
    }
}

pub type SharedDetectedAppState = Arc<Mutex<DetectedApp>>;

// Changed signature to accept app_state
pub fn init_app_awareness<R: tauri::Runtime>(app_handle: tauri::AppHandle<R>, app_state: SharedDetectedAppState) {
    // Removed local creation of shared_state, app_state is now passed in.
    // The clone is still needed for the thread.
    let shared_state_clone = app_state.clone();

    thread::spawn(move || {
        let mut sys = System::new_all();
        loop {
            sys.refresh_processes();
            let mut music_found = false;
            let mut game_found = false;

            for (_pid, process) in sys.processes() {
                let process_name = process.name().to_lowercase();

                // Simple checks, can be made more robust
                // For macOS, check for bundle names if possible or typical process names
                // For Windows, check for .exe names
                // For Linux, check for process names
                
                // Spotify checks
                if cfg!(target_os = "windows") {
                    if process_name.contains("spotify.exe") {
                        music_found = true;
                    }
                } else if cfg!(target_os = "macos") {
                    if process_name.contains("spotify") { // Process name on macOS is often just "Spotify"
                        music_found = true;
                    }
                } else { // Linux and other Unix-like
                    if process_name.contains("spotify") {
                         music_found = true;
                    }
                }

                // Steam checks
                if cfg!(target_os = "windows") {
                    if process_name.contains("steam.exe") {
                        game_found = true;
                    }
                } else if cfg!(target_os = "macos") {
                     // On macOS, Steam has multiple helper processes. "steam_osx" or "steam" might be main.
                    if process_name.contains("steam") { // Main process can be "steam" or "steam_osx"
                        game_found = true;
                    }
                } else { // Linux
                    if process_name.contains("steam") {
                        game_found = true;
                    }
                }
            }

            let current_app_type = if game_found { // Game takes precedence
                DetectedApp::Game
            } else if music_found {
                DetectedApp::Music
            } else {
                DetectedApp::None
            };
            
            // Use the passed-in app_state (via shared_state_clone)
            let mut current_state_guard = shared_state_clone.lock().unwrap();
            if *current_state_guard != current_app_type {
                println!("App detection state changed to: {:?}", current_app_type.as_str()); // For debug
                *current_state_guard = current_app_type.clone();
                // Emit event to frontend using app_handle
                if let Err(e) = app_handle.emit_all("app_detection_change", current_app_type.as_str()) {
                    eprintln!("Failed to emit app_detection_change event: {}", e);
                }
            }
            drop(current_state_guard); // Release lock before sleep

            thread::sleep(Duration::from_secs(5)); // Check every 5 seconds
        }
    });
}

// Tauri command to get current state for debugging
#[tauri::command]
pub fn get_detected_app_debug(state: tauri::State<SharedDetectedAppState>) -> Option<String> {
    // Access the managed state directly
    let current_app = state.inner().lock().unwrap();
    // Use as_str() method from DetectedApp enum
    current_app.as_str()
}
