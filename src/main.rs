mod game;
mod game_math;
mod rendering;

use game::Game;
use rendering::Renderer;
use std::time::Instant;

fn init_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
}

// Configuration constants
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const TARGET_FPS: u32 = 60;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    log::info!("Starting game initialization...");
    
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    log::info!("SDL2 initialized");
    let video_subsystem = sdl_context.video()?;

    // Create window
    let window = video_subsystem
        .window("retris", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()?;

    // Initialize Kira audio manager
    let audio_manager = kira::AudioManager::new(kira::AudioManagerSettings::default())?;

    // Initialize renderer (abstracts away all wgpu complexity)
    log::info!("Initializing renderer...");
    let mut renderer = Renderer::new(&window, WINDOW_WIDTH, WINDOW_HEIGHT)?;
    log::info!("Renderer initialized successfully");

    // Calculate initial centered position
    let initial_x = (WINDOW_WIDTH as f32 - game::CUBE_SIZE) / 2.0;
    let initial_y = (WINDOW_HEIGHT as f32 - game::CUBE_SIZE) / 2.0;

    // Event pump
    let event_pump = sdl_context.event_pump()?;

    // Create game with all initialized components
    let mut game = Game::new(
        event_pump,
        audio_manager,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        initial_x,
        initial_y,
    );
    
    log::info!("Entering main game loop...");
    let mut last_time = Instant::now();
    let mut accumulator = 0.0;
    let fixed_dt = 1.0 / TARGET_FPS as f32; // Fixed timestep for physics updates
    
    while game.is_running() {
        let now = Instant::now();
        let frame_time = last_time.elapsed().as_secs_f32();
        last_time = now;
        
        accumulator += frame_time;
        
        // "Catch up" on physics updates in fixed chunks
        // If the game lagged, this loop runs multiple times to catch up
        while accumulator >= fixed_dt {
            game.update(fixed_dt)?;
            accumulator -= fixed_dt;
        }
        
        // Render as fast as possible (or VSync)
        // Use the accumulator for interpolation if needed (alpha = accumulator / fixed_dt)
        let draw_commands = game.draw(accumulator / fixed_dt);
        
        // Begin frame
        let mut frame = renderer.begin()?;
        
        // Draw all commands
        frame.draw_commands(&draw_commands);
        
        // End frame
        frame.end()?;
    }

    Ok(())
}
