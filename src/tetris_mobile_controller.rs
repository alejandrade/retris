use crate::coordinate_system::CoordinateSystem;
use egor::input::{Input, MouseButton};
use egor::math::{Vec2, vec2};
use egor::render::{Color, Graphics};

pub struct TetrisMobileController {
    screen_width: f32,
    screen_height: f32,
    // D-pad buttons (bottom left)
    left_button_world_pos: Vec2,
    right_button_world_pos: Vec2,
    down_button_world_pos: Vec2,
    // Rotate button (bottom right, circle)
    rotate_button_world_pos: Vec2,
    // Quit button (top center, Q)
    quit_button_world_pos: Vec2,
    // Button states
    left_held: bool,
    right_held: bool,
    down_held: bool,
    rotate_pressed: bool,
    quit_pressed: bool,
    // Touch tracking
    active_touch_id: Option<u64>,
}

impl TetrisMobileController {
    // Constants for UI sizing
    const DPAD_BUTTON_SIZE: f32 = 150.0; // Increased from 60.0
    const DPAD_PADDING: f32 = 80.0; // Increased from 20.0 (distance from edges)
    const DPAD_BUTTON_SPACING: f32 = 5.0; // Spacing between buttons (reduced from implicit spacing)
    const ROTATE_BUTTON_RADIUS: f32 = 75.0; // Increased from 50.0
    const QUIT_BUTTON_SIZE: f32 = 80.0; // Increased from 50.0
    const BUTTON_BORDER_WIDTH: f32 = 4.0; // Increased border width

    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let mut controller = Self {
            screen_width,
            screen_height,
            left_button_world_pos: vec2(0.0, 0.0),
            right_button_world_pos: vec2(0.0, 0.0),
            down_button_world_pos: vec2(0.0, 0.0),
            rotate_button_world_pos: vec2(0.0, 0.0),
            quit_button_world_pos: vec2(0.0, 0.0),
            left_held: false,
            right_held: false,
            down_held: false,
            rotate_pressed: false,
            quit_pressed: false,
            active_touch_id: None,
        };
        controller.update_positions();
        controller
    }

    fn update_positions(&mut self) {
        let coords = CoordinateSystem::with_default_offset(self.screen_width, self.screen_height);
        let half_height = self.screen_height / 2.0;
        let half_width = coords.playing_field_width() / 2.0;

        // Position buttons further up from bottom (reduce vertical offset)
        let bottom_offset = Self::DPAD_PADDING * 0.6; // Move up by reducing offset

        // D-pad buttons at bottom left, all in a row
        let dpad_base_x = -half_width + Self::DPAD_PADDING + 100.0 + Self::DPAD_BUTTON_SIZE / 2.0;
        let dpad_base_y = half_height - bottom_offset - Self::DPAD_BUTTON_SIZE / 2.0;

        // All three buttons in a row: Left, Right, Down
        self.left_button_world_pos = vec2(
            dpad_base_x - Self::DPAD_BUTTON_SIZE - Self::DPAD_BUTTON_SPACING,
            dpad_base_y,
        );

        self.right_button_world_pos = vec2(dpad_base_x, dpad_base_y);

        // Down button to the right of right button
        self.down_button_world_pos = vec2(
            dpad_base_x + Self::DPAD_BUTTON_SIZE + Self::DPAD_BUTTON_SPACING,
            dpad_base_y,
        );

        // Rotate button (circle) at bottom right, further up
        self.rotate_button_world_pos = vec2(
            half_width - Self::DPAD_PADDING - Self::ROTATE_BUTTON_RADIUS,
            half_height - bottom_offset - Self::ROTATE_BUTTON_RADIUS,
        );

        // Quit button at top center
        self.quit_button_world_pos = vec2(
            0.0,
            -half_height + Self::QUIT_BUTTON_SIZE / 2.0 + Self::DPAD_PADDING,
        );
    }

    pub fn update(&mut self, input: &Input, screen_width: f32, screen_height: f32) {
        if (screen_width - self.screen_width).abs() > 0.1
            || (screen_height - self.screen_height).abs() > 0.1
        {
            self.screen_width = screen_width;
            self.screen_height = screen_height;
            self.update_positions();
        }

        // Reset button states
        self.rotate_pressed = false;
        self.quit_pressed = false;

        let coords = CoordinateSystem::with_default_offset(self.screen_width, self.screen_height);

        // Handle touch input
        let touch_count = input.touch_count();
        if touch_count > 0 {
            let (tx, ty) = input.primary_touch_position();
            self.handle_touch(tx, ty, &coords);
        } else {
            self.active_touch_id = None;
            // Reset held states if no touch
            if self.left_held || self.right_held || self.down_held {
                self.left_held = false;
                self.right_held = false;
                self.down_held = false;
            }
        }

        // Handle mouse input (for testing on desktop)
        let (mx, my) = input.mouse_position();
        let mouse_down = input.mouse_held(MouseButton::Left);
        let mouse_just_pressed = input.mouse_pressed(MouseButton::Left);

        if mouse_down || mouse_just_pressed {
            self.handle_mouse(mx, my, mouse_just_pressed, &coords);
        } else {
            // Reset held states if mouse not down
            if self.left_held || self.right_held || self.down_held {
                self.left_held = false;
                self.right_held = false;
                self.down_held = false;
            }
        }
    }

    fn handle_touch(&mut self, tx: f32, ty: f32, coords: &CoordinateSystem) {
        let left_screen = coords.world_to_screen(self.left_button_world_pos);
        let right_screen = coords.world_to_screen(self.right_button_world_pos);
        let down_screen = coords.world_to_screen(self.down_button_world_pos);
        let rotate_screen = coords.world_to_screen(self.rotate_button_world_pos);
        let quit_screen = coords.world_to_screen(self.quit_button_world_pos);

        // Check left button
        if self.is_point_in_square(tx, ty, left_screen, Self::DPAD_BUTTON_SIZE) {
            self.left_held = true;
            return;
        }

        // Check right button
        if self.is_point_in_square(tx, ty, right_screen, Self::DPAD_BUTTON_SIZE) {
            self.right_held = true;
            return;
        }

        // Check down button
        if self.is_point_in_square(tx, ty, down_screen, Self::DPAD_BUTTON_SIZE) {
            self.down_held = true;
            return;
        }

        // Check rotate button (circle)
        if self.is_point_in_circle(tx, ty, rotate_screen, Self::ROTATE_BUTTON_RADIUS) {
            if self.active_touch_id.is_none() {
                self.rotate_pressed = true;
            }
            return;
        }

        // Check quit button
        if self.is_point_in_square(tx, ty, quit_screen, Self::QUIT_BUTTON_SIZE) {
            if self.active_touch_id.is_none() {
                self.quit_pressed = true;
            }
        }
    }

    fn handle_mouse(&mut self, mx: f32, my: f32, just_pressed: bool, coords: &CoordinateSystem) {
        let left_screen = coords.world_to_screen(self.left_button_world_pos);
        let right_screen = coords.world_to_screen(self.right_button_world_pos);
        let down_screen = coords.world_to_screen(self.down_button_world_pos);
        let rotate_screen = coords.world_to_screen(self.rotate_button_world_pos);
        let quit_screen = coords.world_to_screen(self.quit_button_world_pos);

        // Check left button
        if self.is_point_in_square(mx, my, left_screen, Self::DPAD_BUTTON_SIZE) {
            self.left_held = true;
            return;
        }

        // Check right button
        if self.is_point_in_square(mx, my, right_screen, Self::DPAD_BUTTON_SIZE) {
            self.right_held = true;
            return;
        }

        // Check down button
        if self.is_point_in_square(mx, my, down_screen, Self::DPAD_BUTTON_SIZE) {
            self.down_held = true;
            return;
        }

        // Check rotate button (circle)
        if self.is_point_in_circle(mx, my, rotate_screen, Self::ROTATE_BUTTON_RADIUS) {
            if just_pressed {
                self.rotate_pressed = true;
            }
            return;
        }

        // Check quit button
        if self.is_point_in_square(mx, my, quit_screen, Self::QUIT_BUTTON_SIZE) {
            if just_pressed {
                self.quit_pressed = true;
            }
        }
    }

    fn is_point_in_square(&self, px: f32, py: f32, center: Vec2, size: f32) -> bool {
        let half = size / 2.0;
        px >= center.x - half
            && px <= center.x + half
            && py >= center.y - half
            && py <= center.y + half
    }

    fn is_point_in_circle(&self, px: f32, py: f32, center: Vec2, radius: f32) -> bool {
        let dx = px - center.x;
        let dy = py - center.y;
        dx * dx + dy * dy <= radius * radius
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        let coords = CoordinateSystem::with_default_offset(self.screen_width, self.screen_height);

        // Draw D-pad buttons (bottom left)
        self.draw_dpad_button(
            gfx,
            &coords,
            self.left_button_world_pos,
            "<",
            self.left_held,
        );
        self.draw_dpad_button(
            gfx,
            &coords,
            self.right_button_world_pos,
            ">",
            self.right_held,
        );
        self.draw_dpad_button(
            gfx,
            &coords,
            self.down_button_world_pos,
            "v",
            self.down_held,
        );

        // Draw rotate button (circle, bottom right)
        self.draw_circle_button(gfx, &coords, self.rotate_button_world_pos, "O");

        // Draw quit button (top center)
        self.draw_quit_button(gfx, &coords, self.quit_button_world_pos);
    }

    fn draw_dpad_button(
        &self,
        gfx: &mut Graphics,
        coords: &CoordinateSystem,
        world_pos: Vec2,
        label: &str,
        pressed: bool,
    ) {
        let size = Self::DPAD_BUTTON_SIZE;
        let half_size = size / 2.0;

        // Button background (semi-transparent)
        let bg_color = if pressed {
            Color::new([0.3, 0.7, 0.3, 0.8]) // Green when pressed
        } else {
            Color::new([0.2, 0.2, 0.2, 0.7]) // Dark gray when not pressed
        };

        // Use world coordinates for rectangles
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(size, size))
            .color(bg_color);

        // Button border (thicker)
        let border_color = if pressed {
            Color::new([0.5, 1.0, 0.5, 1.0])
        } else {
            Color::new([0.5, 0.5, 0.5, 1.0])
        };

        // Draw border as lines (simple approach: draw 4 rectangles)
        let border_width = Self::BUTTON_BORDER_WIDTH;
        // Top
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(size, border_width))
            .color(border_color);

        // Bottom
        gfx.rect()
            .at(vec2(
                world_pos.x - half_size,
                world_pos.y + half_size - border_width,
            ))
            .size(vec2(size, border_width))
            .color(border_color);
        // Left
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(border_width, size))
            .color(border_color);
        // Right
        gfx.rect()
            .at(vec2(
                world_pos.x + half_size - border_width,
                world_pos.y - half_size,
            ))
            .size(vec2(border_width, size))
            .color(border_color);

        // Label text - convert to screen coordinates for text
        let screen_pos = coords.world_to_screen(world_pos);
        let text_size = size * 0.6;
        gfx.text(label)
            .at(vec2(screen_pos.x, screen_pos.y))
            .size(text_size)
            .color(Color::WHITE);
    }

    fn draw_circle_button(
        &self,
        gfx: &mut Graphics,
        coords: &CoordinateSystem,
        world_pos: Vec2,
        label: &str,
    ) {
        let radius = Self::ROTATE_BUTTON_RADIUS;

        // Button background (semi-transparent dark gray)
        let bg_color = Color::new([0.2, 0.2, 0.2, 0.7]);

        // Draw circle approximation: use a filled rect that covers the circle bounds
        // Use world coordinates for rectangles
        let diameter = radius * 2.0;
        gfx.rect()
            .at(vec2(world_pos.x - radius, world_pos.y - radius))
            .size(vec2(diameter, diameter))
            .color(bg_color);

        // Border (circular approximation - draw as thick square)
        let border_color = Color::new([0.5, 0.5, 0.5, 1.0]);
        let border_width = Self::BUTTON_BORDER_WIDTH;

        // Draw border as 4 rectangles (top, bottom, left, right) - use world coordinates
        // Top
        gfx.rect()
            .at(vec2(world_pos.x - radius, world_pos.y - radius))
            .size(vec2(diameter, border_width))
            .color(border_color);
        // Bottom
        gfx.rect()
            .at(vec2(
                world_pos.x - radius,
                world_pos.y + radius - border_width,
            ))
            .size(vec2(diameter, border_width))
            .color(border_color);
        // Left
        gfx.rect()
            .at(vec2(world_pos.x - radius, world_pos.y - radius))
            .size(vec2(border_width, diameter))
            .color(border_color);
        // Right
        gfx.rect()
            .at(vec2(
                world_pos.x + radius - border_width,
                world_pos.y - radius,
            ))
            .size(vec2(border_width, diameter))
            .color(border_color);

        // Label text - convert to screen coordinates for text
        let screen_pos = coords.world_to_screen(world_pos);
        let text_size = radius * 0.8;
        gfx.text(label)
            .at(vec2(screen_pos.x, screen_pos.y))
            .size(text_size)
            .color(Color::WHITE);
    }

    fn draw_quit_button(&self, gfx: &mut Graphics, coords: &CoordinateSystem, world_pos: Vec2) {
        let size = Self::QUIT_BUTTON_SIZE;
        let half_size = size / 2.0;

        // Button background (semi-transparent red)
        let bg_color = Color::new([0.7, 0.2, 0.2, 0.8]);

        // Use world coordinates for rectangles
        gfx.rect()
            .at(vec2(world_pos.x - half_size, world_pos.y - half_size))
            .size(vec2(size, size))
            .color(bg_color);

        // Button border (thicker)
        let border_color = Color::new([0.9, 0.3, 0.3, 1.0]);
        let border_width = Self::BUTTON_BORDER_WIDTH;

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

    pub fn down_held(&self) -> bool {
        self.down_held
    }

    pub fn rotate_pressed(&self) -> bool {
        self.rotate_pressed
    }

    pub fn quit_pressed(&self) -> bool {
        self.quit_pressed
    }
}
