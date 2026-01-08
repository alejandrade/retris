use crate::game_data::ScoreManager;
use crate::retris_colors::*;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
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
        let score = score_manager.score();
        let level = score_manager.level();
        let lines = score_manager.lines_cleared();
        let multiplier = score_manager.multiplier();
        let combo = score_manager.combo_count();

        // Draw large score in the center
        let score_text = format!("{}", score);
        self.draw_centered_text(gfx, &score_text, -100.0, 120.0, COLOR_TEXT_GREEN);

        // Draw level above score with level multiplier indicator
        let level_mult = match level {
            0..=4 => "x1",
            5..=9 => "x2",
            10..=14 => "x3",
            15..=19 => "x5",
            _ => "x8",
        };
        let level_text = format!("LEVEL {} ({})", level, level_mult);
        self.draw_centered_text(gfx, &level_text, -200.0, 40.0, COLOR_TEXT_GREEN);

        // Draw lines below score
        let lines_text = format!("LINES: {}", lines);
        self.draw_centered_text(gfx, &lines_text, 50.0, 32.0, COLOR_TEXT_GREEN);

        // Draw multiplier
        if multiplier > 1 {
            let mult_text = format!("{}x MULTIPLIER", multiplier);
            self.draw_centered_text(gfx, &mult_text, 100.0, 36.0, COLOR_ORANGE);
        }

        // Draw combo (if active)
        if combo > 1 {
            let combo_text = format!("COMBO x{}", combo);
            self.draw_centered_text(gfx, &combo_text, 150.0, 40.0, COLOR_MAGENTA);
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
        // Estimate text width for centering
        let chars_per_pixel = 0.5; // Estimate: each character is ~0.5 * font_size wide
        let estimated_width = text.len() as f32 * size * chars_per_pixel;
        
        // Calculate world-space position (centered at x=0)
        let world_x = -estimated_width / 2.0;
        
        // Convert world coordinates to screen coordinates
        // World (0,0) is at screen center
        let screen_x = world_x + (SCREEN_WIDTH as f32 / 2.0);
        let screen_y = world_y + (SCREEN_HEIGHT as f32 / 2.0);

        gfx.text(text)
            .at(vec2(screen_x, screen_y))
            .size(size)
            .color(color);
    }
}

impl Default for GameUI {
    fn default() -> Self {
        Self::new()
    }
}
