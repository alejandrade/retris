use crate::coordinate_system::CoordinateSystem;
use crate::retris_colors::COLOR_SOFTWARE_GREEN;
use egor::input::{Input, MouseButton};
use egor::math::vec2;
use egor::render::Graphics;

pub struct DebugOverlay {
    click_hold_timer: Option<f32>,
    click_hold_position: Option<(f32, f32)>, // Screen coordinates (with DPI adjustment)
    debug_squares: Vec<(f32, f32, f32)>,      // List of (x, y, dpi) positions to show squares at
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

    pub fn update(&mut self, input: &Input, delta: f32, screen_width: f32, screen_height: f32) {
        // Update device pixel ratio
        #[cfg(target_arch = "wasm32")]
        {
            self.device_pixel_ratio = crate::get_device_pixel_ratio();
        }

        // Check for mouse or touch input
        #[cfg(target_arch = "wasm32")]
        let is_pressed = input.mouse_pressed(MouseButton::Left);
        let is_held = input.mouse_held(MouseButton::Left) || input.touch_count() > 0;
        let (input_x, input_y) = if input.touch_count() > 0 {
            input.primary_touch_position()
        } else {
            input.mouse_position()
        };

        // Calculate adjusted coordinates with detailed logging
        #[cfg(target_arch = "wasm32")]
        let (adjusted_x, adjusted_y) = {
            use wasm_bindgen::JsCast;

            let (buffer_x, buffer_y) = crate::retris_ui::window_to_buffer_coords_detailed(
                input_x, input_y, screen_width, screen_height
            );

            // Log comprehensive debug info on click
            if is_pressed {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let canvas = document.query_selector("canvas").unwrap().unwrap();
                let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
                let rect = canvas.get_bounding_client_rect();
                let dpr = crate::get_device_pixel_ratio();
                let css_x = input_x / dpr;
                let css_y = input_y / dpr;
                let canvas_rel_x = css_x - rect.left() as f32;
                let canvas_rel_y = css_y - rect.top() as f32;

                crate::log!(
                    "🔍 CLICK DEBUG:\n  Physical px: ({:.0}, {:.0})\n  DPR: {:.2}x\n  CSS px: ({:.0}, {:.0})\n  Canvas offset: ({:.0}, {:.0})\n  Canvas-relative: ({:.0}, {:.0})\n  Canvas CSS size: {:.0}×{:.0}\n  Buffer size: {:.0}×{:.0}\n  Final buffer coords: ({:.0}, {:.0})",
                    input_x, input_y,
                    dpr,
                    css_x, css_y,
                    rect.left(), rect.top(),
                    canvas_rel_x, canvas_rel_y,
                    rect.width(), rect.height(),
                    canvas.width(), canvas.height(),
                    buffer_x, buffer_y
                );
            }

            (buffer_x, buffer_y)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (adjusted_x, adjusted_y) = {
            let _ = (screen_width, screen_height); // Acknowledge unused parameters
            (input_x, input_y)
        };

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
                            let already_exists = self.debug_squares.iter().any(|(x, y, _)| {
                                (x - pos.0).abs() < 5.0 && (y - pos.1).abs() < 5.0
                            });

                            if !already_exists {
                                // Store position with current DPI
                                let entry = (pos.0, pos.1, self.device_pixel_ratio);
                                self.debug_squares.push(entry);
                                println!("Debug square added at ({}, {}) with DPI: {:.2}x", pos.0, pos.1, self.device_pixel_ratio);
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
        for (screen_x, screen_y, dpi) in &self.debug_squares {
            let world_pos = coords.screen_to_world(vec2(*screen_x, *screen_y));

            // Draw a small green square at this position
            gfx.rect()
                .at(world_pos)
                .size(vec2(Self::DEBUG_SQUARE_SIZE, Self::DEBUG_SQUARE_SIZE))
                .color(COLOR_SOFTWARE_GREEN);

            // Draw DPI text next to the square (offset to the right and slightly below)
            let text_offset_x = Self::DEBUG_SQUARE_SIZE + 10.0;
            let text_offset_y = Self::DEBUG_SQUARE_SIZE / 2.0;
            let text_world_pos = coords.screen_to_world(vec2(
                *screen_x + text_offset_x,
                *screen_y + text_offset_y,
            ));
            let text_screen_pos = coords.world_to_screen(text_world_pos);
            let dpi_text = format!("DPI: {:.2}x", dpi);
            let text_size = 20.0;

            gfx.text(&dpi_text)
                .at(text_screen_pos)
                .size(text_size)
                .color(COLOR_SOFTWARE_GREEN);
        }
    }

    /// Clear all debug squares
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.debug_squares.clear();
    }
}
