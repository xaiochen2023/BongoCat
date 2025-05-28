use tauri::Manager;
use oauth2::{
    AuthorizationCode,
    AuthUrl,
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    PkceCodeVerifier,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl,
    StandardTokenResponse, // For token exchange
    EmptyExtraTokenFields, // For token exchange
    RefreshToken // For refresh token
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client; // For async requests
use std::sync::Mutex;
use once_cell::sync::Lazy;
// rand is already in Cargo.toml from previous step
use rand::RngCore; 
use rand::rngs::OsRng;
use tiny_http::{Server, Response, Header, Request};
use url::Url;
use std::thread;
use std::io::{Read, Write}; // For request.as_reader() and response.as_writer()
use serde::{Serialize, Deserialize}; // For token serialization
use keyring::Entry; // For secure storage

// --- Constants ---
const CLIENT_ID: &str = "YOUR_SPOTIFY_CLIENT_ID"; 
const CLIENT_SECRET: &str = "YOUR_SPOTIFY_CLIENT_SECRET"; 
const REDIRECT_URI_STR: &str = "http://localhost:14732/spotify_callback";
const AUTH_URL_STR: &str = "https://accounts.spotify.com/authorize";
const TOKEN_URL_STR: &str = "https://accounts.spotify.com/api/token";

const KEYRING_SERVICE_NAME: &str = "BongoCatSpotify";
const KEYRING_USERNAME_TOKENS: &str = "user_spotify_tokens";
// CSRF_STATE static is used for storing the CSRF token during the auth flow
static CSRF_STATE: Lazy<Mutex<Option<CsrfToken>>> = Lazy::new(|| Mutex::new(None));
// PKCE_VERIFIER static is used for storing the PKCE verifier during the auth flow
static PKCE_VERIFIER: Lazy<Mutex<Option<PkceCodeVerifier>>> = Lazy::new(|| Mutex::new(None));

// --- Token Struct ---
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SpotifyTokens {
    access_token: String,
    refresh_token: Option<String>,
    // expires_at: Option<u64>, // Placeholder for future expiry handling
}

// --- Helper Functions ---
fn get_oauth_client() -> Result<BasicClient, String> {
    let auth_url = AuthUrl::new(AUTH_URL_STR.to_string())
        .map_err(|e| format!("Invalid authorization URL: {}", e))?;
    let token_url = TokenUrl::new(TOKEN_URL_STR.to_string())
        .map_err(|e| format!("Invalid token URL: {}", e))?;

    Ok(BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        Some(ClientSecret::new(CLIENT_SECRET.to_string())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new(REDIRECT_URI_STR.to_string())
            .map_err(|e| format!("Invalid redirect URI: {}", e))?,
    ))
}

// --- Token Storage ---
fn store_tokens(tokens: &SpotifyTokens) -> Result<(), String> {
    let entry = Entry::new(KEYRING_SERVICE_NAME, KEYRING_USERNAME_TOKENS)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    let tokens_json = serde_json::to_string(tokens)
        .map_err(|e| format!("Failed to serialize tokens: {}", e))?;
    entry.set_password(&tokens_json)
        .map_err(|e| format!("Failed to store tokens in keyring: {}", e))?;
    println!("Tokens stored successfully.");
    Ok(())
}

fn get_tokens() -> Result<Option<SpotifyTokens>, String> {
    let entry = Entry::new(KEYRING_SERVICE_NAME, KEYRING_USERNAME_TOKENS)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    match entry.get_password() {
        Ok(tokens_json) => {
            let tokens: SpotifyTokens = serde_json::from_str(&tokens_json)
                .map_err(|e| format!("Failed to deserialize tokens: {}", e))?;
            println!("Tokens retrieved successfully.");
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => {
            println!("No tokens found in keyring.");
            Ok(None)
        }
        Err(e) => Err(format!("Failed to retrieve tokens from keyring: {}", e)),
    }
}

// --- Authentication Flow ---
#[tauri::command]
pub async fn start_auth_flow(app_handle: tauri::AppHandle) -> Result<(), String> {
    if CLIENT_ID == "YOUR_SPOTIFY_CLIENT_ID" || CLIENT_SECRET == "YOUR_SPOTIFY_CLIENT_SECRET" {
        let msg = "Spotify Client ID or Secret not configured. Please set them in spotify_auth.rs".to_string();
        // It's good practice to emit error to frontend if it's user-actionable or for debugging
        app_handle.emit_all("spotify_auth_error", Some(msg.clone())).unwrap_or_default();
        return Err(msg);
    }

    let client = get_oauth_client()?;
    let (pkce_challenge, pkce_verifier_generated) = PkceCodeChallenge::new_random_sha256();
    let csrf_token_generated = CsrfToken::new_random();

    // Store PKCE verifier and CSRF token
    *PKCE_VERIFIER.lock().unwrap() = Some(pkce_verifier_generated);
    *CSRF_STATE.lock().unwrap() = Some(csrf_token_generated.clone());

    let (authorize_url, _returned_csrf_state) = client
        .authorize_url(csrf_token_generated)
        .add_scope(Scope::new("user-read-playback-state".to_string()))
        .add_scope(Scope::new("user-modify-playback-state".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();
    
    let server_addr = "0.0.0.0:14732";
    let server = Server::http(server_addr).map_err(|e| format!("Failed to start HTTP server: {}", e))?;
    
    let app_handle_clone_for_server = app_handle.clone();
    thread::spawn(move || {
        println!("Spotify callback server listening on {}...", server_addr);
        if let Ok(Some(mut request)) = server.try_recv() { // Use try_recv in a loop or handle one request
            let request_url_str = format!("http://localhost{}", request.url()); // Construct full URL for parsing
            
            let respond_with_html = |req: Request, html_body: &str, status_code: i32| {
                let response = Response::from_string(html_body)
                    .with_status_code(status_code)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap());
                if let Err(e) = req.respond(response) {
                    eprintln!("Failed to send response from callback server: {}", e);
                }
            };

            match Url::parse(&request_url_str) {
                Ok(parsed_url) => {
                    let params: std::collections::HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
                    let received_code = params.get("code").cloned();
                    let received_state = params.get("state").cloned();

                    let stored_csrf_state = CSRF_STATE.lock().unwrap().take();

                    if stored_csrf_state.is_none() || received_state.is_none() || stored_csrf_state.unwrap().secret() != &received_state.unwrap() {
                        eprintln!("CSRF token mismatch or missing.");
                        app_handle_clone_for_server.emit_all("spotify_auth_error", Some("CSRF token mismatch.".to_string())).unwrap_or_default();
                        respond_with_html(request, "<!DOCTYPE html><html><body><h1>Error: CSRF token mismatch.</h1><p>Please try authenticating again.</p></body></html>", 400);
                        return; // Stop processing
                    }

                    if let Some(code) = received_code {
                        let pkce_verifier_secret = match PKCE_VERIFIER.lock().unwrap().take() {
                            Some(verifier) => verifier.secret().to_string(),
                            None => {
                                eprintln!("PKCE verifier not found.");
                                app_handle_clone_for_server.emit_all("spotify_auth_error", Some("PKCE verifier missing.".to_string())).unwrap_or_default();
                                respond_with_html(request, "<!DOCTYPE html><html><body><h1>Error: PKCE verifier missing.</h1></body></html>", 400);
                                return;
                            }
                        };
                        
                        let app_handle_for_token_exchange = app_handle_clone_for_server.clone();
                        // As exchange_code_for_tokens is async, we need to run it in a way that doesn't block this thread
                        // For tiny_http, this thread is short-lived anyway for one request.
                        // If using a persistent server, you'd spawn a new async task.
                        // Here, we can block this server thread briefly or make it async.
                        // For simplicity now, let's use a blocking call within a new async runtime just for this exchange.
                        // A better approach for a real app might be to send this to a Tauri async command or use Tokio.
                        let code_clone = code.clone();
                        
                        // Using tauri::async_runtime::spawn to run the async token exchange
                        tauri::async_runtime::spawn(async move {
                            match exchange_code_for_tokens(app_handle_for_token_exchange.clone(), code_clone, pkce_verifier_secret).await {
                                Ok(_) => {
                                    println!("Token exchange successful from server thread.");
                                    // Respond success in the main server thread if possible, or handle it here.
                                    // For now, the main response is simplified.
                                }
                                Err(e) => {
                                    eprintln!("Token exchange failed: {}", e);
                                    // Error already emitted by exchange_code_for_tokens
                                }
                            }
                        });
                        respond_with_html(request, "<!DOCTYPE html><html><head><title>BongoCat Spotify Auth</title></head><body><h1>Authentication successful!</h1><p>You can close this window and return to BongoCat.</p><script>window.close();</script></body></html>", 200);

                    } else {
                        eprintln!("Authorization code not found in callback URL.");
                        app_handle_clone_for_server.emit_all("spotify_auth_error", Some("Authorization code missing.".to_string())).unwrap_or_default();
                        respond_with_html(request, "<!DOCTYPE html><html><body><h1>Error: Authorization code missing.</h1></body></html>", 400);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse callback URL: {}", e);
                    app_handle_clone_for_server.emit_all("spotify_auth_error", Some(format!("Invalid callback URL: {}", e))).unwrap_or_default();
                    respond_with_html(request, "<!DOCTYPE html><html><body><h1>Error: Invalid callback URL.</h1></body></html>", 400);
                }
            }
        } else if let Err(e) = server.try_recv() {
             eprintln!("Error receiving request on callback server: {}", e);
             app_handle_clone_for_server.emit_all("spotify_auth_error", Some(format!("Callback server error: {}", e))).unwrap_or_default();
        }
        // Server stops after one attempt to receive or one successful receive.
    });

    tauri::api::shell::open(&app_handle.shell().scope(), authorize_url.as_str(), None)
        .map_err(|e| format!("Failed to open auth URL: {}", e))?;
    Ok(())
}


async fn exchange_code_for_tokens(
    app_handle: tauri::AppHandle, 
    code: String, 
    pkce_verifier_secret: String
) -> Result<(), String> {
    let client = get_oauth_client()?;
    
    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier_secret))
        .request_async(async_http_client)
        .await
        .map_err(|e| format!("Token exchange request failed: {:?}", e))?;

    let access_token = token_result.access_token().secret().to_string();
    let refresh_token_opt = token_result.refresh_token().map(|rt| rt.secret().to_string());
    
    // expires_in is also available in token_result.expires_in() -> Option<Duration>
    // let expires_at = ... // Calculate and store this for robust expiry checks

    let tokens_to_store = SpotifyTokens {
        access_token,
        refresh_token: refresh_token_opt,
        // expires_at: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + token_result.expires_in().map_or(3600, |d| d.as_secs()))
    };

    store_tokens(&tokens_to_store)?;
    app_handle.emit_all("spotify_auth_success", ()).unwrap_or_default();
    println!("Successfully exchanged code for tokens and stored them.");
    Ok(())
}


pub async fn refresh_access_token() -> Result<String, String> {
    println!("Attempting to refresh access token...");
    let stored_tokens = match get_tokens()? {
        Some(tokens) => tokens,
        None => return Err("No stored tokens found to refresh.".to_string()),
    };

    let refresh_token_str = match stored_tokens.refresh_token {
        Some(rt) => rt,
        None => return Err("No refresh token available.".to_string()),
    };
    
    let client = get_oauth_client()?;
    let token_result = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token_str))
        .request_async(async_http_client)
        .await
        .map_err(|e| format!("Token refresh request failed: {:?}", e))?;

    let new_access_token = token_result.access_token().secret().to_string();
    let new_refresh_token_opt = token_result.refresh_token().map(|rt| rt.secret().to_string());

    let updated_tokens = SpotifyTokens {
        access_token: new_access_token.clone(),
        refresh_token: new_refresh_token_opt.or(stored_tokens.refresh_token), // Keep old refresh token if new one isn't provided
        // expires_at: ... // Update expiry
    };
    
    store_tokens(&updated_tokens)?;
    println!("Access token refreshed and stored successfully.");
    Ok(new_access_token)
}

pub async fn get_valid_access_token(app_handle: tauri::AppHandle) -> Result<String, String> {
    println!("Attempting to get valid access token...");
    match get_tokens()? {
        Some(tokens) => {
            // Basic check: if access token exists, return it.
            // For robust expiry, you'd check tokens.expires_at against current time.
            // If expired, call refresh_access_token.
            // For this batch, we'll rely on API calls failing with 401 to trigger refresh elsewhere if needed.
            // A more proactive refresh would involve storing and checking 'expires_at'.
            // For now, just return the stored one.
            println!("Found stored access token.");
            Ok(tokens.access_token)
        }
        None => {
            println!("No tokens found. User needs to authenticate.");
            // Optionally, emit an event to prompt frontend for auth, though usually frontend initiates.
            app_handle.emit_all("spotify_auth_required", ()).unwrap_or_default();
            Err("Not authenticated. No tokens found.".to_string())
        }
    }
}

// Removed old placeholder: handle_spotify_callback_logic
// Removed old placeholder: refresh_access_token_impl (renamed to refresh_access_token)
// store_tokens and get_tokens were placeholders and are now implemented.
// get_valid_access_token was a placeholder and is now implemented with basic logic.
