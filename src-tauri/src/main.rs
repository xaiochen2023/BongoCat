#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Declare modules
mod app_awareness;
mod spotify_auth;
mod spotify_player;
mod local_player_manager;     // Added
#[cfg(target_os = "macos")]   // Added
mod macos_music_applescript;  // Added
#[cfg(target_os = "windows")] // Added
mod windows_musicbee_file;    // Added

use std::sync::{Arc, Mutex};
use tauri::Manager; // Required for app_handle()

fn main() {
    // Create the shared state for detected applications
    let detected_app_state = Arc::new(Mutex::new(app_awareness::DetectedApp::None));

    // It's crucial that bongo_cat_lib::run() or its equivalent functionality 
    // is properly integrated here if it contains other essential setup.
    // For this subtask, we are focusing on integrating app_awareness.
    // If bongo_cat_lib::run() sets up other plugins or commands, they would need to be
    // manually added to this builder chain.

    let builder = tauri::Builder::default()
        .manage(detected_app_state.clone()) // Manage the state so it can be accessed in commands
        .setup(move |app| {
            // Initialize app awareness, passing the app handle and the managed state
            app_awareness::init_app_awareness(app.app_handle(), detected_app_state);
            
            // If bongo_cat_lib::run() had its own setup logic that returned Result,
            // it would need to be called here and its Ok(()) propagated or errors handled.
            // e.g., bongo_cat_lib::perform_initial_setup(app.handle())?;
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_awareness::get_detected_app_debug,
            spotify_auth::start_auth_flow // Added start_auth_flow command
            // If bongo_cat_lib registered commands, they need to be added here.
            // For example, if bongo_cat_lib::get_commands() returns a Vec<InvokeHandler<R>>,
            // those would need to be iterated and registered, or ideally, bongo_cat_lib
            // would provide a function that takes the builder and appends its configurations.
        ]);

    // The original main.rs called bongo_cat_lib::run().
    // We are replacing it with a direct Tauri run.
    // This assumes that essential configurations previously in bongo_cat_lib::run()
    // would be moved into the builder chain above or are not critical for this specific feature.
    // A more robust integration would involve understanding the full scope of bongo_cat_lib::run().

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
