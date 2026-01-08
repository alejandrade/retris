use crate::music_manager::MusicManager;
use crate::retris_colors::*;
use crate::retris_ui::{Button, VolumeSlider};
use crate::sound_manager::SoundManager;
use crate::volume_manager::VolumeManager;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use egor::input::Input;
use egor::math::vec2;
use egor::render::Graphics;

/// Volume control screen - accessible from anywhere in the game
pub struct VolumeControlScreen {
    music_slider: VolumeSlider,
    sfx_slider: VolumeSlider,
    close_button: Button,
    test_sound_timer: f32,
}

impl VolumeControlScreen {
    pub fn new(volume_manager: &VolumeManager) -> Self {
        Self {
            music_slider: VolumeSlider::new(
                -150.0,
                -50.0,
                300.0,
                "Music Volume",
                volume_manager.music_volume(),
            ),
            sfx_slider: VolumeSlider::new(
                -150.0,
                50.0,
                300.0,
                "Sound Effects Volume",
                volume_manager.sfx_volume(),
            ),
            close_button: Button::new(-75.0, 150.0, 150.0, 50.0, "Close"),
            test_sound_timer: 0.0,
        }
    }

    /// Update the volume control screen
    pub fn update(
        &mut self,
        delta: f32,
        input: &Input,
        music_manager: &mut MusicManager,
        sound_manager: &mut SoundManager,
        volume_manager: &VolumeManager,
    ) -> bool {
        // Update test sound timer
        self.test_sound_timer += delta;

        // Stop test sound after 2 seconds
        if self.test_sound_timer >= 2.0 {
            music_manager.stop_test_sound();
            self.test_sound_timer = 0.0;
        }

        // Update music slider
        if self.music_slider.update(input) {
            volume_manager.set_music_volume(self.music_slider.value());
            music_manager.update_volume();
        }

        // Play test sound and save when mouse is released
        if self.music_slider.was_just_released() {
            music_manager.test_sound();
            self.test_sound_timer = 0.0;
            volume_manager.save();
        }

        // Update SFX slider
        if self.sfx_slider.update(input) {
            volume_manager.set_sfx_volume(self.sfx_slider.value());
            sound_manager.update_volume();
        }

        // Play test sound and save when mouse is released
        if self.sfx_slider.was_just_released() {
            sound_manager.test_sound();
            volume_manager.save();
        }

        // Return true if user clicked Close button
        self.close_button.is_clicked(input)
    }

    /// Draw the volume control screen
    pub fn draw(&self, gfx: &mut Graphics) {
        // Draw semi-transparent background overlay
        // Since (0,0) is center, we need to position from top-left corner
        let overlay_size = vec2(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
        let overlay_pos = vec2(-(SCREEN_WIDTH as f32 / 2.0), -(SCREEN_HEIGHT as f32 / 2.0));
        gfx.rect()
            .at(overlay_pos)
            .size(overlay_size)
            .color(COLOR_DARK_GRAY);

        // Draw title
        self.draw_centered_text(gfx, "VOLUME CONTROL", -200.0, 48.0, COLOR_TEXT_GREEN);

        // Draw sliders
        self.music_slider.draw(gfx);
        self.sfx_slider.draw(gfx);

        // Draw close button
        self.close_button.draw(gfx);
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
        let chars_per_pixel = 0.5;
        let estimated_width = text.len() as f32 * size * chars_per_pixel;

        let world_x = -estimated_width / 2.0;
        let screen_x = world_x + (SCREEN_WIDTH as f32 / 2.0);
        let screen_y = world_y + (SCREEN_HEIGHT as f32 / 2.0);

        gfx.text(text)
            .at(vec2(screen_x, screen_y))
            .size(size)
            .color(color);
    }
}
