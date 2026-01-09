use crate::coordinate_system::CoordinateSystem;
use crate::game_data::ScoreManager;
use crate::retris_colors::*;
use crate::retris_ui::Button;
use egor::input::Input;
use egor::render::Graphics;

pub struct GameOverScreen {
    quit_button: Button,
    back_to_menu_button: Button,
    retry_button: Button,
}

impl GameOverScreen {
    // Base constants for aspect-ratio-aware scaling
    const BASE_BUTTON_Y_OFFSET: f32 = 50.0; // Base Y position (normalized to 1048px height)

    // Percentage-based sizing for aspect-ratio-aware scaling
    const BUTTON_WIDTH_PERCENT: f32 = 0.31; // 31% of screen width
    const BUTTON_HEIGHT_PERCENT: f32 = 0.048; // 4.8% of screen height
    const BUTTON_SPACING_PERCENT: f32 = 0.014; // 1.4% of screen height

    // Min/max constraints to prevent extreme sizes
    const MIN_BUTTON_WIDTH: f32 = 150.0;
    const MAX_BUTTON_WIDTH: f32 = 300.0;
    const MIN_BUTTON_HEIGHT: f32 = 40.0;
    const MAX_BUTTON_HEIGHT: f32 = 80.0;
    const MIN_BUTTON_SPACING: f32 = 10.0;
    const MAX_BUTTON_SPACING: f32 = 25.0;

    pub fn new() -> Self {
        // Use default screen dimensions for initial calculation (will be updated via update)
        let default_width = 640.0;
        let default_height = 1048.0;

        // Calculate button dimensions using aspect-ratio-aware scaling
        let button_width = (default_width * Self::BUTTON_WIDTH_PERCENT)
            .max(Self::MIN_BUTTON_WIDTH)
            .min(Self::MAX_BUTTON_WIDTH);
        let button_height = (default_height * Self::BUTTON_HEIGHT_PERCENT)
            .max(Self::MIN_BUTTON_HEIGHT)
            .min(Self::MAX_BUTTON_HEIGHT);
        let button_spacing = (default_height * Self::BUTTON_SPACING_PERCENT)
            .max(Self::MIN_BUTTON_SPACING)
            .min(Self::MAX_BUTTON_SPACING);

        // Stack buttons vertically, centered horizontally
        // Button::new expects top-left corner, so we need to offset by half width
        let start_y = Self::BASE_BUTTON_Y_OFFSET * (default_height / 1048.0);
        let center_x = 0.0; // Center horizontally (world coordinate)
        let button_left_x = center_x - button_width / 2.0; // Top-left X position

        Self {
            quit_button: Button::new(button_left_x, start_y, button_width, button_height, "Quit"),
            back_to_menu_button: Button::new(
                button_left_x,
                start_y + button_height + button_spacing,
                button_width,
                button_height,
                "Back to Menu",
            ),
            retry_button: Button::new(
                button_left_x,
                start_y + (button_height + button_spacing) * 2.0,
                button_width,
                button_height,
                "Retry",
            ),
        }
    }

    /// Update button positions and sizes based on actual screen dimensions
    pub fn update(&mut self, screen_width: f32, screen_height: f32) {
        // Calculate button dimensions using aspect-ratio-aware scaling
        // Width scales with screen width, height scales with screen height
        let button_width = (screen_width * Self::BUTTON_WIDTH_PERCENT)
            .max(Self::MIN_BUTTON_WIDTH)
            .min(Self::MAX_BUTTON_WIDTH);
        let button_height = (screen_height * Self::BUTTON_HEIGHT_PERCENT)
            .max(Self::MIN_BUTTON_HEIGHT)
            .min(Self::MAX_BUTTON_HEIGHT);
        let button_spacing = (screen_height * Self::BUTTON_SPACING_PERCENT)
            .max(Self::MIN_BUTTON_SPACING)
            .min(Self::MAX_BUTTON_SPACING);

        // Stack buttons vertically, centered horizontally
        // Button::new expects top-left corner, so we need to offset by half width
        let start_y = Self::BASE_BUTTON_Y_OFFSET * (screen_height / 1048.0);
        let center_x = 0.0; // Center horizontally (world coordinate)
        let button_left_x = center_x - button_width / 2.0; // Top-left X position

        // Update button positions (Button doesn't have a method to update size, so we recreate them)
        // Note: Button::new() uses world coordinates with top-left corner
        self.quit_button = Button::new(button_left_x, start_y, button_width, button_height, "Quit");
        self.back_to_menu_button = Button::new(
            button_left_x,
            start_y + button_height + button_spacing,
            button_width,
            button_height,
            "Back to Menu",
        );
        self.retry_button = Button::new(
            button_left_x,
            start_y + (button_height + button_spacing) * 2.0,
            button_width,
            button_height,
            "Retry",
        );

        // Also call update in case Button has its own update logic
        self.quit_button.update(screen_width, screen_height);
        self.back_to_menu_button.update(screen_width, screen_height);
        self.retry_button.update(screen_width, screen_height);
    }

    /// Handle input for game over screen
    pub fn handle_input(
        &self,
        input: &Input,
        screen_width: f32,
        screen_height: f32,
    ) -> GameOverAction {
        if self
            .quit_button
            .is_clicked(input, screen_width, screen_height)
        {
            GameOverAction::Quit
        } else if self
            .back_to_menu_button
            .is_clicked(input, screen_width, screen_height)
        {
            GameOverAction::BackToMenu
        } else if self
            .retry_button
            .is_clicked(input, screen_width, screen_height)
        {
            GameOverAction::Retry
        } else {
            GameOverAction::None
        }
    }

    pub fn draw(
        &self,
        gfx: &mut Graphics,
        score_manager: &ScoreManager,
        screen_width: f32,
        screen_height: f32,
    ) {
        // Use coordinate system with actual screen dimensions
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);

        // Calculate scale factor for Y positions (normalize to 1048px reference)
        // Text sizes already use percentage-based scaling, so they're aspect-ratio-aware
        let scale_factor = (screen_height / 1048.0).max(0.5).min(2.0);

        // Draw "GAME OVER" text in the center
        let title_text = "GAME OVER";
        let title_size = (screen_height * 0.069).max(36.0).min(144.0);
        // Use center_text_x which properly calculates the left edge to center the text
        // This ensures equal spacing on both sides
        let title_world_x = coords.center_text_x(title_text, title_size, 0.5);
        let title_world_y = -250.0 * scale_factor; // Position higher up
        let title_screen_pos = coords.world_to_screen(egor::math::vec2(
            title_world_x - (title_size / 2.0),
            title_world_y,
        ));

        gfx.text(title_text)
            .at(title_screen_pos)
            .size(title_size)
            .color(COLOR_TEXT_GREEN);

        // Draw score details
        let score = score_manager.score();
        let high_score = score_manager.high_score();
        let level = score_manager.level();
        let lines = score_manager.lines_cleared();
        let is_new_high = score > high_score;

        // Draw final score
        let score_text = format!("Final Score: {}", score);
        let score_size = (screen_height * 0.034).max(18.0).min(72.0);
        let score_world_x = coords.center_text_x(&score_text, score_size, 0.5);
        let score_world_y = -100.0 * scale_factor;
        let score_screen_pos =
            coords.world_to_screen(egor::math::vec2(score_world_x, score_world_y));
        gfx.text(&score_text)
            .at(score_screen_pos)
            .size(score_size)
            .color(if is_new_high {
                COLOR_ORANGE
            } else {
                COLOR_TEXT_GREEN
            });

        // Draw high score
        let high_score_text = if is_new_high {
            format!("NEW HIGH SCORE: {}!", score)
        } else {
            format!("High Score: {}", high_score)
        };
        let high_score_size = (screen_height * 0.027).max(14.0).min(56.0);
        let high_score_world_x = coords.center_text_x(&high_score_text, high_score_size, 0.5);
        let high_score_world_y = -50.0 * scale_factor;
        let high_score_screen_pos =
            coords.world_to_screen(egor::math::vec2(high_score_world_x, high_score_world_y));
        gfx.text(&high_score_text)
            .at(high_score_screen_pos)
            .size(high_score_size)
            .color(if is_new_high {
                COLOR_ORANGE
            } else {
                COLOR_DARK_GRAY
            });

        // Draw level and lines
        let stats_text = format!("Level: {} | Lines: {}", level, lines);
        let stats_size = (screen_height * 0.023).max(12.0).min(48.0);
        let stats_world_x = coords.center_text_x(&stats_text, stats_size, 0.5);
        let stats_world_y = 0.0;
        let stats_screen_pos =
            coords.world_to_screen(egor::math::vec2(stats_world_x, stats_world_y));
        gfx.text(&stats_text)
            .at(stats_screen_pos)
            .size(stats_size)
            .color(COLOR_DARK_GRAY);

        // Draw buttons (positions should be updated via update() before calling)
        self.quit_button.draw(gfx, screen_width, screen_height);
        self.back_to_menu_button
            .draw(gfx, screen_width, screen_height);
        self.retry_button.draw(gfx, screen_width, screen_height);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameOverAction {
    None,
    Quit,
    BackToMenu,
    Retry,
}

impl Default for GameOverScreen {
    fn default() -> Self {
        Self::new()
    }
}
