use crate::coordinate_system::CoordinateSystem;
use crate::retris_colors::*;
use egor::input::{Input, MouseButton};
use egor::math::vec2;
use egor::render::Graphics;

/// Convert window coordinates to buffer coordinates
/// Handles DPR, canvas offset, and CSS-to-buffer scaling
#[cfg(target_arch = "wasm32")]
fn window_to_buffer_coords(
    window_x: f32,
    window_y: f32,
    buffer_width: f32,
    buffer_height: f32,
) -> (f32, f32) {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.query_selector("canvas").unwrap().unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    // Get DPR - input coordinates are in physical pixels, need to convert to CSS pixels
    let dpr = crate::get_device_pixel_ratio();
    let css_x = window_x / dpr;
    let css_y = window_y / dpr;

    let rect = canvas.get_bounding_client_rect();
    let canvas_x = rect.left() as f32;
    let canvas_y = rect.top() as f32;
    let css_width = rect.width() as f32;
    let css_height = rect.height() as f32;

    let canvas_relative_x = css_x - canvas_x;
    let canvas_relative_y = css_y - canvas_y;

    let scale_x = buffer_width / css_width;
    let scale_y = buffer_height / css_height;

    (canvas_relative_x * scale_x, canvas_relative_y * scale_y)
}

#[cfg(not(target_arch = "wasm32"))]
fn window_to_buffer_coords(
    window_x: f32,
    window_y: f32,
    _buffer_width: f32,
    _buffer_height: f32,
) -> (f32, f32) {
    (window_x, window_y)
}

/// Public version for debug module
#[cfg(target_arch = "wasm32")]
pub fn window_to_buffer_coords_detailed(
    window_x: f32,
    window_y: f32,
    buffer_width: f32,
    buffer_height: f32,
) -> (f32, f32) {
    window_to_buffer_coords(window_x, window_y, buffer_width, buffer_height)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn window_to_buffer_coords_detailed(
    window_x: f32,
    window_y: f32,
    _buffer_width: f32,
    _buffer_height: f32,
) -> (f32, f32) {
    (window_x, window_y)
}

/// Button position in both coordinate systems
pub struct ButtonPosition {
    pub world_x: f32, // For drawing (0,0 = center)
    pub world_y: f32,
    pub screen_x: f32, // For mouse clicks (0,0 = top-left)
    pub screen_y: f32,
    pub size: f32,
}

impl ButtonPosition {
    /// Scale factor based on screen height, clamped to prevent extreme sizes
    fn scale_factor(screen_height: f32) -> f32 {
        (screen_height / 1048.0).clamp(0.5, 2.0)
    }

    /// Base size for mute button (normalized to 1048px height)
    const BASE_SIZE: f32 = 50.0;
    /// Base padding for mute button (normalized to 1048px height)
    const BASE_PADDING: f32 = 15.0;

    /// Update screen positions based on actual screen dimensions
    pub fn update_screen_pos(&mut self, screen_width: f32, screen_height: f32) {
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let world_pos = vec2(self.world_x, self.world_y);
        let screen_pos = coords.world_to_screen(world_pos);
        self.screen_x = screen_pos.x;
        self.screen_y = screen_pos.y;
    }

    /// Create position for bottom-right corner (screen size will be set later)
    pub fn for_bottom_right(screen_width: f32, screen_height: f32) -> Self {
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let scale = Self::scale_factor(screen_height);
        let size = Self::BASE_SIZE * scale;
        let padding = Self::BASE_PADDING * scale;
        // Position relative to actual screen width, not playing field width
        let world_x = screen_width / 2.0 - size - padding;
        let world_y = screen_height / 2.0 - size - padding;
        let world_pos = vec2(world_x, world_y);
        let screen_pos = coords.world_to_screen(world_pos);

        Self {
            world_x,
            world_y,
            screen_x: screen_pos.x,
            screen_y: screen_pos.y,
            size,
        }
    }

    /// Create position for bottom-left corner (screen size will be set later)
    pub fn for_bottom_left(screen_width: f32, screen_height: f32) -> Self {
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let scale = Self::scale_factor(screen_height);
        let size = Self::BASE_SIZE * scale;
        let padding = Self::BASE_PADDING * scale;
        // Bottom left: negative world_x, positive world_y
        // Position relative to actual screen width, not playing field width
        let world_x = -screen_width / 2.0 + padding;
        let world_y = screen_height / 2.0 - size - padding;
        let world_pos = vec2(world_x, world_y);
        let screen_pos = coords.world_to_screen(world_pos);

        Self {
            world_x,
            world_y,
            screen_x: screen_pos.x,
            screen_y: screen_pos.y,
            size,
        }
    }
}

/// Simple mute button that displays a speaker icon
pub struct MuteButton {
    pos: ButtonPosition,
    is_bottom_right: bool, // Track which corner this button is for
    is_muted: bool,
    speaker_on_texture: Option<usize>,
    speaker_off_texture: Option<usize>,
}

impl MuteButton {
    /// Create button for bottom-right corner (screen size will be updated in draw)
    pub fn for_bottom_right() -> Self {
        // Initialize with default dimensions, will be updated in draw
        let default_width = 640.0;
        let default_height = 1048.0;
        Self {
            pos: ButtonPosition::for_bottom_right(default_width, default_height),
            is_bottom_right: true,
            is_muted: false,
            speaker_on_texture: None,
            speaker_off_texture: None,
        }
    }

    /// Create button for bottom-left corner (screen size will be updated in draw)
    pub fn for_bottom_left() -> Self {
        // Initialize with default dimensions, will be updated in draw
        let default_width = 640.0;
        let default_height = 1048.0;
        Self {
            pos: ButtonPosition::for_bottom_left(default_width, default_height),
            is_bottom_right: false,
            is_muted: false,
            speaker_on_texture: None,
            speaker_off_texture: None,
        }
    }

    /// Load textures on first frame
    pub fn load_textures(&mut self, gfx: &mut Graphics) {
        if self.speaker_on_texture.is_none() {
            self.speaker_on_texture =
                Some(gfx.load_texture(include_bytes!("../assets/speaker.png")));
        }
        if self.speaker_off_texture.is_none() {
            self.speaker_off_texture =
                Some(gfx.load_texture(include_bytes!("../assets/speaker-off.png")));
        }
    }

    /// Update button position based on actual screen dimensions
    pub fn update(&mut self, gfx: &mut Graphics) {
        let screen = gfx.screen_size();
        let screen_width = screen.x;
        let screen_height = screen.y;
        let scale = ButtonPosition::scale_factor(screen_height);
        let size = ButtonPosition::BASE_SIZE * scale;
        let padding = ButtonPosition::BASE_PADDING * scale;

        // Recalculate world position based on which corner (using actual screen width)
        if self.is_bottom_right {
            // Position relative to actual screen width, not playing field width
            self.pos.world_x = screen_width / 2.0 - size - padding;
            self.pos.world_y = screen_height / 2.0 - size - padding;
        } else {
            // Bottom-left
            // Position relative to actual screen width, not playing field width
            self.pos.world_x = -screen_width / 2.0 + padding;
            self.pos.world_y = screen_height / 2.0 - size - padding;
        }
        self.pos.size = size;
        // Update screen position based on new world position
        self.pos.update_screen_pos(screen_width, screen_height);

        // Debug log first update
        static mut LOGGED: bool = false;
        unsafe {
            if !LOGGED {
                LOGGED = true;
                crate::log!(
                    "📍 BTN POS: world=({:.0},{:.0}) screen=({:.0},{:.0}) size={:.0} corner={}",
                    self.pos.world_x,
                    self.pos.world_y,
                    self.pos.screen_x,
                    self.pos.screen_y,
                    self.pos.size,
                    if self.is_bottom_right { "BR" } else { "BL" }
                );
            }
        }
    }

    /// Check if button was clicked
    pub fn is_clicked(
        &self,
        input: &Input,
        #[allow(unused_variables)] gfx: &egor::render::Graphics,
    ) -> bool {
        if !input.mouse_pressed(egor::input::MouseButton::Left) {
            return false;
        }

        let (mx, my) = input.mouse_position();
        let screen = gfx.screen_size();
        let (buffer_x, buffer_y) = window_to_buffer_coords(mx, my, screen.x, screen.y);

        // Convert buffer coords to world coords for comparison
        let coords = CoordinateSystem::with_default_offset(screen.x, screen.y);
        let click_world = coords.screen_to_world(vec2(buffer_x, buffer_y));

        // Button is drawn at (world_x, world_y) with size, so check if click is in that box
        let hit = click_world.x >= self.pos.world_x
            && click_world.x <= self.pos.world_x + self.pos.size
            && click_world.y >= self.pos.world_y
            && click_world.y <= self.pos.world_y + self.pos.size;

        // Debug log button clicks
        crate::log!(
            "🎯 BTN: buffer=({:.0},{:.0}) world=({:.0},{:.0}) btn_world=({:.0},{:.0}) size={:.0} hit={}",
            buffer_x,
            buffer_y,
            click_world.x,
            click_world.y,
            self.pos.world_x,
            self.pos.world_y,
            self.pos.size,
            hit
        );

        hit
    }

    /// Toggle mute state
    pub fn toggle(&mut self) {
        self.is_muted = !self.is_muted;
    }

    /// Get mute state
    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    /// Draw the button (position should be updated via update() before calling)
    pub fn draw(&self, gfx: &mut Graphics) {
        // Skip if textures not loaded
        if self.speaker_on_texture.is_none() || self.speaker_off_texture.is_none() {
            return;
        }

        let texture_id = if self.is_muted {
            self.speaker_off_texture.unwrap()
        } else {
            self.speaker_on_texture.unwrap()
        };

        // Use world coordinates for drawing (already calculated in update())
        gfx.rect()
            .at(vec2(self.pos.world_x, self.pos.world_y))
            .size(vec2(self.pos.size, self.pos.size))
            .texture(texture_id);
    }
}

/// Volume slider UI component
pub struct VolumeSlider {
    x: f32, // World X (center origin)
    y: f32, // World Y
    width: f32,
    height: f32,
    value: f32, // 0.0 to 1.0
    dragging: bool,
    label: String,
    just_released: bool, // Track if mouse was just released this frame
}

impl VolumeSlider {
    /// Scale factor based on screen height, clamped to prevent extreme sizes
    fn scale_factor(screen_height: f32) -> f32 {
        (screen_height / 1048.0).clamp(0.5, 2.0)
    }

    /// Base height for slider (normalized to 1048px height)
    const BASE_HEIGHT: f32 = 30.0;
    /// Base label Y offset (normalized to 1048px height)
    const BASE_LABEL_Y_OFFSET: f32 = 25.0;
    /// Base percentage X offset (normalized to 1048px height)
    const BASE_PERCENT_X_OFFSET: f32 = 10.0;

    /// Create a new volume slider
    pub fn new(x: f32, y: f32, width: f32, label: &str, initial_value: f32) -> Self {
        Self {
            x,
            y,
            width,
            height: Self::BASE_HEIGHT, // Will be scaled in draw/update
            value: initial_value.clamp(0.0, 1.0),
            dragging: false,
            label: label.to_string(),
            just_released: false,
        }
    }

    /// Set position and size (for aspect-ratio-aware scaling)
    pub fn set_position(&mut self, x: f32, y: f32, width: f32) {
        self.x = x;
        self.y = y;
        self.width = width;
    }

    /// Update slider position based on actual screen dimensions
    /// This should be called before handle_input() to update position for hit testing
    /// Note: Slider position (x, y) is in world coordinates and doesn't need updating,
    /// but this method is included for consistency with other UI elements
    pub fn update(&mut self, _screen_width: f32, screen_height: f32) {
        // Scale height based on screen height
        let scale = Self::scale_factor(screen_height);
        self.height = Self::BASE_HEIGHT * scale;
    }

    /// Handle mouse input for the slider
    /// Returns true if value changed significantly
    /// Note: update() should be called first to ensure position is current
    pub fn handle_input(&mut self, input: &Input, screen_width: f32, screen_height: f32) -> bool {
        let (mx, my) = input.mouse_position();
        let (buffer_x, buffer_y) = window_to_buffer_coords(mx, my, screen_width, screen_height);

        let mouse_pressed = input.mouse_pressed(MouseButton::Left);
        let mouse_released = input.mouse_released(MouseButton::Left);

        // Reset just_released flag from previous frame
        self.just_released = false;

        // Convert buffer coords to world coords for hit testing
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let click_world = coords.screen_to_world(vec2(buffer_x, buffer_y));

        // Check if mouse is over slider (in world coordinates)
        let in_bounds = click_world.x >= self.x
            && click_world.x <= self.x + self.width
            && click_world.y >= self.y
            && click_world.y <= self.y + self.height;

        // Start dragging when clicked on slider
        if in_bounds && mouse_pressed {
            self.dragging = true;
        }

        // Stop dragging when mouse button is released
        if mouse_released && self.dragging {
            self.dragging = false;
            self.just_released = true; // Mark that we just released
        }

        // Update value while dragging
        if self.dragging {
            let relative_x = (click_world.x - self.x).clamp(0.0, self.width);
            let old_value = self.value;
            self.value = relative_x / self.width;
            return (old_value - self.value).abs() > 0.01; // Value changed significantly
        }

        false
    }

    /// Check if the slider was just released this frame
    pub fn was_just_released(&self) -> bool {
        self.just_released
    }

    /// Get current value (0.0 to 1.0)
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Draw the slider (position should be updated via update() before calling)
    pub fn draw(&self, gfx: &mut Graphics, screen_width: f32, screen_height: f32) {
        // Use coordinate system with actual screen dimensions for text positioning
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let scale = Self::scale_factor(screen_height);

        // Draw label above slider
        let label_size = (screen_height * 0.019).max(16.0).min(32.0); // Scaled text size
        let label_y_offset = Self::BASE_LABEL_Y_OFFSET * scale;
        let label_world_pos = vec2(self.x, self.y - label_y_offset);
        let label_screen_pos = coords.world_to_screen(label_world_pos);
        gfx.text(&self.label)
            .at(label_screen_pos)
            .size(label_size)
            .color(COLOR_TEXT_GREEN);

        // Draw slider background (dark)
        gfx.rect()
            .at(vec2(self.x, self.y))
            .size(vec2(self.width, self.height))
            .color(COLOR_CELL_BORDER);

        // Draw slider fill (green)
        let fill_width = self.width * self.value;
        if fill_width > 0.0 {
            gfx.rect()
                .at(vec2(self.x, self.y))
                .size(vec2(fill_width, self.height))
                .color(COLOR_SOFTWARE_GREEN);
        }

        // Draw slider handle
        let handle_size = 10.0 * scale;
        let handle_x = self.x + (self.width * self.value) - handle_size / 2.0;
        let handle_y_offset = 5.0 * scale;
        gfx.rect()
            .at(vec2(handle_x, self.y - handle_y_offset))
            .size(vec2(handle_size, self.height + handle_y_offset * 2.0))
            .color(COLOR_TEXT_GREEN);

        // Draw percentage text
        let percent = (self.value * 100.0) as i32;
        let percent_text = format!("{}%", percent);
        let percent_size = (screen_height * 0.017).max(14.0).min(28.0); // Scaled text size
        let percent_x_offset = Self::BASE_PERCENT_X_OFFSET * scale;
        let percent_y_offset = 5.0 * scale;
        let percent_world_pos = vec2(
            self.x + self.width + percent_x_offset,
            self.y + percent_y_offset,
        );
        let percent_screen_pos = coords.world_to_screen(percent_world_pos);
        gfx.text(&percent_text)
            .at(percent_screen_pos)
            .size(percent_size)
            .color(COLOR_DARK_GRAY);
    }
}

/// Simple button UI component
pub struct Button {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    label: String,
}

impl Button {
    /// Scale factor based on screen height, clamped to prevent extreme sizes
    fn scale_factor(screen_height: f32) -> f32 {
        (screen_height / 1048.0).clamp(0.5, 2.0)
    }

    /// Base border width (normalized to 1048px height)
    const BASE_BORDER: f32 = 3.0;

    pub fn new(x: f32, y: f32, width: f32, height: f32, label: &str) -> Self {
        Self {
            x,
            y,
            width,
            height,
            label: label.to_string(),
        }
    }

    /// Set position and size (for aspect-ratio-aware scaling)
    pub fn set_position(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }

    /// Update button position based on actual screen dimensions
    /// Currently buttons are positioned in world coordinates at creation, so this is a no-op
    /// but included for consistency with other UI elements
    pub fn update(&mut self, _screen_width: f32, _screen_height: f32) {
        // Button position (x, y) is set at creation in world coordinates
        // If we need to recalculate position based on screen size, we'd do it here
    }

    /// Check if button was clicked (position should be updated via update() before calling)
    pub fn is_clicked(&self, input: &Input, screen_width: f32, screen_height: f32) -> bool {
        if !input.mouse_pressed(MouseButton::Left) {
            return false;
        }

        let (mx, my) = input.mouse_position();
        let (buffer_x, buffer_y) = window_to_buffer_coords(mx, my, screen_width, screen_height);

        // Convert buffer coords to world coords for comparison
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let click_world = coords.screen_to_world(vec2(buffer_x, buffer_y));

        // Button is drawn at (x, y) with size, check if click is in that box
        let hit = click_world.x >= self.x
            && click_world.x <= self.x + self.width
            && click_world.y >= self.y
            && click_world.y <= self.y + self.height;

        // Debug log
        crate::log!(
            "🎯 BUTTON: buffer=({:.0},{:.0}) world=({:.0},{:.0}) btn=({:.0},{:.0}) size=({:.0}×{:.0}) label='{}' hit={}",
            buffer_x,
            buffer_y,
            click_world.x,
            click_world.y,
            self.x,
            self.y,
            self.width,
            self.height,
            self.label,
            hit
        );

        hit
    }

    /// Draw the button (position should be updated via update() before calling)
    pub fn draw(&self, gfx: &mut Graphics, screen_width: f32, screen_height: f32) {
        // Use coordinate system with actual screen dimensions for text positioning
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let scale = Self::scale_factor(screen_height);

        // Draw button background
        gfx.rect()
            .at(vec2(self.x, self.y))
            .size(vec2(self.width, self.height))
            .color(COLOR_SOFTWARE_GREEN);

        // Draw button border
        let border = Self::BASE_BORDER * scale;
        gfx.rect()
            .at(vec2(self.x - border, self.y - border))
            .size(vec2(self.width + border * 2.0, self.height + border * 2.0))
            .color(COLOR_TEXT_GREEN);

        // Draw button background again (on top of border)
        gfx.rect()
            .at(vec2(self.x, self.y))
            .size(vec2(self.width, self.height))
            .color(COLOR_SOFTWARE_GREEN);

        // Draw label text
        let label_size = (screen_height * 0.023).max(18.0).min(40.0); // Scaled text size
        let estimated_width = self.label.len() as f32 * label_size * 0.5;
        let label_world_pos = vec2(
            self.x + (self.width - estimated_width) / 2.0,
            self.y + (self.height - label_size) / 2.0,
        );
        let label_screen_pos = coords.world_to_screen(label_world_pos);
        gfx.text(&self.label)
            .at(label_screen_pos)
            .size(label_size)
            .color(COLOR_CELL_BORDER);
    }
}
