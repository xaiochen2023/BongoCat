use serde::Deserialize;
use reqwest;
use tauri::AppHandle;

use crate::spotify_auth::{get_valid_access_token, refresh_access_token}; // Assuming these are pub in spotify_auth

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
}

#[derive(Deserialize, Debug)]
struct SpotifyArtist {
    name: String,
}

#[derive(Deserialize, Debug)]
struct SpotifyTrackItem {
    name: String,
    artists: Vec<SpotifyArtist>,
    // Add other fields like album, duration_ms if ever needed for future expansion
}

#[derive(Deserialize, Debug)]
struct CurrentlyPlayingResponse {
    item: Option<SpotifyTrackItem>,
    is_playing: bool,
    // Add progress_ms if needed for future expansion
}

const SPOTIFY_CURRENTLY_PLAYING_URL: &str = "https://api.spotify.com/v1/me/player/currently-playing";

async fn fetch_with_token(app_handle: AppHandle, access_token: &str) -> Result<Option<SongInfo>, reqwest::Error> {
    let client = reqwest::Client::new();
    println!("Fetching current song with token: {}...", &access_token[..std::cmp::min(15, access_token.len())]); // Log safely

    let response = client
        .get(SPOTIFY_CURRENTLY_PLAYING_URL)
        .bearer_auth(access_token)
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::NO_CONTENT {
        println!("Spotify API returned 204 No Content (no song playing or inactive device).");
        return Ok(None);
    }
    
    if response.status().is_success() {
        let response_data = response.json::<CurrentlyPlayingResponse>().await?;
        if response_data.is_playing {
            if let Some(track_item) = response_data.item {
                let title = track_item.name;
                let artist = track_item
                    .artists
                    .get(0)
                    .map_or_else(|| "Unknown Artist".to_string(), |a| a.name.clone());
                println!("Currently playing: {} - {}", title, artist);
                return Ok(Some(SongInfo { title, artist }));
            } else {
                 println!("Spotify reports is_playing true, but no track item found.");
                return Ok(None); // Playing, but no item (e.g., an ad or podcast without item structure)
            }
        } else {
            println!("Spotify reports is_playing false.");
            return Ok(None); // Not playing
        }
    }
    
    // Store status for error handling outside this helper
    Err(reqwest::Error::from(response.error_for_status().unwrap_err()))
}

pub async fn fetch_current_song_info(app_handle: AppHandle) -> Result<Option<SongInfo>, String> {
    println!("fetch_current_song_info called.");
    let access_token = match get_valid_access_token(app_handle.clone()).await {
        Ok(token) => token,
        Err(e) => {
            // If get_valid_access_token determined auth is required, it might have emitted an event.
            // No need to emit another here, just return the error.
            eprintln!("Auth error in fetch_current_song_info: {}", e);
            return Err(format!("Authentication error: {}", e));
        }
    };

    match fetch_with_token(app_handle.clone(), &access_token).await {
        Ok(song_info_option) => Ok(song_info_option),
        Err(e) => {
            if let Some(status) = e.status() {
                if status == reqwest::StatusCode::UNAUTHORIZED {
                    println!("Spotify token unauthorized (401). Attempting refresh...");
                    match refresh_access_token().await { // refresh_access_token doesn't need app_handle if it uses global/static client or gets tokens directly
                        Ok(new_access_token) => {
                            println!("Token refreshed successfully. Retrying fetch...");
                            // Retry the GET request once with new_access_token
                            match fetch_with_token(app_handle.clone(), &new_access_token).await {
                                Ok(song_info_option_retry) => Ok(song_info_option_retry),
                                Err(retry_e) => {
                                    eprintln!("Failed to fetch song after token refresh: {:?}", retry_e);
                                    Err(format!("Failed to fetch song after token refresh: {:?}", retry_e))
                                }
                            }
                        }
                        Err(refresh_e) => {
                            eprintln!("Failed to refresh token: {}", refresh_e);
                             // If refresh fails, it might mean user needs to re-authenticate.
                            app_handle.emit_all("spotify_auth_required", ()).unwrap_or_default();
                            Err(format!("Failed to refresh token: {}", refresh_e))
                        }
                    }
                } else {
                    eprintln!("Spotify API error: {} - {:?}", status, e);
                    Err(format!("Spotify API error: {}", status))
                }
            } else {
                eprintln!("Network or other error: {:?}", e);
                Err(format!("Network or other error: {:?}", e))
            }
        }
    }
}
