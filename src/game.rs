use crate::rendering::draw_commands::{DrawCommand, DrawCommandList};
use crate::game_math::Vec2;
use sdl2::event::Event;

pub const CUBE_SIZE: f32 = 50.0;
pub const MOVE_SPEED: f32 = 200.0; // pixels per second

pub struct Game {
    event_pump: sdl2::EventPump,
    _audio_manager: kira::AudioManager,
    // Game state
    cube_position: Vec2,
    running: bool,
    window_width: u32,
    window_height: u32,
}

impl Game {
    pub fn new(
        event_pump: sdl2::EventPump,
        audio_manager: kira::AudioManager,
        window_width: u32,
        window_height: u32,
        initial_x: f32,
        initial_y: f32,
    ) -> Self {
        log::info!("Game initialized. Cube starting at ({}, {})", initial_x, initial_y);
        log::info!("Window size: {}x{}", window_width, window_height);

        Self {
            event_pump,
            _audio_manager: audio_manager,
            cube_position: Vec2::new(initial_x, initial_y),
            running: true,
            window_width,
            window_height,
        }
    }

    pub fn update(&mut self, delta_time: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Handle events
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    self.running = false;
                }
                Event::MouseButtonDown { x, y, .. } => {
                    // Set cube position to mouse click (top-left corner of cube)
                    // Clamp to keep cube within bounds
                    let new_x = (x as f32 - CUBE_SIZE / 2.0).max(0.0).min(self.window_width as f32 - CUBE_SIZE);
                    let new_y = (y as f32 - CUBE_SIZE / 2.0).max(0.0).min(self.window_height as f32 - CUBE_SIZE);
                    self.cube_position = Vec2::new(new_x, new_y);
                    log::info!("Mouse click at ({}, {}), cube moved to ({:.1}, {:.1})", x, y, new_x, new_y);
                }
                _ => {}
            }
        }

        // Handle keyboard input
        let keyboard_state = self.event_pump.keyboard_state();

        let mut velocity = Vec2::ZERO;
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
            velocity.x -= MOVE_SPEED;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
            velocity.x += MOVE_SPEED;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
            velocity.y -= MOVE_SPEED;
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
            velocity.y += MOVE_SPEED;
        }

        // Update position
        self.cube_position += velocity * delta_time;

        // Keep cube within bounds
        self.cube_position.x = self.cube_position.x.max(0.0).min(self.window_width as f32 - CUBE_SIZE);
        self.cube_position.y = self.cube_position.y.max(0.0).min(self.window_height as f32 - CUBE_SIZE);

        Ok(())
    }

    /// Collect all draw commands for this frame
    pub fn draw(&self, _delta_time: f32) -> DrawCommandList {
        let mut commands = DrawCommandList::new();
        
        // Add cube draw command
        commands.add(DrawCommand::Cube {
            position: self.cube_position,
            size: CUBE_SIZE,
        });
        
        // Future: Add more draw commands here (other objects, sprites, etc.)
        // Example: let cube_id = commands.add(DrawCommand::Cube { ... });
        // Later: commands.remove(cube_id);
        
        commands
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
