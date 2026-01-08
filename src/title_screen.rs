use crate::retris_colors::*;
use crate::tetris_shape::{ShapeName, TetrisShapeNode};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use egor::input::{Input, KeyCode};
use egor::math::vec2;
use egor::render::Graphics;


/// Target Y position in screen coordinates (middle of screen)
const TARGET_Y: f32 = 0.0; // Screen center is at 0,0

/// Spacing between letters (in cells)
const LETTER_SPACING: f32 = 1.0;

/// Get a color from the piece colors array by index
fn get_piece_color(index: usize) -> egor::render::Color {
    PIECE_COLORS[index % PIECE_COLORS.len()]
}

pub struct TitleScreen {
    letters: Vec<TetrisShapeNode>,
    horizontal_offset: f32,      // Current horizontal offset for bounce animation
    horizontal_velocity: f32,    // Velocity for bounce back
    vertical_offset: f32,        // Current vertical offset for drop/bounce animation
    vertical_velocity: f32,      // Velocity for vertical bounce
    rotation_angle: f32,         // Current rotation angle (in radians)
    rotation_velocity: f32,      // Rotation velocity for spin animation
    float_timer: f32,            // Timer for floating animation
    high_score: u64,             // High score to display
}

impl TitleScreen {
    pub fn new() -> Self {
        use crate::storage::Storage;
        
        let game_data = Storage::load_game_data();

        // Calculate appropriate cell size based on screen width
        // We have 6 letters, each ~3 cells wide + spacing between them
        const NUM_LETTERS: f32 = 6.0;
        const LETTER_WIDTH_CELLS: f32 = 3.0; // Each letter is 3 cells wide
        const PADDING: f32 = 80.0; // Leave padding on sides

        // Calculate cell size to fit all letters on screen
        let available_width = SCREEN_WIDTH as f32 - PADDING;
        let total_cells =
            (NUM_LETTERS * LETTER_WIDTH_CELLS) + ((NUM_LETTERS - 1.0) * LETTER_SPACING);
        let cell_size = available_width / total_cells;

        // Each letter is roughly 3 cells wide, 5 cells tall
        let letter_width = LETTER_WIDTH_CELLS * cell_size;
        let letter_spacing = LETTER_SPACING * cell_size;

        // Create letter shapes for "RETRIS" with colors from our palette
        let letter_shapes = vec![
            (ShapeName::new_letter_r(), get_piece_color(0)), // Cyan
            (ShapeName::new_letter_e(), get_piece_color(1)), // Yellow
            (ShapeName::new_letter_t(), get_piece_color(2)), // Magenta
            (ShapeName::new_letter_r(), get_piece_color(3)), // Orange
            (ShapeName::new_letter_i(), get_piece_color(4)), // Green
            (ShapeName::new_letter_s(), get_piece_color(0)), // Cyan
        ];

        let num_letters = letter_shapes.len() as f32;
        let total_width = (num_letters * letter_width) + ((num_letters - 1.0) * letter_spacing);

        // Position letters in the center of the screen (0,0 is screen center)
        let start_x = -total_width / 2.0 + letter_width / 2.0;
        let start_y = TARGET_Y; // Center vertically

        // Create a small virtual grid just for the shape positioning system
        let grid_width_cells = 200;
        let grid_height_cells = 100;
        
        // Position grid so letters appear at center
        let grid_position = vec2(start_x - (100.0 * cell_size), start_y);

        let mut letters = Vec::new();
        let mut current_x_world = start_x;

        for (shape_name, color) in letter_shapes {
            // Convert world X to grid cell X
            let cell_x = ((current_x_world - grid_position.x) / cell_size) as i32;
            let cell_y = -2;

            let mut letter = TetrisShapeNode::new_with_shape_and_color(
                0,
                cell_x,
                cell_y,
                cell_size,
                grid_position,
                grid_width_cells,
                grid_height_cells,
                shape_name,
                color,
            );
            
            // Mark as already stopped (no falling animation)
            letter.stopped = true;
            
            letters.push(letter);

            // Move to next letter position
            current_x_world += letter_width + letter_spacing;
        }

        Self {
            letters,
            horizontal_offset: 0.0,
            horizontal_velocity: 0.0,
            vertical_offset: 0.0,
            vertical_velocity: 0.0,
            rotation_angle: 0.0,
            rotation_velocity: 0.0,
            float_timer: 0.0,
            high_score: game_data.high_score,
        }
    }

    pub fn update(&mut self, input: &Input, fixed_delta: f32) {
        // Update floating animation timer
        self.float_timer += fixed_delta;
        
        // Handle interactive controls
        
        // Arrow Left: bounce left
        if input.key_pressed(KeyCode::ArrowLeft) {
            self.horizontal_offset = -30.0; // Shift left
            self.horizontal_velocity = 0.0;
        }
        
        // Arrow Right: bounce right
        if input.key_pressed(KeyCode::ArrowRight) {
            self.horizontal_offset = 30.0; // Shift right
            self.horizontal_velocity = 0.0;
        }
        
        // Space: spin
        if input.key_pressed(KeyCode::Space) {
            self.rotation_velocity += std::f32::consts::TAU * 2.0; // Add one full rotation
        }
        
        // Arrow Down: drop and bounce
        if input.key_pressed(KeyCode::ArrowDown) {
            self.vertical_offset = 50.0; // Drop down
            self.vertical_velocity = 0.0;
        }
        
        // Update horizontal bounce-back animation
        const SPRING_STIFFNESS: f32 = 200.0; // How quickly it springs back
        const DAMPING: f32 = 10.0; // Damping to prevent infinite oscillation
        
        let spring_force = -self.horizontal_offset * SPRING_STIFFNESS * fixed_delta;
        let damping_force = -self.horizontal_velocity * DAMPING * fixed_delta;
        
        self.horizontal_velocity += spring_force + damping_force;
        self.horizontal_offset += self.horizontal_velocity * fixed_delta;
        
        // Stop small oscillations
        if self.horizontal_offset.abs() < 0.1 && self.horizontal_velocity.abs() < 1.0 {
            self.horizontal_offset = 0.0;
            self.horizontal_velocity = 0.0;
        }
        
        // Update vertical bounce-back animation
        let vertical_spring_force = -self.vertical_offset * SPRING_STIFFNESS * fixed_delta;
        let vertical_damping_force = -self.vertical_velocity * DAMPING * fixed_delta;
        
        self.vertical_velocity += vertical_spring_force + vertical_damping_force;
        self.vertical_offset += self.vertical_velocity * fixed_delta;
        
        // Stop small oscillations
        if self.vertical_offset.abs() < 0.1 && self.vertical_velocity.abs() < 1.0 {
            self.vertical_offset = 0.0;
            self.vertical_velocity = 0.0;
        }
        
        // Update rotation animation
        const ROTATION_DAMPING: f32 = 5.0;
        self.rotation_angle += self.rotation_velocity * fixed_delta;
        self.rotation_velocity *= 1.0 - (ROTATION_DAMPING * fixed_delta);
        
        // Normalize rotation angle to prevent overflow
        if self.rotation_angle.abs() > std::f32::consts::TAU * 10.0 {
            self.rotation_angle = self.rotation_angle % std::f32::consts::TAU;
        }
        
        // Stop small rotations
        if self.rotation_velocity.abs() < 0.1 {
            self.rotation_velocity = 0.0;
            self.rotation_angle = 0.0; // Reset to upright
        }
    }

    pub fn draw(&mut self, gfx: &mut Graphics, _alpha: f32) {
        // Store transformation values to avoid borrowing issues
        let horizontal_offset = self.horizontal_offset;
        let vertical_offset = self.vertical_offset;
        let rotation_angle = self.rotation_angle;
        let float_timer = self.float_timer;
        
        // Draw letters with position offsets applied
        for letter in self.letters.iter_mut() {
            TitleScreen::draw_letter_with_transform(
                letter,
                gfx,
                horizontal_offset,
                vertical_offset,
                rotation_angle,
                float_timer,
            );
        }
        
        // Draw high score above instructions
        if self.high_score > 0 {
            let text = format!("Your highest score: {}", self.high_score);
            let text_size = 28.0;
            
            // Estimate text width for centering
            let chars_per_pixel = 0.5;
            let estimated_width = text.len() as f32 * text_size * chars_per_pixel;
            
            // Position below the title (in world coordinates)
            let world_x = -estimated_width / 2.0;
            let world_y = TARGET_Y + 100.0;
            
            // Convert to screen coordinates
            let screen_x = world_x + (SCREEN_WIDTH as f32 / 2.0);
            let screen_y = world_y + (SCREEN_HEIGHT as f32 / 2.0);
            
            gfx.text(&text)
                .at(vec2(screen_x, screen_y))
                .size(text_size)
                .color(COLOR_TEXT_GREEN);
        }

        // Draw instructions in green text below the title
        // Since (0,0) is the center of the screen, position text relative to center
        let instructions_y = TARGET_Y + 150.0;

        // Calculate text size based on screen height (roughly 2.5% of screen height)
        let text_size = (SCREEN_HEIGHT as f32 * 0.018).max(14.0).min(24.0);

        let instructions = [
            "Arrow Left/Right: Move",
            "Arrow Down: Speed Up",
            "Space: Rotate",
            "",
            "Press Enter to Start",
        ];

        let line_height = 35.0;
        let start_y = instructions_y;

        for (i, line) in instructions.iter().enumerate() {
            if line.is_empty() {
                continue;
            }

            // Text API uses screen-space coordinates (0,0 at top-left)
            // But our game uses world-space coordinates (0,0 at center)
            // Convert from world-space to screen-space:
            // screen_x = world_x + SCREEN_WIDTH/2
            // screen_y = world_y + SCREEN_HEIGHT/2

            let chars_per_pixel = 0.5; // Estimate: each character is ~0.5 * font_size wide
            let estimated_width = line.len() as f32 * text_size * chars_per_pixel;

            // Calculate world-space position (centered at x=0)
            let world_x = -estimated_width / 2.0;
            let world_y = start_y + i as f32 * line_height;

            // Convert to screen-space coordinates
            let screen_x = world_x + (SCREEN_WIDTH as f32 / 2.0);
            let screen_y = world_y + (SCREEN_HEIGHT as f32 / 2.0);

            gfx.text(line)
                .at(vec2(screen_x, screen_y))
                .size(text_size)
                .color(COLOR_TEXT_GREEN);
        }
    }
    
    fn draw_letter_with_transform(
        letter: &mut TetrisShapeNode,
        gfx: &mut Graphics,
        horizontal_offset: f32,
        vertical_offset: f32,
        rotation_angle: f32,
        float_timer: f32,
    ) {
        const BORDER_WIDTH: f32 = 1.0;

        // Get the world position of the letter (this is the letter's center)
        let world_pos = letter.world_position();
        
        // Apply transformations
        let transformed_pos = vec2(
            world_pos.x + horizontal_offset,
            world_pos.y + vertical_offset
        );
        
        // Add floating animation
        let mut center_pos = transformed_pos;
        const FLOAT_AMPLITUDE: f32 = 8.0;
        const FLOAT_SPEED: f32 = 1.5;
        let float_offset = (float_timer * FLOAT_SPEED * std::f32::consts::TAU).sin() * FLOAT_AMPLITUDE;
        center_pos.y += float_offset;

        // Draw each block of the letter with rotation
        for dimension in letter.shape_name.get_dimensions() {
            // Get the block position relative to the letter's center
            let relative_pos = dimension.position * letter.cell_size;
            
            // Apply rotation around the center
            let rotated_pos = if rotation_angle.abs() > 0.01 {
                let cos_angle = rotation_angle.cos();
                let sin_angle = rotation_angle.sin();
                vec2(
                    relative_pos.x * cos_angle - relative_pos.y * sin_angle,
                    relative_pos.x * sin_angle + relative_pos.y * cos_angle,
                )
            } else {
                relative_pos
            };
            
            // Final position = center + rotated offset
            let block_world_pos = center_pos + rotated_pos;

            let fill_size = vec2(
                letter.cell_size - BORDER_WIDTH * 2.0,
                letter.cell_size - BORDER_WIDTH * 2.0,
            );
            let fill_pos = block_world_pos + vec2(BORDER_WIDTH, BORDER_WIDTH);
            gfx.rect().size(fill_size).at(fill_pos).color(letter.color);
        }
    }
}
