use crate::coordinate_system::CoordinateSystem;
use crate::game_data::ScoreManager;
use crate::retris_colors::*;
use crate::retris_ui::Button;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use egor::input::Input;
use egor::render::Graphics;

pub struct GameOverScreen {
    quit_button: Button,
    back_to_menu_button: Button,
    retry_button: Button,
}

impl GameOverScreen {
    pub fn new() -> Self {
        // Buttons are 150px wide, spaced 20px apart, centered on screen
        // Button positions are world coordinates (top-left corner)
        Self {
            quit_button: Button::new(-235.0, 100.0, 150.0, 50.0, "Quit"),
            back_to_menu_button: Button::new(-65.0, 100.0, 150.0, 50.0, "Back to Menu"),
            retry_button: Button::new(105.0, 100.0, 150.0, 50.0, "Retry"),
        }
    }

    pub fn update(&self, input: &Input) -> GameOverAction {
        if self.quit_button.is_clicked(input) {
            GameOverAction::Quit
        } else if self.back_to_menu_button.is_clicked(input) {
            GameOverAction::BackToMenu
        } else if self.retry_button.is_clicked(input) {
            GameOverAction::Retry
        } else {
            GameOverAction::None
        }
    }

    pub fn draw(&self, gfx: &mut Graphics, score_manager: &ScoreManager) {
        let coords = CoordinateSystem::with_default_offset(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

        // Draw "GAME OVER" text in the center
        let title_text = "GAME OVER";
        let title_size = 72.0;
        let title_world_x = coords.center_text_x(title_text, title_size, 0.5);
        let title_world_y = -200.0;
        let title_screen_pos = coords.world_to_screen(egor::math::vec2(title_world_x, title_world_y));

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
        let score_size = 36.0;
        let score_world_x = coords.center_text_x(&score_text, score_size, 0.5);
        let score_world_y = -100.0;
        let score_screen_pos = coords.world_to_screen(egor::math::vec2(score_world_x, score_world_y));
        gfx.text(&score_text)
            .at(score_screen_pos)
            .size(score_size)
            .color(if is_new_high { COLOR_ORANGE } else { COLOR_TEXT_GREEN });

        // Draw high score
        let high_score_text = if is_new_high {
            format!("NEW HIGH SCORE: {}!", score)
        } else {
            format!("High Score: {}", high_score)
        };
        let high_score_size = 28.0;
        let high_score_world_x = coords.center_text_x(&high_score_text, high_score_size, 0.5);
        let high_score_world_y = -50.0;
        let high_score_screen_pos = coords.world_to_screen(egor::math::vec2(high_score_world_x, high_score_world_y));
        gfx.text(&high_score_text)
            .at(high_score_screen_pos)
            .size(high_score_size)
            .color(if is_new_high { COLOR_ORANGE } else { COLOR_DARK_GRAY });

        // Draw level and lines
        let stats_text = format!("Level: {} | Lines: {}", level, lines);
        let stats_size = 24.0;
        let stats_world_x = coords.center_text_x(&stats_text, stats_size, 0.5);
        let stats_world_y = 0.0;
        let stats_screen_pos = coords.world_to_screen(egor::math::vec2(stats_world_x, stats_world_y));
        gfx.text(&stats_text)
            .at(stats_screen_pos)
            .size(stats_size)
            .color(COLOR_DARK_GRAY);

        // Draw buttons
        self.quit_button.draw(gfx);
        self.back_to_menu_button.draw(gfx);
        self.retry_button.draw(gfx);
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
