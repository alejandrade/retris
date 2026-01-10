use crate::coordinate_system::CoordinateSystem;
use egor::input::{Input, MouseButton};
use egor::math::{Vec2, vec2};
use egor::render::{Color, Graphics};

pub struct TetrisMobileController {
    screen_width: f32,
    screen_height: f32,
    // Quit button (top center, Q)
    quit_button_world_pos: Vec2,
    quit_button_size: f32,
    // Red button (under grid)
    red_button_world_pos: Vec2,
    red_button_size: f32,
    // Button states
    left_held: bool,
    right_held: bool,
    rotate_pressed: bool,
    quit_pressed: bool,
    red_button_pressed: bool,
    // Track if device is touch capable
    is_touch_capable: bool,
}

impl TetrisMobileController {
    // Base constants for UI sizing (normalized to 1048px screen height)
    const BASE_QUIT_BUTTON_SIZE: f32 = 80.0;
    const BASE_RED_BUTTON_SIZE: f32 = 100.0;
    const BASE_BUTTON_BORDER_WIDTH: f32 = 4.0;
    const BASE_PADDING: f32 = 80.0;

    // Helper to calculate scale factor
    fn scale_factor(screen_height: f32) -> f32 {
        (screen_height / 1048.0).max(0.5).min(2.0)
    }

    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let mut controller = Self {
            screen_width,
            screen_height,
            quit_button_world_pos: vec2(0.0, 0.0),
            quit_button_size: 0.0,
            red_button_world_pos: vec2(0.0, 0.0),
            red_button_size: 0.0,
            left_held: false,
            right_held: false,
            rotate_pressed: false,
            quit_pressed: false,
            red_button_pressed: false,
            is_touch_capable: false,
        };
        controller.update_positions();
        controller
    }

    fn update_positions(&mut self) {
        let _coords = CoordinateSystem::with_default_offset(self.screen_width, self.screen_height);
        let half_height = self.screen_height / 2.0;
        let scale = Self::scale_factor(self.screen_height);

        self.quit_button_size = Self::BASE_QUIT_BUTTON_SIZE * scale;
        self.red_button_size = Self::BASE_RED_BUTTON_SIZE * scale;
        let padding = Self::BASE_PADDING * scale;

        // Quit button at top center
        self.quit_button_world_pos =
            vec2(0.0, -half_height + self.quit_button_size / 2.0 + padding);

        // Red button position will be set based on grid position (updated in update method)
        // For now, just initialize it
        self.red_button_world_pos = vec2(0.0, 0.0);
    }

    pub fn update(
        &mut self,
        input: &Input,
        screen_width: f32,
        screen_height: f32,
        piece_world_x: Option<f32>,
        piece_world_pos: Option<Vec2>,
        piece_cell_size: Option<f32>,
        grid_bottom_y: Option<f32>,
    ) {
        // Update touch capability status
        self.is_touch_capable = input.is_touch_capable();

        if (screen_width - self.screen_width).abs() > 0.1
            || (screen_height - self.screen_height).abs() > 0.1
        {
            self.screen_width = screen_width;
            self.screen_height = screen_height;
            self.update_positions();
        }

        // Update red button position to be under the grid
        // In world coordinates: negative Y is up, positive Y is down
        // grid_bottom_y is the bottom edge of the visible grid (positive Y value)
        // We want the button below the grid, so we add spacing
        if let Some(grid_bottom) = grid_bottom_y {
            let scale = Self::scale_factor(self.screen_height);
            let button_spacing = 20.0 * scale; // Space between grid and button
            // Button center should be below grid bottom: grid_bottom + spacing + button_half_size
            self.red_button_world_pos = vec2(
                0.0,
                grid_bottom + button_spacing + self.red_button_size / 2.0,
            );
        }

        // Reset button states
        self.rotate_pressed = false;
        self.quit_pressed = false;
        self.left_held = false;
        self.right_held = false;
        self.red_button_pressed = false;

        let coords = CoordinateSystem::with_default_offset(self.screen_width, self.screen_height);

        // Handle touch input
        let touch_count = input.touch_count();
        if touch_count > 0 {
            let (tx, ty) = input.primary_touch_position();
            let mouse_held = input.mouse_held(MouseButton::Left);
            let mouse_pressed = input.mouse_pressed(MouseButton::Left);
            self.handle_input(
                tx,
                ty,
                mouse_held,
                mouse_pressed,
                &coords,
                piece_world_x,
                piece_world_pos,
                piece_cell_size,
            );
        }

        // Handle mouse input (for testing on desktop)
        let (mx, my) = input.mouse_position();
        let mouse_down = input.mouse_held(MouseButton::Left);
        let mouse_just_pressed = input.mouse_pressed(MouseButton::Left);

        if mouse_down || mouse_just_pressed {
            self.handle_input(
                mx,
                my,
                mouse_down,
                mouse_just_pressed,
                &coords,
                piece_world_x,
                piece_world_pos,
                piece_cell_size,
            );
        }
    }

    fn handle_input(
        &mut self,
        x: f32,
        y: f32,
        held: bool,
        just_pressed: bool,
        coords: &CoordinateSystem,
        piece_world_x: Option<f32>,
        piece_world_pos: Option<Vec2>,
        piece_cell_size: Option<f32>,
    ) {
        // Coordinates from input are already in buffer space (converted by egor library)
        // Convert buffer coordinates to world coordinates for comparison
        let touch_world = coords.screen_to_world(vec2(x, y));

        // Check quit button first (has priority) - use world coordinates
        let quit_half = self.quit_button_size / 2.0;
        if touch_world.x >= self.quit_button_world_pos.x - quit_half
            && touch_world.x <= self.quit_button_world_pos.x + quit_half
            && touch_world.y >= self.quit_button_world_pos.y - quit_half
            && touch_world.y <= self.quit_button_world_pos.y + quit_half
        {
            if just_pressed {
                self.quit_pressed = true;
            }
            return;
        }

        // Check red button (fast drop) - use world coordinates
        let red_half = self.red_button_size / 2.0;
        if touch_world.x >= self.red_button_world_pos.x - red_half
            && touch_world.x <= self.red_button_world_pos.x + red_half
            && touch_world.y >= self.red_button_world_pos.y - red_half
            && touch_world.y <= self.red_button_world_pos.y + red_half
        {
            // Red button: fast drop (held)
            if held {
                self.red_button_pressed = true;
            }
            return;
        }

        // Check if click is on the piece itself (for rotation) - use world coordinates
        if just_pressed {
            if let (Some(piece_pos), Some(cell_size)) = (piece_world_pos, piece_cell_size) {
                // Approximate piece bounds: pieces are typically 2-4 cells wide/tall
                // Use a generous hitbox (4 cells) to make it easier to tap
                let piece_half = cell_size * 2.0;
                if touch_world.x >= piece_pos.x - piece_half
                    && touch_world.x <= piece_pos.x + piece_half
                    && touch_world.y >= piece_pos.y - piece_half
                    && touch_world.y <= piece_pos.y + piece_half
                {
                    self.rotate_pressed = true;
                    return;
                }
            }
        }

        // If piece exists, compare touch position to piece position
        if let Some(piece_x) = piece_world_x {
            if touch_world.x < piece_x {
                // Touch is to the left of piece - move left
                if held {
                    self.left_held = true;
                }
            } else {
                // Touch is to the right of piece - move right
                if held {
                    self.right_held = true;
                }
            }
        } else {
            // No piece - use world center (x=0) as fallback
            if touch_world.x < 0.0 {
                if held {
                    self.left_held = true;
                }
            } else {
                if held {
                    self.right_held = true;
                }
            }
        }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        let coords = CoordinateSystem::with_default_offset(self.screen_width, self.screen_height);

        // Draw quit button (top center)
        self.draw_quit_button(gfx, &coords, self.quit_button_world_pos);

        // Draw red button (under grid)
        self.draw_bottom_button(
            gfx,
            &coords,
            self.red_button_world_pos,
            Color::new([1.0, 0.2, 0.2, 0.4]),
            Color::new([1.0, 0.4, 0.4, 0.6]),
        );
    }

    fn draw_bottom_button(
        &self,
        gfx: &mut Graphics,
        _coords: &CoordinateSystem,
        world_pos: Vec2,
        bg_color: Color,
        border_color: Color,
    ) {
        let size = self.red_button_size;
        let half_size = size / 2.0;
        let border_width = Self::BASE_BUTTON_BORDER_WIDTH * Self::scale_factor(self.screen_height);

        // Use world coordinates for rectangles
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(size, size))
            .color(bg_color);

        // Draw border - use world coordinates
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(size, border_width))
            .color(border_color);
        gfx.rect()
            .at(vec2(
                world_pos.x - half_size,
                world_pos.y + half_size - border_width,
            ))
            .size(vec2(size, border_width))
            .color(border_color);
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(border_width, size))
            .color(border_color);
        gfx.rect()
            .at(vec2(
                world_pos.x + half_size - border_width,
                world_pos.y - half_size,
            ))
            .size(vec2(border_width, size))
            .color(border_color);
    }

    fn draw_quit_button(&self, gfx: &mut Graphics, coords: &CoordinateSystem, world_pos: Vec2) {
        let size = self.quit_button_size;
        let half_size = size / 2.0;
        let border_width = Self::BASE_BUTTON_BORDER_WIDTH * Self::scale_factor(self.screen_height);

        // Button background (semi-transparent red)
        let bg_color = Color::new([0.7, 0.2, 0.2, 0.8]);

        // Use world coordinates for rectangles
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(size, size))
            .color(bg_color);

        // Button border (thicker)
        let border_color = Color::new([0.9, 0.3, 0.3, 1.0]);

        // Draw border - use world coordinates
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(size, border_width))
            .color(border_color);
        gfx.rect()
            .at(vec2(
                world_pos.x - half_size,
                world_pos.y + half_size - border_width,
            ))
            .size(vec2(size, border_width))
            .color(border_color);
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(border_width, size))
            .color(border_color);
        gfx.rect()
            .at(vec2(
                world_pos.x + half_size - border_width,
                world_pos.y - half_size,
            ))
            .size(vec2(border_width, size))
            .color(border_color);

        // Label text "Q" - convert to screen coordinates for text
        let screen_pos = coords.world_to_screen(world_pos);
        let text_size = size * 0.6;
        gfx.text("Q")
            .at(vec2(screen_pos.x, screen_pos.y))
            .size(text_size)
            .color(Color::WHITE);
    }

    // Getters for input states
    pub fn left_held(&self) -> bool {
        self.left_held
    }

    pub fn right_held(&self) -> bool {
        self.right_held
    }

    pub fn rotate_pressed(&self) -> bool {
        self.rotate_pressed
    }

    pub fn quit_pressed(&self) -> bool {
        self.quit_pressed
    }

    pub fn red_button_pressed(&self) -> bool {
        self.red_button_pressed
    }
}
