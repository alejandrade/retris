use crate::game_data::ScoreManager;
use crate::game_ui::GameUI;
use crate::grid::Grid;
use crate::sound_manager::SoundManager;
use crate::tetris_shape::TetrisShapeNode;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use egor::input::Input;
use egor::render::Graphics;

/// Grid width in cells
const GRID_WIDTH_CELLS: usize = 10;

/// Grid height in cells (visible playfield)
const GRID_HEIGHT_CELLS: usize = 20;

/// Minimum padding (in pixels) around the grid on all sides
const MIN_PADDING: f32 = 40.0;

/// Initial falling velocity for new pieces (cells per second)
const SPAWN_VELOCITY: u16 = 2;

/// Row index where new pieces spawn (top row of the grid, in spawn area)
const SPAWN_ROW: i32 = 0;

/// Duration of the level transition cascade effect (in seconds)
const LEVEL_TRANSITION_DURATION: f32 = 1.5;

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    LevelTransition { timer: f32 },
}

pub struct Game {
    active_piece: Option<TetrisShapeNode>,
    grid: Grid,
    score_manager: ScoreManager,
    ui: GameUI,
    state: GameState,
}

impl Game {
    pub fn new() -> Self {
        Self {
            active_piece: None,
            grid: Grid::new(
                SCREEN_WIDTH as f32,
                SCREEN_HEIGHT as f32,
                GRID_WIDTH_CELLS,
                GRID_HEIGHT_CELLS,
                MIN_PADDING,
            ),
            score_manager: ScoreManager::new(),
            ui: GameUI::new(),
            state: GameState::Playing,
        }
    }

    pub fn update(&mut self, input: &Input, fixed_delta: f32, sound_manager: &mut SoundManager) {
        match self.state {
            GameState::LevelTransition { timer } => {
                // Update cascade animation
                let new_timer = timer + fixed_delta;
                
                if new_timer >= LEVEL_TRANSITION_DURATION {
                    // Transition complete - resume playing
                    self.state = GameState::Playing;
                    self.grid.clear_cascade_animation();
                    println!("Level {} - GO!", self.score_manager.level());
                } else {
                    // Continue cascade animation
                    self.state = GameState::LevelTransition { timer: new_timer };
                    self.grid.update_cascade_animation(new_timer / LEVEL_TRANSITION_DURATION);
                }
            }
            GameState::Playing => {
                self.grid.update(input, fixed_delta);

                // Check if we need to spawn a new piece first (before updating)
                let needs_spawn = self.active_piece.is_none();
                if needs_spawn {
                    self.spawn_new_piece();
                }

                // Update the active piece if it exists and isn't stopped
                if let Some(ref mut piece) = self.active_piece {
                    if !piece.stopped {
                        piece.update(input, fixed_delta, &mut self.grid, sound_manager);
                    }
                }

                // Check if the piece stopped and transfer it to the grid
                if let Some(piece) = self.active_piece.take() {
                    if piece.stopped {
                        // Play bounce sound when piece lands
                        sound_manager.play_bounce();
                        
                        let cells_with_colors = piece.get_occupied_cells_with_color();
                        self.grid.mark_cells_occupied(&cells_with_colors);
                        
                        // Clear completed lines and update score
                        let lines_cleared = self.grid.clear_completed_lines();
                        
                        if lines_cleared > 0 {
                            // Play success sound when lines cleared
                            sound_manager.play_success();
                            
                            // Award points for clearing lines
                            let old_level = self.score_manager.level();
                            let points = self.score_manager.on_rows_cleared(lines_cleared as u32);
                            let new_level = self.score_manager.level();
                            let combo = self.score_manager.combo_count();
                            let multiplier = self.score_manager.multiplier();
                            let total_score = self.score_manager.score();
                            
                            // Show different messages for special clears
                            let clear_name = match lines_cleared {
                                1 => "Single",
                                2 => "Double",
                                3 => "Triple",
                                4 => "🎆 TETRIS",
                                _ => "Multi",
                            };
                            
                            // Add level indicator for high levels
                            let level_indicator = if new_level >= 20 {
                                " 🚀"
                            } else if new_level >= 15 {
                                " ⚡"
                            } else if new_level >= 10 {
                                " 💪"
                            } else {
                                ""
                            };
                            
                            if combo > 2 {
                                println!("🔥💥 {} COMBO! {}{} +{} points! ({}x next) [Lv{} | Total: {}]", 
                                    combo, clear_name, level_indicator, points, multiplier, new_level, total_score);
                            } else if combo > 1 {
                                println!("🔥 COMBO x{}! {}{} +{} points! ({}x next) [Lv{} | Total: {}]", 
                                    combo, clear_name, level_indicator, points, multiplier, new_level, total_score);
                            } else {
                                println!("{}{} cleared! +{} points ({}x next) [Lv{} | Total: {}]", 
                                    clear_name, level_indicator, points, multiplier, new_level, total_score);
                            }
                            
                            // Check for level up
                            if new_level > old_level {
                                sound_manager.play_level_up();
                                self.start_level_transition();
                            }
                        } else {
                            // Piece landed without clearing lines - reset multiplier
                            self.score_manager.on_piece_landed_no_clear();
                            println!("💔 Combo broken! Multiplier reset.");
                        }
                    } else {
                        self.active_piece = Some(piece);
                    }
                }
            }
        }
    }

    fn start_level_transition(&mut self) {
        println!("🎉 LEVEL UP! Now level {}", self.score_manager.level());
        self.state = GameState::LevelTransition { timer: 0.0 };
        self.active_piece = None; // Clear active piece during transition
        self.grid.start_cascade_animation();
    }

    fn spawn_new_piece(&mut self) {
        let grid_pos = self.grid.position();
        let cell_size = self.grid.cell_size();
        let grid_width = self.grid.width_cells();

        let spawn_cell_x = (grid_width / 2) as i32;
        let spawn_cell_y = SPAWN_ROW;

        let new_piece = TetrisShapeNode::new(
            SPAWN_VELOCITY,
            spawn_cell_x,
            spawn_cell_y,
            cell_size,
            grid_pos,
            grid_width,
            self.grid.height_cells(),
        );

        self.active_piece = Some(new_piece);
    }

    pub fn draw(&mut self, gfx: &mut Graphics, alpha: f32) {
        // Draw UI first so it appears behind the grid and pieces
        
        // Draw grid and pieces on top
        self.grid.draw(gfx, alpha);

        if let Some(ref mut piece) = self.active_piece {
            piece.draw(gfx, alpha);
        }
        
        self.ui.draw(gfx, &self.score_manager);
    }

    /// Get a reference to the score manager for displaying stats
    pub fn score_manager(&self) -> &ScoreManager {
        &self.score_manager
    }

    /// Get a mutable reference to the score manager
    pub fn score_manager_mut(&mut self) -> &mut ScoreManager {
        &mut self.score_manager
    }
}
