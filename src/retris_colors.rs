pub use egor::render::Color;

// Tetris piece colors - softer, easier on the eyes
pub const COLOR_CYAN: Color = Color::new([0.3, 0.7, 0.8, 0.8]); // Straight (I-piece)
pub const COLOR_YELLOW: Color = Color::new([0.8, 0.75, 0.4, 0.8]); // Square (O-piece)
pub const COLOR_MAGENTA: Color = Color::new([0.75, 0.4, 0.7, 0.8]); // Tee (T-piece)
pub const COLOR_ORANGE: Color = Color::new([0.8, 0.6, 0.35, 0.8]); // Ell (L-piece)
pub const COLOR_SOFTWARE_GREEN: Color = Color::new([0.15, 0.8, 0.35, 0.8]); // Slew (S-piece)

// UI colors
pub const COLOR_BORDER_GREEN: Color = Color::new([0.2, 0.95, 0.4, 0.8]); // Grid borders
pub const COLOR_TEXT_GREEN: Color = Color::new([0.15, 0.8, 0.35, 1.0]); // UI text
pub const COLOR_CELL_BORDER: Color = Color::new([0.0, 0.0, 0.0, 1.0]); // Black cell borders
pub const COLOR_DARK_GRAY: Color = Color::new([0.4, 0.4, 0.4, 1.0]); // Dark gray for subtle text

// Background
pub const COLOR_BACKGROUND: Color = Color::new([0.05, 0.05, 0.08, 1.0]); // Dark blue-gray
pub const COLOR_BACKGROUND_ALPHA: Color = Color::new([0.05, 0.05, 0.08, 0.7]); // Dark blue-gray

// All game piece colors in an array for easy selection
pub const PIECE_COLORS: [Color; 5] = [
    COLOR_CYAN,
    COLOR_YELLOW,
    COLOR_MAGENTA,
    COLOR_ORANGE,
    COLOR_SOFTWARE_GREEN,
];
