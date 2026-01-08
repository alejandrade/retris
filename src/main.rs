mod background;
mod game;
mod game_data;
mod game_ui;
mod grid;
mod loading_screen;
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
use loading_screen::LoadingScreen;
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
    Loading,
    Title,
    Playing,
    VolumeControl,
}

fn main() {
    let mut state = GameState::Loading;
    let mut title_screen = TitleScreen::new();
    let mut game: Option<Game> = None;
    let mut background = Background::new(100);

    // Create shared volume manager
    let volume_manager = VolumeManager::new();
    let mut loading_screen = LoadingScreen::new(&volume_manager);

    // Create sound manager and start loading in background (loads quickly)
    let mut sound_manager =
        SoundManager::new(volume_manager.clone()).expect("Failed to create sound manager");
    sound_manager.start_loading_background();

    // Create music manager and start loading in background
    let mut music_manager =
        MusicManager::new(volume_manager.clone()).expect("Failed to create music manager");
    music_manager.start_loading_background();

    // Create small mute button for bottom right
    let mut mute_button_small = MuteButton::for_bottom_right();

    // Create volume control button for bottom left
    let mut volume_button = MuteButton::for_bottom_left();

    // Create volume control screen
    let mut volume_control_screen = VolumeControlScreen::new(&volume_manager);
    let mut previous_state = GameState::Title; // Track state before opening volume control

    App::new()
        .title("Retris")
        .screen_size_centered(SCREEN_WIDTH, SCREEN_HEIGHT)
        .vsync(true)
        .run(move |gfx, input, timer| {
            // Load textures on first frame
            if timer.frame == 0 {
                mute_button_small.load_textures(gfx);
                volume_button.load_textures(gfx);
            }
            // Update and draw animated starfield background
            background.update(timer.delta);
            background.draw(gfx);

            match state {
                GameState::Loading => {
                    // Update loading screen
                    loading_screen.update(
                        timer.delta,
                        input,
                        &mut music_manager,
                        &mut sound_manager,
                        &volume_manager,
                    );

                    // Draw loading screen
                    let loading_state = music_manager.loading_state();
                    loading_screen.draw(gfx, &loading_state);

                    // Check if user clicked OK to continue
                    if loading_screen.is_ready_to_continue(input) {
                        // Start music
                        if !music_manager.is_muted() {
                            music_manager.start();
                        }
                        state = GameState::Title;
                    }
                }
                GameState::Title => {
                    // Update music (check for song transitions)
                    music_manager.update();

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
                    }

                    // Handle mute button toggle
                    if mute_button_small.is_clicked(input) {
                        mute_button_small.toggle();
                        let is_muted = mute_button_small.is_muted();
                        music_manager.set_muted(is_muted);
                        sound_manager.set_muted(is_muted);
                        if !is_muted {
                            music_manager.start();
                        }
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
                    if input.key_pressed(KeyCode::Escape) {
                        game = None;
                        state = GameState::Title;
                    }
                }
                GameState::VolumeControl => {
                    volume_control_screen.draw(gfx);
                    if mute_button_small.is_clicked(input) {
                        mute_button_small.toggle();
                        let is_muted = mute_button_small.is_muted();
                        music_manager.set_muted(is_muted);
                        sound_manager.set_muted(is_muted);
                        if !is_muted {
                            music_manager.start();
                        }
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
        })
}
