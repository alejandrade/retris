use crate::retris_colors::COLOR_BACKGROUND;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
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
}

impl Background {
    pub fn new(star_count: usize) -> Self {
        let mut rng = rand::rng();
        let mut stars = Vec::new();

        for _ in 0..star_count {
            stars.push(Star {
                x: rng.random_range(-((SCREEN_WIDTH / 2) as f32)..((SCREEN_WIDTH / 2) as f32)),
                y: rng
                    .random_range(-((SCREEN_HEIGHT / 2) as f32)..((SCREEN_HEIGHT / 2) as f32)),
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
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.elapsed_time += delta;
        
        let half_width = (SCREEN_WIDTH / 2) as f32;
        let half_height = (SCREEN_HEIGHT / 2) as f32;

        for star in self.stars.iter_mut() {
            // Update position with gentle drift
            star.x += star.velocity_x * delta;
            star.y += star.velocity_y * delta;

            // Wrap around screen edges
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
            let twinkle = ((self.elapsed_time * star.twinkle_speed + star.twinkle_offset).sin() * 0.3 + 0.7)
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
