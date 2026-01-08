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
mod tetris_shape;
mod title_screen;
mod volume_control_screen;
mod volume_manager;

use background::Background;
use egor::app::*;
use egor::input::KeyCode;
use game::Game;
use game_over_screen::{GameOverAction, GameOverScreen};
use music_manager::MusicManager;
use retris_ui::MuteButton;
use sound_manager::SoundManager;
use title_screen::TitleScreen;
use volume_control_screen::VolumeControlScreen;
use volume_manager::VolumeManager;

pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 1048;



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

    let mut state = GameState::Title;
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

    // Create sound manager and start loading in background (loads quickly)
    let mut sound_manager =
        SoundManager::new(volume_manager.clone()).expect("Failed to create sound manager");
    sound_manager.play_bounce();
    // Create music manager and start loading in background
    let mut music_manager =
        MusicManager::new(volume_manager.clone()).expect("Failed to create music manager");
    // Create small mute button for bottom right
    let mut mute_button_small = MuteButton::for_bottom_right();

    // Create volume control button for bottom left
    let mut volume_button = MuteButton::for_bottom_left();

    // Create volume control screen
    let mut volume_control_screen = VolumeControlScreen::new(&volume_manager);
    let mut previous_state = GameState::Title; // Track state before opening volume control

    // Create game over screen
    let game_over_screen = GameOverScreen::new();

    let (initial_width, initial_height) = get_viewport_size();

    App::new()
        .title("Retris")
        .screen_size_centered(initial_width, initial_height)
        .vsync(true)
        .run(move |gfx, input, timer| {
            let is_focused = input.has_focus();

            // Load textures on first frame
            if timer.frame == 0 {
                mute_button_small.load_textures(gfx);
                volume_button.load_textures(gfx);
            }
            // Update and draw animated starfield background
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
                    title_screen.draw(gfx, timer.delta);
                    volume_button.draw(gfx);

                    if volume_button.is_clicked(input) {
                        previous_state = GameState::Title;
                        state = GameState::VolumeControl;
                    }

                    // Check for Enter key to start game
                    if input.key_pressed(KeyCode::Enter) {
                        game = Some(Game::new());
                        state = GameState::Playing;
                    }
                }
                GameState::Playing => {
                    // Update music (check for song transitions)
                    music_manager.update();

                    if let Some(ref mut g) = game {
                        g.update(input, timer.delta, &mut sound_manager);
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

                    // Handle mute button toggle
                    if mute_button_small.is_clicked(input) {
                        mute_button_small.toggle();
                        let is_muted = mute_button_small.is_muted();
                        music_manager.set_muted(is_muted);
                        sound_manager.set_muted(is_muted);
                        // Start music (will check muted internally)
                        music_manager.start();
                    }

                    // Draw volume control button in bottom left
                    volume_button.draw(gfx);

                    // Handle volume button click
                    if volume_button.is_clicked(input) {
                        previous_state = GameState::Playing;
                        state = GameState::VolumeControl;
                    }

                    // Restart on R key
                    if input.key_pressed(KeyCode::KeyR) {
                        game = Some(Game::new());
                    }

                    // Return to title on Escape
                    if input.key_pressed(KeyCode::Escape) || input.key_pressed(KeyCode::KeyQ) {
                        game = None;
                        state = GameState::Title;
                    }
                }
                GameState::GameOver => {
                    // Update music (check for song transitions)
                    music_manager.update();

                    // Handle game over screen actions
                    match game_over_screen.update(input) {
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
                            game = Some(Game::new());
                            state = GameState::Playing;
                        }
                        GameOverAction::None => {
                            // Continue showing game over screen
                        }
                    }

                    // Draw game over screen with score details
                    if let Some(ref g) = game {
                        game_over_screen.draw(gfx, g.score_manager());
                    }
                }
                GameState::VolumeControl => {
                    volume_control_screen.draw(gfx);
                    if mute_button_small.is_clicked(input) {
                        mute_button_small.toggle();
                        let is_muted = mute_button_small.is_muted();
                        music_manager.set_muted(is_muted);
                        sound_manager.set_muted(is_muted);
                        // Start music (will check muted internally)
                        music_manager.start();
                    }
                    mute_button_small.draw(gfx);
                    if volume_control_screen.update(
                        timer.delta,
                        input,
                        &mut music_manager,
                        &mut sound_manager,
                        &volume_manager,
                    ) {
                        state = previous_state;
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
