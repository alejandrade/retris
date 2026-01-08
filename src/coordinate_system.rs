use egor::math::{vec2, Vec2};

/// Coordinate system utility for converting between world coordinates (center origin)
/// and screen coordinates (top-left origin).
/// 
/// World coordinates: (0, 0) is at the center of the screen
/// Screen coordinates: (0, 0) is at the top-left of the screen
pub struct CoordinateSystem {
    screen_width: f32,
    screen_height: f32,
    top_left_offset: Vec2, // Offset for top-left position (usually (0, 0) but can be adjusted)
}

impl CoordinateSystem {
    /// Create a new coordinate system
    /// 
    /// # Arguments
    /// * `screen_width` - Width of the screen in pixels
    /// * `screen_height` - Height of the screen in pixels
    /// * `top_left_offset` - Offset for the top-left position (defaults to (0, 0))
    pub fn new(screen_width: f32, screen_height: f32, top_left_offset: Vec2) -> Self {
        Self {
            screen_width,
            screen_height,
            top_left_offset,
        }
    }

    /// Create a coordinate system with default top-left offset (0, 0)
    pub fn with_default_offset(screen_width: f32, screen_height: f32) -> Self {
        Self::new(screen_width, screen_height, vec2(0.0, 0.0))
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        vec2(
            world_pos.x + self.screen_width / 2.0 + self.top_left_offset.x,
            world_pos.y + self.screen_height / 2.0 + self.top_left_offset.y,
        )
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        vec2(
            screen_pos.x - self.screen_width / 2.0 - self.top_left_offset.x,
            screen_pos.y - self.screen_height / 2.0 - self.top_left_offset.y,
        )
    }

    /// Convert world X to screen X
    pub fn world_x_to_screen_x(&self, world_x: f32) -> f32 {
        world_x + self.screen_width / 2.0 + self.top_left_offset.x
    }

    /// Convert world Y to screen Y
    pub fn world_y_to_screen_y(&self, world_y: f32) -> f32 {
        world_y + self.screen_height / 2.0 + self.top_left_offset.y
    }

    /// Convert screen X to world X
    pub fn screen_x_to_world_x(&self, screen_x: f32) -> f32 {
        screen_x - self.screen_width / 2.0 - self.top_left_offset.x
    }

    /// Convert screen Y to world Y
    pub fn screen_y_to_world_y(&self, screen_y: f32) -> f32 {
        screen_y - self.screen_height / 2.0 - self.top_left_offset.y
    }

    /// Get the world position of the top-left corner of the screen
    pub fn top_left_world(&self) -> Vec2 {
        vec2(
            -self.screen_width / 2.0 - self.top_left_offset.x,
            -self.screen_height / 2.0 - self.top_left_offset.y,
        )
    }

    /// Get the world position of the bottom-right corner of the screen
    pub fn bottom_right_world(&self) -> Vec2 {
        vec2(
            self.screen_width / 2.0 - self.top_left_offset.x,
            self.screen_height / 2.0 - self.top_left_offset.y,
        )
    }

    /// Calculate the world X position to center text horizontally
    /// 
    /// # Arguments
    /// * `text` - The text to center
    /// * `font_size` - The font size in pixels
    /// * `chars_per_pixel` - Estimated characters per pixel (default: 0.5)
    pub fn center_text_x(&self, text: &str, font_size: f32, chars_per_pixel: f32) -> f32 {
        let estimated_width = text.len() as f32 * font_size * chars_per_pixel;
        -estimated_width / 2.0
    }

    /// Get screen width
    pub fn screen_width(&self) -> f32 {
        self.screen_width
    }

    /// Get screen height
    pub fn screen_height(&self) -> f32 {
        self.screen_height
    }
}
