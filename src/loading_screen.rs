use crate::coordinate_system::CoordinateSystem;
use crate::music_manager::{LoadingState, MusicManager};
use crate::retris_colors::*;
use crate::retris_ui::{Button, MuteButton, VolumeSlider};
use crate::sound_manager::SoundManager;
use crate::volume_manager::VolumeManager;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use egor::input::Input;
use egor::math::vec2;
use egor::render::Graphics;

#[derive(PartialEq)]
enum LoadingScreenState {
    Loading,
    VolumeConfig,
    Ready, // Auto-ready for returning users
}

pub struct LoadingScreen {
    dots_timer: f32,
    dots_count: usize,
    state: LoadingScreenState,
    music_slider: VolumeSlider,
    sfx_slider: VolumeSlider,
    ok_button: Button,
    mute_button: MuteButton,
    test_sound_timer: f32, // Track how long test sound has been playing
    skip_volume_config: bool, // True if user already has saved settings
    loading_start_time: f32, // Track when loading started to ensure minimum display time
    min_loading_duration: f32, // Minimum time to show loading screen (in seconds)
}


impl LoadingScreen {
    pub fn new(volume_manager: &VolumeManager) -> Self {
        // Check if user already has customized volume settings (not default)
        // This is faster than checking storage again since VolumeManager already loaded them
        let skip_volume_config = !volume_manager.is_default();
        
        if skip_volume_config {
            println!("Found customized volume settings - skipping volume config screen");
        } else {
            println!("Using default volume settings - will show volume config screen");
        }
        
        Self {
            dots_timer: 0.0,
            dots_count: 0,
            state: LoadingScreenState::Loading,
            music_slider: VolumeSlider::new(-150.0, -50.0, 300.0, "Music Volume", volume_manager.music_volume()),
            sfx_slider: VolumeSlider::new(-150.0, 50.0, 300.0, "Sound Effects Volume", volume_manager.sfx_volume()),
            ok_button: Button::new(-75.0, 150.0, 150.0, 50.0, "OK"),
            mute_button: MuteButton::for_loading(),
            test_sound_timer: 0.0,
            skip_volume_config,
            loading_start_time: 0.0,
            min_loading_duration: 0.5, // Show loading screen for at least 0.5 seconds
        }
    }

    pub fn update(&mut self, delta: f32, input: &Input, music_manager: &mut MusicManager, sound_manager: &mut SoundManager, volume_manager: &VolumeManager) {
        // Update loading dots animation
        self.dots_timer += delta;
        if self.dots_timer >= 0.5 {
            self.dots_timer = 0.0;
            self.dots_count = (self.dots_count + 1) % 4;
        }
        
        // Track loading start time on first update
        if self.state == LoadingScreenState::Loading && self.loading_start_time == 0.0 {
            self.loading_start_time = 0.0; // Will be set to current time, but we track elapsed instead
        }
        
        // Check if loading is complete or at least one song is loaded - transition to appropriate state
        // But only if minimum display time has passed
        if self.state == LoadingScreenState::Loading {
            let elapsed = self.dots_timer; // Use dots_timer as a simple elapsed time tracker
            let min_time_passed = elapsed >= self.min_loading_duration;
            
            let loading_done = match music_manager.loading_state() {
                LoadingState::Loading { current, total: _ } => current >= 1,
                LoadingState::Complete => true,
                LoadingState::Failed(_) => true,
                LoadingState::NotStarted => false,
            };
            
            // Only transition if loading is done AND minimum time has passed
            if loading_done && min_time_passed {
                // Skip volume config if user already has saved settings
                if self.skip_volume_config {
                    self.state = LoadingScreenState::Ready;
                } else {
                    self.state = LoadingScreenState::VolumeConfig;
                }
            }
        }
        
        // Update volume sliders if in config state
        if self.state == LoadingScreenState::VolumeConfig {
            // Update test sound timer
            self.test_sound_timer += delta;
            
            // Stop test sound after 2 seconds
            if self.test_sound_timer >= 2.0 {
                music_manager.stop_test_sound();
                self.test_sound_timer = 0.0;
            }
            
            if self.music_slider.update(input) {
                volume_manager.set_music_volume(self.music_slider.value());
                music_manager.update_volume();
            }
            
            // Only play test sound and save when mouse is released
            if self.music_slider.was_just_released() {
                music_manager.test_sound();
                self.test_sound_timer = 0.0; // Reset timer when new test starts
                volume_manager.save(); // Save only on release
            }
            
            if self.sfx_slider.update(input) {
                volume_manager.set_sfx_volume(self.sfx_slider.value());
                sound_manager.update_volume();
            }
            
            // Save SFX volume only on release
            if self.sfx_slider.was_just_released() {
                sound_manager.test_sound();
                volume_manager.save(); // Save only on release
            }
        }
    }
    
    /// Check if ready to continue (either clicked OK or auto-ready for returning users)
    pub fn is_ready_to_continue(&self, input: &Input) -> bool {
        match self.state {
            LoadingScreenState::Ready => true, // Auto-ready for returning users
            LoadingScreenState::VolumeConfig => self.ok_button.is_clicked(input), // New users click OK
            _ => false,
        }
    }

    /// Draw the loading screen
    pub fn draw(&mut self, gfx: &mut Graphics, loading_state: &LoadingState) {
        match self.state {
            LoadingScreenState::Ready => {
                // Don't draw anything - we're ready to transition
            }
            LoadingScreenState::Loading => {
                // Draw "LOADING" text in center
                let loading_text = "LOADING";
                let dots = ".".repeat(self.dots_count);
                let full_text = format!("{}{}", loading_text, dots);
                
                self.draw_centered_text(gfx, &full_text, -200.0, 60.0, COLOR_TEXT_GREEN);
                
                // Draw subtitle based on loading state
                match loading_state {
                    LoadingState::NotStarted => {
                        self.draw_centered_text(gfx, "Initializing...", -120.0, 32.0, COLOR_DARK_GRAY);
                    }
                    LoadingState::Loading { current, total } => {
                        let progress_text = format!("Loading Music... {}/{}", current + 1, total);
                        self.draw_centered_text(gfx, &progress_text, -120.0, 32.0, COLOR_DARK_GRAY);
                    }
                    LoadingState::Complete => {
                        self.draw_centered_text(gfx, "Complete!", -120.0, 32.0, COLOR_TEXT_GREEN);
                    }
                    LoadingState::Failed(msg) => {
                        self.draw_centered_text(gfx, &format!("Error: {}", msg), -120.0, 28.0, COLOR_ORANGE);
                    }
                }
            }
            LoadingScreenState::VolumeConfig => {
                // Draw title
                self.draw_centered_text(gfx, "AUDIO SETTINGS", -200.0, 48.0, COLOR_TEXT_GREEN);
                self.draw_centered_text(gfx, "Adjust volumes to your preference", -150.0, 24.0, COLOR_DARK_GRAY);
                
                // Draw volume sliders
                self.music_slider.draw(gfx);
                self.sfx_slider.draw(gfx);
                
                // Draw OK button
                self.ok_button.draw(gfx);
            }
        }
    }
    
    /// Helper to draw centered text
    fn draw_centered_text(
        &self,
        gfx: &mut Graphics,
        text: &str,
        world_y: f32,
        size: f32,
        color: egor::render::Color,
    ) {
        let coords = CoordinateSystem::with_default_offset(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
        
        // Calculate world-space position (centered at x=0)
        let world_x = coords.center_text_x(text, size, 0.5);
        
        // Convert world coordinates to screen coordinates
        let screen_pos = coords.world_to_screen(vec2(world_x, world_y));

        gfx.text(text)
            .at(screen_pos)
            .size(size)
            .color(color);
    }
}

impl Default for LoadingScreen {
    fn default() -> Self {
        let volume_manager = VolumeManager::new();
        Self::new(&volume_manager)
    }
}
