use egor::math::{vec2, Vec2};

/// Fixed aspect ratio matching the original screen dimensions (640x1048)
const ASPECT_RATIO: f32 = 640.0 / 1048.0; // width / height

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
    /// Uses actual screen width and height for conversions
    /// 
    /// # Arguments
    /// * `screen_width` - Actual width of the screen in pixels
    /// * `screen_height` - Actual height of the screen in pixels
    pub fn with_default_offset(screen_width: f32, screen_height: f32) -> Self {
        Self::new(screen_width, screen_height, vec2(0.0, 0.0))
    }
    
    pub fn playing_field_width(&self) -> f32 {
        self.screen_height * ASPECT_RATIO
    }
    
    /// Get the playing field height (uses full screen height)
    pub fn playing_field_height(&self) -> f32 {
        self.screen_height
    }
    
    pub fn left_edge_x(&self) -> f32 {
        let playing_width = self.playing_field_width();
        // Center the playing field within the actual screen width
        -playing_width / 2.0
    }
    
    /// Get the right edge X coordinate of the playing field (respecting aspect ratio)
    /// This centers the playing field within the actual screen width
    pub fn right_edge_x(&self) -> f32 {
        let playing_width = self.playing_field_width();
        // Center the playing field within the actual screen width
        playing_width / 2.0
    }
    
    /// Get the top-left position of the playing field (respecting aspect ratio)
    /// Returns a Vec2 with world coordinates (x, y) where:
    /// - x: left edge of the playing field
    /// - y: top of the screen (playing field uses full screen height)
    pub fn playing_field_top_left(&self) -> Vec2 {
        vec2(
            self.left_edge_x(),
            -self.screen_height / 2.0,
        )
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        vec2(
            world_pos.x + self.screen_width / 2.0 + self.top_left_offset.x,
            world_pos.y + self.screen_height / 2.0 + self.top_left_offset.y,
        )
    }

    /// Get the world position of the top-left corner of the screen
    pub fn top_left_world(&self) -> Vec2 {
        vec2(
            -self.screen_width / 2.0 - self.top_left_offset.x,
            -self.screen_height / 2.0 - self.top_left_offset.y,
        )
    }
    
    /// Get the center X coordinate (always 0.0, but included for clarity)
    pub fn center_x(&self) -> f32 {
        0.0
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
