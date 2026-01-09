use crate::coordinate_system::CoordinateSystem;
use crate::game_data::ScoreManager;
use crate::retris_colors::*;
use egor::math::vec2;
use egor::render::Graphics;

/// Renders the game UI (score, level, etc.) behind the game board
/// Text gets obscured by blocks as the player fills the board
pub struct GameUI {}

impl GameUI {
    pub fn new() -> Self {
        Self {}
    }

    /// Draw the game UI behind the board
    /// This should be called BEFORE drawing the grid and pieces
    pub fn draw(&self, gfx: &mut Graphics, score_manager: &ScoreManager) {
        let screen = gfx.screen_size();
        let screen_height = screen.y;
        
        // Calculate scale factor based on screen height (normalize to 1048px reference)
        let scale_factor = screen_height / 1048.0;
        
        // Clamp scale factor to prevent text from becoming too small or too large
        let scale_factor = scale_factor.max(0.5).min(2.0);
        
        let score = score_manager.score();
        let level = score_manager.level();
        let lines = score_manager.lines_cleared();
        let multiplier = score_manager.multiplier();
        let combo = score_manager.combo_count();

        // Draw "Press Q to quit game" at the top of the screen
        let quit_y = -450.0 * scale_factor;
        let quit_size = (screen_height * 0.019).max(16.0).min(32.0);
        self.draw_centered_text(gfx, "Press Q to quit game", quit_y, quit_size, COLOR_DARK_GRAY);

        // Draw large score in the center
        let score_text = format!("{}", score);
        let score_y = -100.0 * scale_factor;
        let score_size = (screen_height * 0.115).max(60.0).min(200.0);
        self.draw_centered_text(gfx, &score_text, score_y, score_size, COLOR_TEXT_GREEN);

        // Draw level above score with level multiplier indicator
        let level_mult = match level {
            0..=4 => "x1",
            5..=9 => "x2",
            10..=14 => "x3",
            15..=19 => "x5",
            _ => "x8",
        };
        let level_text = format!("LEVEL {} ({})", level, level_mult);
        let level_y = -200.0 * scale_factor;
        let level_size = (screen_height * 0.038).max(24.0).min(64.0);
        self.draw_centered_text(gfx, &level_text, level_y, level_size, COLOR_TEXT_GREEN);

        // Draw lines below score
        let lines_text = format!("LINES: {}", lines);
        let lines_y = 50.0 * scale_factor;
        let lines_size = (screen_height * 0.031).max(20.0).min(48.0);
        self.draw_centered_text(gfx, &lines_text, lines_y, lines_size, COLOR_TEXT_GREEN);

        // Draw multiplier
        if multiplier > 1 {
            let mult_text = format!("{}x MULTIPLIER", multiplier);
            let mult_y = 100.0 * scale_factor;
            let mult_size = (screen_height * 0.034).max(22.0).min(52.0);
            self.draw_centered_text(gfx, &mult_text, mult_y, mult_size, COLOR_ORANGE);
        }

        // Draw combo (if active)
        if combo > 1 {
            let combo_text = format!("COMBO x{}", combo);
            let combo_y = 150.0 * scale_factor;
            let combo_size = (screen_height * 0.038).max(24.0).min(64.0);
            self.draw_centered_text(gfx, &combo_text, combo_y, combo_size, COLOR_MAGENTA);
        }
    }

    /// Helper to draw centered text
    /// world_y: Y position in world coordinates (0 is center of screen)
    fn draw_centered_text(
        &self,
        gfx: &mut Graphics,
        text: &str,
        world_y: f32,
        size: f32,
        color: egor::render::Color,
    ) {
        // Use coordinate system with actual screen dimensions
        let screen = gfx.screen_size();
        let coords = CoordinateSystem::with_default_offset(screen.x, screen.y);
        
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

impl Default for GameUI {
    fn default() -> Self {
        Self::new()
    }
}
