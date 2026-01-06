use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

/// 2D vector with x and y components
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };
    pub const UP: Vec2 = Vec2 { x: 0.0, y: -1.0 }; // Screen coordinates: Y increases downward
    pub const DOWN: Vec2 = Vec2 { x: 0.0, y: 1.0 };
    pub const LEFT: Vec2 = Vec2 { x: -1.0, y: 0.0 };
    pub const RIGHT: Vec2 = Vec2 { x: 1.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::ZERO
        }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn distance(&self, other: &Self) -> f32 {
        (*self - *other).length()
    }

    pub fn distance_squared(&self, other: &Self) -> f32 {
        (*self - *other).length_squared()
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn rotate(&self, angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<Vec2> for (f32, f32) {
    fn from(vec: Vec2) -> Self {
        (vec.x, vec.y)
    }
}

/// Rotation represented as an angle in radians
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    pub angle: f32, // in radians
}

impl Rotation {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }

    pub fn from_degrees(degrees: f32) -> Self {
        Self {
            angle: degrees.to_radians(),
        }
    }

    pub fn to_degrees(&self) -> f32 {
        self.angle.to_degrees()
    }

    pub fn sin(&self) -> f32 {
        self.angle.sin()
    }

    pub fn cos(&self) -> f32 {
        self.angle.cos()
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        // Normalize angles to [-PI, PI] range for proper interpolation
        let mut diff = other.angle - self.angle;
        while diff > std::f32::consts::PI {
            diff -= 2.0 * std::f32::consts::PI;
        }
        while diff < -std::f32::consts::PI {
            diff += 2.0 * std::f32::consts::PI;
        }
        Self {
            angle: self.angle + diff * t,
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self { angle: 0.0 }
    }
}

/// 2D Transform containing position, rotation, and scale
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: Rotation,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            rotation: Rotation::default(),
            scale: Vec2::ONE,
        }
    }

    pub fn with_rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_scale_uniform(mut self, scale: f32) -> Self {
        self.scale = Vec2::new(scale, scale);
        self
    }

    /// Transform a point from local space to world space
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        // Apply scale
        let scaled = Vec2::new(point.x * self.scale.x, point.y * self.scale.y);
        // Apply rotation
        let rotated = scaled.rotate(self.rotation.angle);
        // Apply translation
        rotated + self.position
    }

    /// Transform a direction vector (ignores position and scale)
    pub fn transform_direction(&self, direction: Vec2) -> Vec2 {
        direction.rotate(self.rotation.angle)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: Rotation::default(),
            scale: Vec2::ONE,
        }
    }
}

/// Rectangle defined by position (top-left corner) and size
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    pub fn from_position_size(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }

    pub fn from_center(center: Vec2, size: Vec2) -> Self {
        Self {
            position: Vec2::new(center.x - size.x / 2.0, center.y - size.y / 2.0),
            size,
        }
    }

    pub fn x(&self) -> f32 {
        self.position.x
    }

    pub fn y(&self) -> f32 {
        self.position.y
    }

    pub fn width(&self) -> f32 {
        self.size.x
    }

    pub fn height(&self) -> f32 {
        self.size.y
    }

    pub fn left(&self) -> f32 {
        self.position.x
    }

    pub fn right(&self) -> f32 {
        self.position.x + self.size.x
    }

    pub fn top(&self) -> f32 {
        self.position.y
    }

    pub fn bottom(&self) -> f32 {
        self.position.y + self.size.y
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(
            self.position.x + self.size.x / 2.0,
            self.position.y + self.size.y / 2.0,
        )
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }
}

/// Utility functions for common math operations
pub mod math {

    /// Linear interpolation between two values
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// Clamp a value between min and max
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.max(min).min(max)
    }

    /// Clamp a value between 0.0 and 1.0
    pub fn clamp01(value: f32) -> f32 {
        clamp(value, 0.0, 1.0)
    }

    /// Smooth step interpolation (ease in/out)
    pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = clamp01((x - edge0) / (edge1 - edge0));
        t * t * (3.0 - 2.0 * t)
    }

    /// Convert degrees to radians
    pub fn deg_to_rad(degrees: f32) -> f32 {
        degrees.to_radians()
    }

    /// Convert radians to degrees
    pub fn rad_to_deg(radians: f32) -> f32 {
        radians.to_degrees()
    }

    /// Move towards a target value by a maximum delta
    pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
        if (target - current).abs() <= max_delta {
            target
        } else {
            current + (target - current).signum() * max_delta
        }
    }
}
