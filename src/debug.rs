use crate::coordinate_system::CoordinateSystem;
use crate::retris_colors::COLOR_SOFTWARE_GREEN;
use egor::input::{Input, MouseButton};
use egor::math::vec2;
use egor::render::Graphics;

pub struct DebugOverlay {
    click_hold_timer: Option<f32>,
    click_hold_position: Option<(f32, f32)>, // Screen coordinates (with DPI adjustment)
    debug_squares: Vec<(f32, f32)>,          // List of positions to show squares at
    device_pixel_ratio: f32,
}

impl DebugOverlay {
    const CLICK_HOLD_DURATION: f32 = 2.0; // seconds
    const DEBUG_SQUARE_SIZE: f32 = 50.0;  // Size of debug squares

    pub fn new() -> Self {
        Self {
            click_hold_timer: None,
            click_hold_position: None,
            debug_squares: Vec::new(),
            device_pixel_ratio: 1.0,
        }
    }

    pub fn update(&mut self, input: &Input, delta: f32, _screen_width: f32, _screen_height: f32) {
        // Update device pixel ratio
        #[cfg(target_arch = "wasm32")]
        {
            self.device_pixel_ratio = crate::get_device_pixel_ratio();
        }

        // Check for mouse or touch input
        let is_held = input.mouse_held(MouseButton::Left) || input.touch_count() > 0;
        let (input_x, input_y) = if input.touch_count() > 0 {
            input.primary_touch_position()
        } else {
            input.mouse_position()
        };

        // Apply DPI adjustment
        let adjusted_x = input_x / self.device_pixel_ratio;
        let adjusted_y = input_y / self.device_pixel_ratio;

        if is_held {
            // Check if this is a new press (timer is None)
            if self.click_hold_timer.is_none() {
                // Start tracking
                self.click_hold_timer = Some(0.0);
                self.click_hold_position = Some((adjusted_x, adjusted_y));
            } else {
                // Update timer
                if let Some(ref mut timer) = self.click_hold_timer {
                    *timer += delta;

                    // Check if we've held for 5 seconds
                    if *timer >= Self::CLICK_HOLD_DURATION {
                        // Add this position to debug squares if not already there
                        if let Some(pos) = self.click_hold_position {
                            // Check if this position is already in the list (within a small threshold)
                            let already_exists = self.debug_squares.iter().any(|(x, y)| {
                                (x - pos.0).abs() < 5.0 && (y - pos.1).abs() < 5.0
                            });

                            if !already_exists {
                                self.debug_squares.push(pos);
                                println!("Debug square added at ({}, {})", pos.0, pos.1);
                            }
                        }
                        // Reset timer to allow adding more squares if held longer
                        *timer = 0.0;
                    }
                }
            }
        } else {
            // Not held - reset timer
            self.click_hold_timer = None;
            self.click_hold_position = None;
        }
    }

    pub fn draw(&self, gfx: &mut Graphics, screen_width: f32, screen_height: f32) {
        if self.debug_squares.is_empty() {
            return;
        }

        // Convert screen coordinates to world coordinates for drawing
        let coords = CoordinateSystem::with_default_offset(screen_width, screen_height);
        for (screen_x, screen_y) in &self.debug_squares {
            let world_pos = coords.screen_to_world(vec2(*screen_x, *screen_y));

            // Draw a small green square at this position
            gfx.rect()
                .at(world_pos)
                .size(vec2(Self::DEBUG_SQUARE_SIZE, Self::DEBUG_SQUARE_SIZE))
                .color(COLOR_SOFTWARE_GREEN);
        }
    }

    /// Clear all debug squares
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.debug_squares.clear();
    }
}
