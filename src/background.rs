use crate::retris_colors::COLOR_BACKGROUND;
use egor::math::vec2;
use egor::render::{Color, Graphics};
use rand::Rng;

struct Star {
    x: f32,
    y: f32,
    size: f32,
    velocity_x: f32,
    velocity_y: f32,
    twinkle_offset: f32, // Phase offset for twinkling
    twinkle_speed: f32,  // How fast it twinkles
}

pub struct Background {
    stars: Vec<Star>,
    elapsed_time: f32,
    screen_width: f32,  // Actual screen width (full canvas)
    screen_height: f32, // Actual screen height (full canvas)
}

impl Background {
    /// Calculate how many stars are needed based on screen area
    /// Uses a density-based approach that scales better with screen size
    fn calculate_star_count(screen_width: f32, screen_height: f32) -> usize {
        // Use density: approximately 1 star per 500 square pixels
        // This gives a nice starfield without being too dense
        const STARS_PER_500_PIXELS: f32 = 1.0;
        
        let screen_area = screen_width * screen_height;
        let star_count = (screen_area / 500.0 * STARS_PER_500_PIXELS).ceil() as usize;

        // Clamp to reasonable bounds: minimum 50, maximum 2000
        let clamped = star_count.clamp(50, 2000);

        // Debug output to see what's happening
        #[cfg(debug_assertions)]
        {
            println!(
                "calculate_star_count: screen={}x{}, area={}, calculated={}, clamped={}",
                screen_width, screen_height, screen_area, star_count, clamped
            );
        }
        
        clamped
    }

    /// Generate a new star at a random position within screen bounds
    fn generate_star(screen_width: f32, screen_height: f32) -> Star {
        let mut rng = rand::rng();
        let half_width = screen_width / 2.0;
        let half_height = screen_height / 2.0;

        Star {
            x: rng.random_range(-half_width..half_width),
            y: rng.random_range(-half_height..half_height),
            size: rng.random_range(1.0..4.0),
            velocity_x: rng.random_range(-10.0..10.0),
            velocity_y: rng.random_range(-10.0..10.0),
            twinkle_offset: rng.random_range(0.0..(std::f32::consts::TAU)),
            twinkle_speed: rng.random_range(0.5..2.0),
        }
    }

    pub fn new(star_count: usize) -> Self {
        // Use default size for initialization (will be updated with actual screen dimensions)
        let default_width = 640.0;
        let default_height = 1048.0;
        let half_width = default_width / 2.0;
        let half_height = default_height / 2.0;

        let mut rng = rand::rng();
        let mut stars = Vec::new();

        for _ in 0..star_count {
            stars.push(Star {
                x: rng.random_range(-half_width..half_width),
                y: rng.random_range(-half_height..half_height),
                size: rng.random_range(1.0..4.0),
                velocity_x: rng.random_range(-10.0..10.0), // Gentle drift speed
                velocity_y: rng.random_range(-10.0..10.0),
                twinkle_offset: rng.random_range(0.0..(std::f32::consts::TAU)),
                twinkle_speed: rng.random_range(0.5..2.0),
            });
        }

        Self {
            stars,
            elapsed_time: 0.0,
            screen_width: default_width,
            screen_height: default_height,
        }
    }

    /// Update screen dimensions (should be called from game loop with actual screen size)
    /// Recalculates and adjusts star count to maintain 30% coverage, then repositions all stars
    pub fn update_screen_size(&mut self, screen_width: f32, screen_height: f32) {
        let old_width = self.screen_width;
        let old_height = self.screen_height;

        self.screen_width = screen_width;
        self.screen_height = screen_height;

        // If screen size changed, recalculate star count and reposition
        if (old_width - screen_width).abs() > 0.1 || (old_height - screen_height).abs() > 0.1 {
            let required_star_count = Self::calculate_star_count(screen_width, screen_height);
            let current_count = self.stars.len();

            // Adjust star count to match required count
            if required_star_count > current_count {
                // Add more stars
                for _ in 0..(required_star_count - current_count) {
                    self.stars
                        .push(Self::generate_star(screen_width, screen_height));
                }
            } else if required_star_count < current_count {
                // Remove excess stars
                self.stars.truncate(required_star_count);
            }

            // Reposition all stars to fill the new screen dimensions
            let half_width = screen_width / 2.0;
            let half_height = screen_height / 2.0;
            let mut rng = rand::rng();

            for star in self.stars.iter_mut() {
                star.x = rng.random_range(-half_width..half_width);
                star.y = rng.random_range(-half_height..half_height);
            }
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.elapsed_time += delta;

        // Use actual screen dimensions for wrapping (full screen, not just playing field)
        let half_width = self.screen_width / 2.0;
        let half_height = self.screen_height / 2.0;

        for star in self.stars.iter_mut() {
            // Update position with gentle drift
            star.x += star.velocity_x * delta;
            star.y += star.velocity_y * delta;

            // Wrap around screen edges (using full screen dimensions)
            if star.x > half_width {
                star.x = -half_width;
            } else if star.x < -half_width {
                star.x = half_width;
            }

            if star.y > half_height {
                star.y = -half_height;
            } else if star.y < -half_height {
                star.y = half_height;
            }
        }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        // Clear with dark gray background
        gfx.clear(COLOR_BACKGROUND);

        // Draw stars with twinkling effect
        for star in &self.stars {
            // Calculate twinkling brightness (0.4 to 1.0 alpha)
            let twinkle =
                ((self.elapsed_time * star.twinkle_speed + star.twinkle_offset).sin() * 0.3 + 0.7)
                    .max(0.4)
                    .min(1.0);

            // Draw a small white cube with varying brightness
            gfx.rect()
                .at(vec2(star.x, star.y))
                .size(vec2(star.size, star.size))
                .color(Color::new([1.0, 1.0, 1.0, twinkle]));
        }
    }
}
