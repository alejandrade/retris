use crate::coordinate_system::CoordinateSystem;
use crate::retris_colors::*;
use egor::input::{Input, MouseButton};
use egor::math::vec2;
use egor::render::Graphics;

/// Button position in both coordinate systems
pub struct ButtonPosition {
    pub world_x: f32,    // For drawing (0,0 = center)
    pub world_y: f32,
    pub screen_x: f32,   // For mouse clicks (0,0 = top-left)
    pub screen_y: f32,
    pub size: f32,
}

impl ButtonPosition {
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
        let size = 50.0;
        let padding = 15.0;
        let world_x = coords.playing_field_width() / 2.0 - size - padding;
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
        let size = 50.0;
        let padding = 15.0;
        // Bottom left: negative world_x, positive world_y
        let world_x = -coords.playing_field_width() / 2.0 + padding;
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
            self.speaker_on_texture = Some(gfx.load_texture(include_bytes!("../assets/speaker.png")));
        }
        if self.speaker_off_texture.is_none() {
            self.speaker_off_texture = Some(gfx.load_texture(include_bytes!("../assets/speaker-off.png")));
        }
    }
    
    /// Update button position based on actual screen dimensions
    pub fn update(&mut self, gfx: &mut Graphics) {
        let screen = gfx.screen_size();
        let screen_width = screen.x;
        let screen_height = screen.y;
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let size = 50.0;
        let padding = 15.0;
        
        // Recalculate world position based on which corner (using playing field width)
        if self.is_bottom_right {
            self.pos.world_x = coords.playing_field_width() / 2.0 - size - padding;
            self.pos.world_y = screen_height / 2.0 - size - padding;
        } else {
            // Bottom-left
            self.pos.world_x = -coords.playing_field_width() / 2.0 + padding;
            self.pos.world_y = screen_height / 2.0 - size - padding;
        }
        // Update screen position based on new world position
        self.pos.update_screen_pos(screen_width, screen_height);
    }
    
    /// Check if button was clicked
    pub fn is_clicked(&self, input: &Input) -> bool {
        if !input.mouse_pressed(egor::input::MouseButton::Left) {
            return false;
        }
        
        let (mx, my) = input.mouse_position();
        
        // Use screen coordinates for mouse comparison
        mx >= self.pos.screen_x && mx <= self.pos.screen_x + self.pos.size && 
        my >= self.pos.screen_y && my <= self.pos.screen_y + self.pos.size
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
    x: f32,           // World X (center origin)
    y: f32,           // World Y
    width: f32,
    height: f32,
    value: f32,       // 0.0 to 1.0
    dragging: bool,
    label: String,
    just_released: bool, // Track if mouse was just released this frame
}

impl VolumeSlider {
    /// Create a new volume slider
    pub fn new(x: f32, y: f32, width: f32, label: &str, initial_value: f32) -> Self {
        Self {
            x,
            y,
            width,
            height: 30.0,
            value: initial_value.clamp(0.0, 1.0),
            dragging: false,
            label: label.to_string(),
            just_released: false,
        }
    }
    
    /// Update slider position based on actual screen dimensions
    /// This should be called before handle_input() to update position for hit testing
    /// Note: Slider position (x, y) is in world coordinates and doesn't need updating,
    /// but this method is included for consistency with other UI elements
    pub fn update(&mut self, _screen_width: f32, _screen_height: f32) {
        // Position doesn't change - slider position is in world coordinates
        // But we might want to recalculate screen position for hit testing if needed
        // For now, slider position (x, y) is set at creation and doesn't need updating
    }
    
    /// Handle mouse input for the slider
    /// Returns true if value changed significantly
    /// Note: update() should be called first to ensure position is current
    pub fn handle_input(&mut self, input: &Input, screen_width: f32, screen_height: f32) -> bool {
        let (mx, my) = input.mouse_position();
        let mouse_pressed = input.mouse_pressed(MouseButton::Left);
        let mouse_released = input.mouse_released(MouseButton::Left);
        
        // Reset just_released flag from previous frame
        self.just_released = false;
        
        // Convert to screen coords for hit testing using actual screen dimensions
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let screen_pos = coords.world_to_screen(vec2(self.x, self.y));
        let screen_x = screen_pos.x;
        let screen_y = screen_pos.y;
        
        // Check if mouse is over slider
        let in_bounds = mx >= screen_x && mx <= screen_x + self.width &&
                       my >= screen_y && my <= screen_y + self.height;
        
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
            let relative_x = (mx - screen_x).clamp(0.0, self.width);
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
        
        // Draw label above slider
        let label_size = 20.0;
        let label_world_pos = vec2(self.x, self.y - 25.0);
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
        let handle_x = self.x + (self.width * self.value) - 5.0;
        gfx.rect()
            .at(vec2(handle_x, self.y - 5.0))
            .size(vec2(10.0, self.height + 10.0))
            .color(COLOR_TEXT_GREEN);
        
        // Draw percentage text
        let percent = (self.value * 100.0) as i32;
        let percent_text = format!("{}%", percent);
        let percent_world_pos = vec2(self.x + self.width + 10.0, self.y + 5.0);
        let percent_screen_pos = coords.world_to_screen(percent_world_pos);
        gfx.text(&percent_text)
            .at(percent_screen_pos)
            .size(18.0)
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
    pub fn new(x: f32, y: f32, width: f32, height: f32, label: &str) -> Self {
        Self {
            x,
            y,
            width,
            height,
            label: label.to_string(),
        }
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
        // Use coordinate system with actual screen dimensions for hit testing
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        let screen_pos = coords.world_to_screen(vec2(self.x, self.y));
        let screen_x = screen_pos.x;
        let screen_y = screen_pos.y;
        
        mx >= screen_x && mx <= screen_x + self.width &&
        my >= screen_y && my <= screen_y + self.height
    }
    
    /// Draw the button (position should be updated via update() before calling)
    pub fn draw(&self, gfx: &mut Graphics, screen_width: f32, screen_height: f32) {
        // Use coordinate system with actual screen dimensions for text positioning
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        
        // Draw button background
        gfx.rect()
            .at(vec2(self.x, self.y))
            .size(vec2(self.width, self.height))
            .color(COLOR_SOFTWARE_GREEN);
        
        // Draw button border
        let border = 3.0;
        gfx.rect()
            .at(vec2(self.x - border, self.y - border))
            .size(vec2(self.width + border * 2.0, self.height + border * 2.0))
            .color(COLOR_TEXT_GREEN);
        
        // Draw button background again (on top of border)
        gfx.rect()
            .at(vec2(self.x, self.y))
            .size(vec2(self.width, self.height))
            .color(COLOR_SOFTWARE_GREEN);
        let label_size = 24.0;
        let estimated_width = self.label.len() as f32 * label_size * 0.5;
        let label_world_pos = vec2(
            self.x + (self.width - estimated_width) / 2.0,
            self.y + (self.height - label_size) / 2.0
        );
        let label_screen_pos = coords.world_to_screen(label_world_pos);
        gfx.text(&self.label)
            .at(label_screen_pos)
            .size(label_size)
            .color(COLOR_CELL_BORDER);
    }
}
