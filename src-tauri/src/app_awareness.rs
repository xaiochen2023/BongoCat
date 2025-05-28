use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::Manager;
use once_cell::sync::Lazy; // Added for LAST_LOCAL_SONG_INFO

// Assuming local_player_manager and its LocalSongInfo are in the crate root or correctly pathed
use crate::local_player_manager::{self, LocalSongInfo}; // Added

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppDefinition {
    pub name: String,
    pub reaction_key: String,
    pub processes_win: Vec<String>,
    pub processes_mac: Vec<String>,
    pub processes_linux: Option<Vec<String>>,
}

// Updated DetectedApp Enum
#[derive(Clone, Debug, PartialEq)]
pub enum DetectedApp {
    Music,
    Game,
    Email, 
    Notes, 
    None,
}

impl DetectedApp {
    fn from_reaction_key(key: &str) -> Self {
        match key {
            "music" => DetectedApp::Music,
            "game" => DetectedApp::Game,
            "email" => DetectedApp::Email,
            "notes" => DetectedApp::Notes,
            _ => DetectedApp::None,
        }
    }

    fn as_str(&self) -> Option<String> {
        match self {
            DetectedApp::Music => Some("music".to_string()),
            DetectedApp::Game => Some("game".to_string()),
            DetectedApp::Email => Some("email".to_string()), 
            DetectedApp::Notes => Some("notes".to_string()), 
            DetectedApp::None => None,
        }
    }
}

pub type SharedDetectedAppState = Arc<Mutex<DetectedApp>>;

fn load_app_definitions<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>) -> Vec<AppDefinition> {
    let config_path = app_handle
        .path()
        .resolve("app_awareness_config.json", tauri::path::BaseDirectory::Resource) 
        .unwrap_or_else(|_| {
            PathBuf::from("app_awareness_config.json") 
        });

    println!("Attempting to load app definitions from: {:?}", config_path);

    match fs::read_to_string(config_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(definitions) => {
                println!("Successfully loaded app definitions from JSON.");
                return definitions;
            }
            Err(e) => {
                eprintln!("Failed to parse app_awareness_config.json: {}. Using hardcoded fallbacks.", e);
            }
        },
        Err(e) => {
            eprintln!("Failed to read app_awareness_config.json: {}. Using hardcoded fallbacks.", e);
        }
    }

    println!("Using hardcoded app definitions.");
    vec![
        AppDefinition {
            name: "Spotify Music (Fallback)".to_string(), reaction_key: "music".to_string(),
            processes_win: vec!["Spotify.exe".to_string(), "spotifywebhelper.exe".to_string()],
            processes_mac: vec!["Spotify".to_string()],
            processes_linux: Some(vec!["spotify".to_string()]),
        },
        AppDefinition {
            name: "Steam Games (Fallback)".to_string(), reaction_key: "game".to_string(),
            processes_win: vec!["steam.exe".to_string(), "Steam.exe".to_string()],
            processes_mac: vec!["Steam".to_string(), "steam_osx".to_string()],
            processes_linux: Some(vec!["steam".to_string()]),
        },
        AppDefinition {
            name: "Microsoft Outlook (Fallback)".to_string(), reaction_key: "email".to_string(),
            processes_win: vec!["OUTLOOK.EXE".to_string()],
            processes_mac: vec!["Microsoft Outlook".to_string()],
            processes_linux: None,
        },
        AppDefinition {
            name: "Obsidian Notes (Fallback)".to_string(), reaction_key: "notes".to_string(),
            processes_win: vec!["Obsidian.exe".to_string()],
            processes_mac: vec!["Obsidian".to_string()],
            processes_linux: None,
        },
    ]
}

// Static state for the last known local song info
static LAST_LOCAL_SONG_INFO: Lazy<Mutex<Option<LocalSongInfo>>> = Lazy::new(|| Mutex::new(None));

pub fn init_app_awareness<R: tauri::Runtime>(app_handle: tauri::AppHandle<R>, app_state: SharedDetectedAppState) {
    let shared_state_clone = app_state.clone();
    let app_handle_clone = app_handle.clone(); 

    thread::spawn(move || {
        let app_definitions = load_app_definitions(&app_handle_clone);
        let mut sys = System::new_all();
        let precedence: HashMap<String, u8> = [
            ("game".to_string(), 0),
            ("music".to_string(), 1),
            ("email".to_string(), 2),
            ("notes".to_string(), 3),
        ].iter().cloned().collect();

        loop {
            // --- App Detection Logic (existing) ---
            sys.refresh_processes();
            let mut best_detected_app_key: Option<String> = None;
            let mut current_min_precedence = u8::MAX;

            for app_def in &app_definitions {
                let target_processes = if cfg!(target_os = "windows") {
                    &app_def.processes_win
                } else if cfg!(target_os = "macos") {
                    &app_def.processes_mac
                } else {
                    app_def.processes_linux.as_ref().unwrap_or(&Vec::new())
                };

                for running_process in sys.processes().values() {
                    let running_process_name = running_process.name().to_lowercase();
                    for target_name in target_processes {
                        if running_process_name.contains(&target_name.to_lowercase()) {
                            let app_precedence = precedence.get(&app_def.reaction_key).cloned().unwrap_or(u8::MAX);
                            if app_precedence < current_min_precedence {
                                current_min_precedence = app_precedence;
                                best_detected_app_key = Some(app_def.reaction_key.clone());
                            }
                            break; 
                        }
                    }
                    if best_detected_app_key.as_ref().map_or(false, |k| k == &app_def.reaction_key) && 
                       precedence.get(&app_def.reaction_key).cloned().unwrap_or(u8::MAX) == current_min_precedence {
                        break; 
                    }
                }
            }
            
            let final_detected_app = best_detected_app_key
                .map_or(DetectedApp::None, |key| DetectedApp::from_reaction_key(&key));

            let mut current_app_state_guard = shared_state_clone.lock().unwrap();
            if *current_app_state_guard != final_detected_app {
                println!("App detection state changed to: {:?}", final_detected_app.as_str());
                *current_app_state_guard = final_detected_app.clone();
                if let Err(e) = app_handle_clone.emit_all("app_detection_change", final_detected_app.as_str()) {
                    eprintln!("Failed to emit app_detection_change event: {}", e);
                }
            }
            drop(current_app_state_guard);
            // --- End App Detection Logic ---

            // --- Poll Local Music Player (NEW) ---
            let current_song_result = local_player_manager::get_current_song_from_active_players();
            let mut last_song_guard = LAST_LOCAL_SONG_INFO.lock().unwrap();

            match current_song_result {
                Ok(Some(current_song_info)) => {
                    if *last_song_guard != Some(current_song_info.clone()) {
                        println!("Local song changed to: {} - {}", current_song_info.title, current_song_info.artist);
                        *last_song_guard = Some(current_song_info.clone());
                        if let Err(e) = app_handle_clone.emit_all("local_song_change", Some(current_song_info)) {
                            eprintln!("Failed to emit local_song_change event (song playing): {}", e);
                        }
                    }
                }
                Ok(None) => { // Nothing playing from local players
                    if last_song_guard.is_some() {
                        println!("Local song stopped playing.");
                        *last_song_guard = None;
                        if let Err(e) = app_handle_clone.emit_all("local_song_change", None::<Option<LocalSongInfo>>) {
                            eprintln!("Failed to emit local_song_change event (song stopped): {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching local song info: {}", e);
                    if last_song_guard.is_some() { 
                        println!("Local song error, clearing last known song.");
                        *last_song_guard = None;
                        if let Err(e_emit) = app_handle_clone.emit_all("local_song_change", None::<Option<LocalSongInfo>>) {
                             eprintln!("Failed to emit local_song_change event (error case): {}", e_emit);
                        }
                    }
                }
            }
            drop(last_song_guard);
            // --- End Poll Local Music Player ---

            thread::sleep(Duration::from_secs(5)); // Existing sleep duration
        }
    });
}

#[tauri::command]
pub fn get_detected_app_debug(state: tauri::State<SharedDetectedAppState>) -> Option<String> {
    let current_app = state.inner().lock().unwrap();
    current_app.as_str()
}
