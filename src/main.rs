mod background;
mod coordinate_system;
mod game;
mod game_data;
mod game_over_screen;
mod game_ui;
mod grid;
mod music_manager;
mod retris_colors;
mod retris_ui;
mod sound_manager;
mod storage;
mod tetris_mobile_controller;
mod tetris_shape;
mod title_screen;
mod volume_control_screen;
mod volume_manager;

use background::Background;
use egor::app::*;
use egor::input::{KeyCode, MouseButton};
use game::Game;
use game_over_screen::{GameOverAction, GameOverScreen};
use music_manager::MusicManager;
use retris_ui::MuteButton;
use sound_manager::SoundManager;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicBool, Ordering};
use title_screen::TitleScreen;
use volume_control_screen::VolumeControlScreen;
use volume_manager::VolumeManager;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
// Screen size is now dynamic and obtained from gfx.screen_size() at runtime

// Boolean flag that JavaScript can set to request music start
#[cfg(target_arch = "wasm32")]
static START_MUSIC_FLAG: AtomicBool = AtomicBool::new(false);

/// Device pixel ratio (set by JavaScript, defaults to 1.0)
#[cfg(target_arch = "wasm32")]
static DEVICE_PIXEL_RATIO: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3F800000); // 1.0 as f32 bits

/// JavaScript-callable function to start the music
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_music() {
    START_MUSIC_FLAG.store(true, Ordering::Relaxed);
}

/// JavaScript-callable function to set device pixel ratio
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_device_pixel_ratio(ratio: f32) {
    let bits = ratio.to_bits();
    DEVICE_PIXEL_RATIO.store(bits, Ordering::Relaxed);
}

/// Get the device pixel ratio (for use in Rust code)
#[cfg(target_arch = "wasm32")]
pub fn get_device_pixel_ratio() -> f32 {
    let bits = DEVICE_PIXEL_RATIO.load(Ordering::Relaxed);
    f32::from_bits(bits)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_device_pixel_ratio() -> f32 {
    1.0 // Default for non-WASM builds
}


/// Helper function to create audio managers
/// This should only be called after user interaction in WASM
fn create_audio_managers(
    volume_manager: &VolumeManager,
) -> (
    Result<SoundManager, Box<dyn std::error::Error>>,
    Result<MusicManager, Box<dyn std::error::Error>>,
) {
    (
        SoundManager::new(volume_manager.clone()),
        MusicManager::new(volume_manager.clone()),
    )
}

// Extension traits to hide Option checks and make game code cleaner
trait SoundManagerOption {
    fn play_bounce(&mut self);
    fn play_shuffle(&mut self);
    fn set_muted(&mut self, muted: bool);
    fn update_game(&mut self, input: &egor::input::Input, delta: f32, game: &mut Game);
}

impl SoundManagerOption for Option<SoundManager> {
    fn play_bounce(&mut self) {
        println!("Play bounce");
        if let Some(mgr) = self.as_mut() {
            println!("Play bounce for real");

            mgr.play_bounce();
        }
    }

    fn play_shuffle(&mut self) {
        if let Some(mgr) = self.as_mut() {
            mgr.play_shuffle();
        }
    }

    fn set_muted(&mut self, muted: bool) {
        if let Some(mgr) = self.as_mut() {
            mgr.set_muted(muted);
        }
    }

    fn update_game(&mut self, input: &egor::input::Input, delta: f32, game: &mut Game) {
        if let Some(mgr) = self.as_mut() {
            game.update(input, delta, mgr);
        }
    }
}

trait MusicManagerOption {
    fn update(&mut self);
    fn start(&mut self);
    fn set_muted(&mut self, muted: bool);
    fn play_game_over_song(&mut self);
    fn get_mut(&mut self) -> Option<&mut MusicManager>;
}

impl MusicManagerOption for Option<MusicManager> {
    fn update(&mut self) {
        if let Some(mgr) = self.as_mut() {
            mgr.update();
        }
    }

    fn start(&mut self) {
        if let Some(mgr) = self.as_mut() {
            mgr.start();
        }
    }

    fn set_muted(&mut self, muted: bool) {
        if let Some(mgr) = self.as_mut() {
            mgr.set_muted(muted);
        }
    }

    fn play_game_over_song(&mut self) {
        if let Some(mgr) = self.as_mut() {
            mgr.play_game_over_song();
        }
    }

    fn get_mut(&mut self) -> Option<&mut MusicManager> {
        self.as_mut()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Title,
    Playing,
    VolumeControl,
    GameOver,
}

fn main() {
    // Initialize panic hook for better error messages in WASM
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // Check if volume settings exist in storage - if yes, go to Title, else VolumeControl
    let mut state = if crate::storage::Storage::has_volume_settings() {
        GameState::Title
    } else {
        GameState::VolumeControl
    };

    let mut title_screen = TitleScreen::new();
    let mut game: Option<Game> = None;
    let mut background = Background::new(100);
    let mut was_focused = true;
    let mut unfocused_timer: Option<f32> = None;
    let mut muted_due_to_unfocused = false; // Track if we muted due to unfocused timeout
    const UNFOCUSED_MUTE_DELAY: f32 = 15.0; // seconds

    // Create shared volume manager
    let volume_manager = VolumeManager::new();
    //let mut loading_screen = LoadingScreen::new(&volume_manager);

    // Create audio managers (lazy loaded in WASM, immediate in native)
    let (mut sound_manager, mut music_manager) = {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native: create immediately and wrap in Some
            let (sound_result, music_result) = create_audio_managers(&volume_manager);
            (
                Some(sound_result.expect("Failed to create sound manager")),
                Some(music_result.expect("Failed to create music manager")),
            )
        }

        #[cfg(target_arch = "wasm32")]
        {
            // WASM: start as None - will be initialized on user interaction
            (None, None)
        }
    };

    // Create small mute button for bottom right
    let mut mute_button_small = MuteButton::for_bottom_right();

    // Create volume control button for bottom left
    let mut volume_button = MuteButton::for_bottom_left();

    // Create volume control screen
    let mut volume_control_screen = VolumeControlScreen::new(&volume_manager);
    let mut previous_state = GameState::Title; // Track state before opening volume control

    // Create game over screen
    let mut game_over_screen = GameOverScreen::new();
    #[cfg(not(target_arch = "wasm32"))]
    {
        if state == GameState::Title {
            music_manager.start();
        }
    }

    App::new()
        .title("Retris")
        .maximized(true)
        .vsync(true)
        .run(move |gfx, input, timer| {
            let is_focused = input.has_focus();
            // Check if JavaScript requested to start music/audio (only once)
            // This is when we initialize the audio managers in WASM
            #[cfg(target_arch = "wasm32")]
            {
                if START_MUSIC_FLAG
                    .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    // User has interacted - now we can create audio managers
                    // Create both managers - they need to be created after user interaction for AudioContext
                    if sound_manager.is_none() || music_manager.is_none() {
                        let (sound_result, music_result) = create_audio_managers(&volume_manager);

                        // Create sound manager if it doesn't exist
                        if sound_manager.is_none() {
                            match sound_result {
                                Ok(sound_mgr) => {
                                    println!("SoundManager created successfully");
                                    sound_manager = Some(sound_mgr);
                                }
                                Err(e) => {
                                    eprintln!("Failed to create SoundManager: {}", e);
                                }
                            }
                        }

                        // Create music manager if it doesn't exist
                        if music_manager.is_none() {
                            match music_result {
                                Ok(music_mgr) => {
                                    println!("MusicManager created successfully");
                                    music_manager = Some(music_mgr);
                                    // Start music after initialization
                                    music_manager.start();
                                }
                                Err(e) => {
                                    eprintln!("Failed to create MusicManager: {}", e);
                                }
                            }
                        }
                    }
                }
            }

            // Load textures on first frame
            if timer.frame == 0 {
                mute_button_small.load_textures(gfx);
                volume_button.load_textures(gfx);
            }
            // Update and draw animated starfield background
            let screen = gfx.screen_size();
            background.update_screen_size(screen.x, screen.y);
            background.update(timer.delta);
            background.draw(gfx);

            match state {
                GameState::Title => {
                    // Update music (check for song transitions)
                    music_manager.update();

                    // Play sounds for title screen interactions
                    if input.key_pressed(KeyCode::ArrowLeft)
                        || input.key_pressed(KeyCode::ArrowRight)
                        || input.key_pressed(KeyCode::ArrowDown)
                    {
                        sound_manager.play_bounce();
                    }
                    if input.key_pressed(KeyCode::Space) {
                        sound_manager.play_shuffle();
                    }

                    title_screen.update(input, timer.delta);
                    let screen = gfx.screen_size();
                    title_screen.update_screen_size(screen.x, screen.y);
                    title_screen.draw(gfx, timer.delta);
                    volume_button.update(gfx);
                    volume_button.draw(gfx);

                    if volume_button.is_clicked(input) {
                        previous_state = GameState::Title;
                        state = GameState::VolumeControl;
                    }

                    // Check for Enter key to start game
                    if input.key_pressed(KeyCode::Enter) || input.mouse_pressed(MouseButton::Left) {
                        let screen = gfx.screen_size();
                        game = Some(Game::new(screen.x, screen.y));
                        state = GameState::Playing;
                    }
                }
                GameState::Playing => {
                    // Update music (check for song transitions)
                    music_manager.update();

                    if let Some(ref mut g) = game {
                        sound_manager.update_game(input, timer.delta, g);
                        g.draw(gfx, timer.delta);

                        // Check for game over condition
                        if g.is_gameover {
                            // Save high score if this is a new record
                            let current_score = g.score_manager().score();
                            let high_score = g.score_manager().high_score();
                            if current_score > high_score {
                                // Save to storage
                                use crate::storage::{GameData, Storage};
                                Storage::save_game_data(&GameData {
                                    high_score: current_score,
                                });
                                // Update high score in score manager
                                g.score_manager_mut().set_high_score(current_score);
                            }
                            // Play game over song (stops other music)
                            music_manager.play_game_over_song();
                            state = GameState::GameOver;
                        }
                    }

                    // Update button positions based on screen dimensions
                    volume_button.update(gfx);

                    // Draw volume control button in bottom left
                    volume_button.draw(gfx);

                    // Handle volume button click
                    if volume_button.is_clicked(input) {
                        previous_state = GameState::Playing;
                        state = GameState::VolumeControl;
                    }

                    // Restart on R key
                    if input.key_pressed(KeyCode::KeyR) {
                        let screen = gfx.screen_size();
                        game = Some(Game::new(screen.x, screen.y));
                    }

                    // Return to title on Escape, Q key, or mobile quit button
                    if input.key_pressed(KeyCode::Escape) || input.key_pressed(KeyCode::KeyQ) {
                        game = None;
                        state = GameState::Title;
                    }

                    // Check mobile controller quit button
                    if let Some(ref g) = game {
                        if g.mobile_quit_pressed() {
                            game = None;
                            state = GameState::Title;
                        }
                    }
                }
                GameState::GameOver => {
                    // Update music (check for song transitions)
                    music_manager.update();

                    // Update and handle game over screen actions
                    let screen = gfx.screen_size();
                    game_over_screen.update(screen.x, screen.y);
                    match game_over_screen.handle_input(input, screen.x, screen.y) {
                        GameOverAction::Quit => {
                            // Exit the application
                            std::process::exit(0);
                        }
                        GameOverAction::BackToMenu => {
                            // Resume regular playlist when returning to menu (will check muted internally)
                            music_manager.start();
                            game = None;
                            state = GameState::Title;
                        }
                        GameOverAction::Retry => {
                            // Resume regular playlist when retrying (will check muted internally)
                            music_manager.start();
                            let screen = gfx.screen_size();
                            game = Some(Game::new(screen.x, screen.y));
                            state = GameState::Playing;
                        }
                        GameOverAction::None => {
                            // Continue showing game over screen
                        }
                    }

                    // Draw game over screen with score details
                    if let Some(ref g) = game {
                        let screen = gfx.screen_size();
                        game_over_screen.draw(gfx, g.score_manager(), screen.x, screen.y);
                    }
                }
                GameState::VolumeControl => {
                    let screen = gfx.screen_size();
                    volume_control_screen.draw(gfx, screen.x, screen.y);
                    mute_button_small.update(gfx);
                    if mute_button_small.is_clicked(input) {
                        mute_button_small.toggle();
                        let is_muted = mute_button_small.is_muted();
                        music_manager.set_muted(is_muted);
                        sound_manager.set_muted(is_muted);
                        music_manager.start();
                    }
                    mute_button_small.draw(gfx);
                    if let Some(ref mut music_mgr) = music_manager.get_mut() {
                        if let Some(ref mut sound_mgr) = sound_manager.as_mut() {
                            if volume_control_screen.update(
                                timer.delta,
                                input,
                                music_mgr,
                                sound_mgr,
                                &volume_manager,
                                screen.x,
                                screen.y,
                            ) {
                                state = previous_state;
                            }
                        }
                    }
                }
            }
            if is_focused != was_focused {
                if !is_focused {
                    // Just lost focus - start the timer
                    unfocused_timer = Some(0.0);
                    muted_due_to_unfocused = false; // Reset flag
                } else {
                    // Regained focus - cancel timer
                    unfocused_timer = None;
                    // Only unmute and restart if we muted due to the unfocused timeout
                    if muted_due_to_unfocused {
                        music_manager.set_muted(false);
                        sound_manager.set_muted(false);
                        music_manager.start();
                        muted_due_to_unfocused = false;
                    }
                    // Otherwise music was never muted, so nothing to do
                }
                was_focused = is_focused;
            }

            // Update unfocused timer and mute if delay has passed
            if !is_focused {
                if let Some(ref mut elapsed) = unfocused_timer {
                    *elapsed += timer.delta;
                    if *elapsed >= UNFOCUSED_MUTE_DELAY {
                        // 15 seconds have passed - now mute
                        music_manager.set_muted(true);
                        sound_manager.set_muted(true);
                        muted_due_to_unfocused = true; // Mark that we muted due to timeout
                        // Clear timer so we don't keep setting muted every frame
                        unfocused_timer = None;
                    }
                }
            }
        })
}
