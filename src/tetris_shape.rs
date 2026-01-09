use crate::retris_colors::*;
use crate::sound_manager::SoundManager;
use crate::tetris_mobile_controller::TetrisMobileController;
use egor::input::{Input, KeyCode};
use egor::math::{Vec2, vec2};
use egor::render::{Color, Graphics};

// ============================================================================
// HOW TO PLAY RETRIS
// ============================================================================
// Arrow Left/Right: Move piece horizontally
// Arrow Down: Speed up falling piece
// Space: Rotate piece clockwise
// Enter: Start game (from title screen)
// ============================================================================

#[derive(Debug)]
pub struct ShapeDimension {
    pub position: Vec2,
}

#[derive(Debug)]
pub enum ShapeName {
    // Gameplay pieces
    Straight(Vec<ShapeDimension>),
    Square(Vec<ShapeDimension>),
    Tee(Vec<ShapeDimension>),
    Ell(Vec<ShapeDimension>),
    Slew(Vec<ShapeDimension>),
    // Display pieces (for UI)
    LetterT(Vec<ShapeDimension>),
    LetterE(Vec<ShapeDimension>),
    LetterR(Vec<ShapeDimension>),
    LetterI(Vec<ShapeDimension>),
    LetterS(Vec<ShapeDimension>),
}

impl std::fmt::Display for ShapeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ShapeName::Straight(_) => "Straight",
            ShapeName::Square(_) => "Square",
            ShapeName::Tee(_) => "Tee",
            ShapeName::Ell(_) => "Ell",
            ShapeName::Slew(_) => "Slew",
            ShapeName::LetterT(_) => "LetterT",
            ShapeName::LetterE(_) => "LetterE",
            ShapeName::LetterR(_) => "LetterR",
            ShapeName::LetterI(_) => "LetterI",
            ShapeName::LetterS(_) => "LetterS",
        };
        write!(f, "{}", name)
    }
}

impl ShapeDimension {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: vec2(x, y),
        }
    }

    /// Rotate 90 degrees clockwise around the origin
    pub fn rotate_clockwise(&mut self) {
        let new_x = self.position.y;
        let new_y = -self.position.x;
        self.position = vec2(new_x, new_y);
    }

    /// Rotate 90 degrees counter-clockwise around the origin
    pub fn rotate_counter_clockwise(&mut self) {
        let new_x = -self.position.y;
        let new_y = self.position.x;
        self.position = vec2(new_x, new_y);
    }
}

impl ShapeName {
    /// Get shape dimensions for each shape type
    /// Dimensions are relative positions from the shape's center/pivot point
    pub fn get_dimensions(&self) -> &Vec<ShapeDimension> {
        match self {
            ShapeName::Straight(dims) => dims,
            ShapeName::Square(dims) => dims,
            ShapeName::Tee(dims) => dims,
            ShapeName::Ell(dims) => dims,
            ShapeName::Slew(dims) => dims,
            ShapeName::LetterT(dims) => dims,
            ShapeName::LetterE(dims) => dims,
            ShapeName::LetterR(dims) => dims,
            ShapeName::LetterI(dims) => dims,
            ShapeName::LetterS(dims) => dims,
        }
    }

    /// Get mutable shape dimensions
    pub fn get_dimensions_mut(&mut self) -> &mut Vec<ShapeDimension> {
        match self {
            ShapeName::Straight(dims) => dims,
            ShapeName::Square(dims) => dims,
            ShapeName::Tee(dims) => dims,
            ShapeName::Ell(dims) => dims,
            ShapeName::Slew(dims) => dims,
            ShapeName::LetterT(dims) => dims,
            ShapeName::LetterE(dims) => dims,
            ShapeName::LetterR(dims) => dims,
            ShapeName::LetterI(dims) => dims,
            ShapeName::LetterS(dims) => dims,
        }
    }

    /// Check if this is a gameplay piece (true) or a display piece (false)
    pub fn is_gameplay_piece(&self) -> bool {
        match self {
            ShapeName::Straight(_)
            | ShapeName::Square(_)
            | ShapeName::Tee(_)
            | ShapeName::Ell(_)
            | ShapeName::Slew(_) => true,
            ShapeName::LetterT(_)
            | ShapeName::LetterE(_)
            | ShapeName::LetterR(_)
            | ShapeName::LetterI(_)
            | ShapeName::LetterS(_) => false,
        }
    }

    /// Create Straight shape (I-piece): 4 blocks in a line
    /// Starts vertically: [0,0], [0,1], [0,2], [0,3] relative to center
    fn new_straight() -> Self {
        let mut dimensions = Vec::new();
        // Vertical line - use integer offsets to avoid rounding issues
        // Center at (0, 0) with blocks at -1, 0, 1, 2
        dimensions.push(ShapeDimension::new(0.0, -1.0));
        dimensions.push(ShapeDimension::new(0.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 1.0));
        dimensions.push(ShapeDimension::new(0.0, 2.0));
        ShapeName::Straight(dimensions)
    }

    /// Create Square shape (O-piece): 2x2 square
    /// Doesn't need rotation, but we'll support it
    fn new_square() -> Self {
        let mut dimensions = Vec::new();
        // 2x2 square aligned to grid cells (integer offsets)
        // Top-left at (0,0) relative to the piece origin cell.
        dimensions.push(ShapeDimension::new(0.0, 0.0));
        dimensions.push(ShapeDimension::new(1.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 1.0));
        dimensions.push(ShapeDimension::new(1.0, 1.0));
        ShapeName::Square(dimensions)
    }

    /// Create Tee shape (T-piece): T shape
    fn new_tee() -> Self {
        let mut dimensions = Vec::new();
        // T shape pointing up
        dimensions.push(ShapeDimension::new(-1.0, 0.0)); // left
        dimensions.push(ShapeDimension::new(0.0, 0.0)); // center
        dimensions.push(ShapeDimension::new(1.0, 0.0)); // right
        dimensions.push(ShapeDimension::new(0.0, 1.0)); // top
        ShapeName::Tee(dimensions)
    }

    /// Create Ell shape (L-piece): L shape
    fn new_ell() -> Self {
        let mut dimensions = Vec::new();
        // L shape pointing up-right
        dimensions.push(ShapeDimension::new(0.0, -1.0)); // bottom
        dimensions.push(ShapeDimension::new(0.0, 0.0)); // middle
        dimensions.push(ShapeDimension::new(0.0, 1.0)); // top
        dimensions.push(ShapeDimension::new(1.0, 1.0)); // right extension
        ShapeName::Ell(dimensions)
    }

    /// Create Slew shape (S-piece): S shape
    fn new_slew() -> Self {
        let mut dimensions = Vec::new();
        // S shape
        dimensions.push(ShapeDimension::new(-1.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 1.0));
        dimensions.push(ShapeDimension::new(1.0, 1.0));
        ShapeName::Slew(dimensions)
    }

    /// Create Letter T shape for display (UI purposes)
    /// 3x5 block letter, centered around origin
    pub fn new_letter_t() -> Self {
        let mut dimensions = Vec::new();
        // Top horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, -2.0));
        dimensions.push(ShapeDimension::new(0.0, -2.0));
        dimensions.push(ShapeDimension::new(1.0, -2.0));
        // Vertical stem
        dimensions.push(ShapeDimension::new(0.0, -1.0));
        dimensions.push(ShapeDimension::new(0.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 1.0));
        dimensions.push(ShapeDimension::new(0.0, 2.0));
        ShapeName::LetterT(dimensions)
    }

    /// Create Letter E shape for display (UI purposes)
    /// 3x5 block letter, centered around origin
    pub fn new_letter_e() -> Self {
        let mut dimensions = Vec::new();
        // Top horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, -2.0));
        dimensions.push(ShapeDimension::new(0.0, -2.0));
        dimensions.push(ShapeDimension::new(1.0, -2.0));
        // Middle horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 0.0));
        dimensions.push(ShapeDimension::new(1.0, 0.0));
        // Bottom horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, 2.0));
        dimensions.push(ShapeDimension::new(0.0, 2.0));
        dimensions.push(ShapeDimension::new(1.0, 2.0));
        // Vertical stem
        dimensions.push(ShapeDimension::new(-1.0, -1.0));
        dimensions.push(ShapeDimension::new(-1.0, 1.0));
        ShapeName::LetterE(dimensions)
    }

    /// Create Letter R shape for display (UI purposes)
    /// 3x5 block letter, centered around origin
    pub fn new_letter_r() -> Self {
        let mut dimensions = Vec::new();
        // Top horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, -2.0));
        dimensions.push(ShapeDimension::new(0.0, -2.0));
        dimensions.push(ShapeDimension::new(1.0, -2.0));
        // Middle horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 0.0));
        dimensions.push(ShapeDimension::new(1.0, 0.0));
        // Left vertical stem
        dimensions.push(ShapeDimension::new(-1.0, -1.0));
        dimensions.push(ShapeDimension::new(-1.0, 1.0));
        dimensions.push(ShapeDimension::new(-1.0, 2.0));
        // Right vertical (top part)
        dimensions.push(ShapeDimension::new(1.0, -1.0));
        // Diagonal leg
        dimensions.push(ShapeDimension::new(0.0, 1.0));
        dimensions.push(ShapeDimension::new(1.0, 2.0));
        ShapeName::LetterR(dimensions)
    }

    /// Create Letter I shape for display (UI purposes)
    /// 3x5 block letter, centered around origin
    pub fn new_letter_i() -> Self {
        let mut dimensions = Vec::new();
        // Top horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, -2.0));
        dimensions.push(ShapeDimension::new(0.0, -2.0));
        dimensions.push(ShapeDimension::new(1.0, -2.0));
        // Bottom horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, 2.0));
        dimensions.push(ShapeDimension::new(0.0, 2.0));
        dimensions.push(ShapeDimension::new(1.0, 2.0));
        // Vertical stem
        dimensions.push(ShapeDimension::new(0.0, -1.0));
        dimensions.push(ShapeDimension::new(0.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 1.0));
        ShapeName::LetterI(dimensions)
    }

    /// Create Letter S shape for display (UI purposes)
    /// 3x5 block letter, centered around origin
    pub fn new_letter_s() -> Self {
        let mut dimensions = Vec::new();
        // Top horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, -2.0));
        dimensions.push(ShapeDimension::new(0.0, -2.0));
        dimensions.push(ShapeDimension::new(1.0, -2.0));
        // Middle horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, 0.0));
        dimensions.push(ShapeDimension::new(0.0, 0.0));
        dimensions.push(ShapeDimension::new(1.0, 0.0));
        // Bottom horizontal bar
        dimensions.push(ShapeDimension::new(-1.0, 2.0));
        dimensions.push(ShapeDimension::new(0.0, 2.0));
        dimensions.push(ShapeDimension::new(1.0, 2.0));
        // Top left vertical
        dimensions.push(ShapeDimension::new(-1.0, -1.0));
        // Bottom right vertical
        dimensions.push(ShapeDimension::new(1.0, 1.0));
        ShapeName::LetterS(dimensions)
    }

    pub fn get_shape_by_index(index: i32) -> ShapeName {
        match index {
            0 => ShapeName::new_straight(),
            1 => ShapeName::new_square(),
            2 => ShapeName::new_tee(),
            3 => ShapeName::new_ell(),
            _ => ShapeName::new_slew(),
        }
    }

    /// Rotate the shape 90 degrees clockwise
    pub fn rotate_clockwise(&mut self) {
        // Display pieces don't rotate
        if !self.is_gameplay_piece() {
            return;
        }

        // Keep Square aligned (no-op rotation). Other shapes rotate normally.
        if let ShapeName::Square(_) = self {
            return;
        }

        for dimension in self.get_dimensions_mut() {
            dimension.rotate_clockwise();
        }
    }

    /// Rotate the shape 90 degrees counter-clockwise
    pub fn rotate_counter_clockwise(&mut self) {
        // Display pieces don't rotate
        if !self.is_gameplay_piece() {
            return;
        }

        // Keep Square aligned (no-op rotation). Other shapes rotate normally.
        if let ShapeName::Square(_) = self {
            return;
        }

        for dimension in self.get_dimensions_mut() {
            dimension.rotate_counter_clockwise();
        }
    }
}

pub struct TetrisShapeNode {
    pub velocity: u16, // Cells per second
    pub cell_x: i32,   // Grid cell X position
    pub cell_y: i32,   // Grid cell Y position
    pub stopped: bool,
    pub shape_name: ShapeName,
    pub color: Color,
    pub cell_size: f32,
    pub grid_position: Vec2,
    pub grid_width_cells: usize,
    pub grid_height_cells: usize,
    pub fall_timer: f32,            // Accumulator for downward movement
    pub horizontal_move_timer: f32, // Accumulator for horizontal movement
    pub das_timer: f32,             // DAS (Delayed Auto Shift) timer
    pub das_active: bool,           // Whether continuous movement is active
    pub last_direction: i32,        // Last horizontal direction (-1, 0, 1)
}

impl TetrisShapeNode {
    pub fn new(
        velocity: u16,
        cell_x: i32,
        cell_y: i32,
        cell_size: f32,
        grid_position: Vec2,
        grid_width_cells: usize,
        grid_height_cells: usize,
    ) -> TetrisShapeNode {
        let shape_index = rand::random_range(0..5);
        let random_shape = ShapeName::get_shape_by_index(shape_index);

        // Set color based on shape type
        let color = match &random_shape {
            ShapeName::Straight(_) => COLOR_CYAN,
            ShapeName::Square(_) => COLOR_YELLOW,
            ShapeName::Tee(_) => COLOR_MAGENTA,
            ShapeName::Ell(_) => COLOR_ORANGE,
            ShapeName::Slew(_) => COLOR_SOFTWARE_GREEN,
            // Display pieces - these shouldn't be randomly generated, but handle them anyway
            ShapeName::LetterT(_)
            | ShapeName::LetterE(_)
            | ShapeName::LetterR(_)
            | ShapeName::LetterI(_)
            | ShapeName::LetterS(_) => Color::WHITE,
        };

        TetrisShapeNode {
            velocity,
            cell_x,
            cell_y,
            stopped: false,
            shape_name: random_shape,
            color,
            cell_size,
            grid_position,
            grid_width_cells,
            grid_height_cells,
            fall_timer: 0.0, // Start at 0 so piece doesn't immediately fall on spawn
            horizontal_move_timer: 0.0,
            das_timer: 0.0,
            das_active: false,
            last_direction: 0,
        }
    }

    /// Create a TetrisShapeNode with a specific shape and color (for title screen, etc.)
    pub fn new_with_shape_and_color(
        velocity: u16,
        cell_x: i32,
        cell_y: i32,
        cell_size: f32,
        grid_position: Vec2,
        grid_width_cells: usize,
        grid_height_cells: usize,
        shape_name: ShapeName,
        color: Color,
    ) -> TetrisShapeNode {
        TetrisShapeNode {
            velocity,
            cell_x,
            cell_y,
            stopped: false,
            shape_name,
            color,
            cell_size,
            grid_position,
            grid_width_cells,
            grid_height_cells,
            fall_timer: 0.0,
            horizontal_move_timer: 0.0,
            das_timer: 0.0,
            das_active: false,
            last_direction: 0,
        }
    }

    /// Convert grid cell position to world position
    pub fn world_position(&self) -> Vec2 {
        vec2(
            self.grid_position.x + self.cell_x as f32 * self.cell_size,
            self.grid_position.y + self.cell_y as f32 * self.cell_size,
        )
    }

    /// Check if the current position is valid (in bounds and not colliding)
    fn is_position_valid(
        &self,
        test_cell_x: i32,
        test_cell_y: i32,
        grid: &crate::grid::Grid,
    ) -> bool {
        let cells = self.get_occupied_cells_at_position(test_cell_x, test_cell_y);

        for (cell_x, cell_y) in cells {
            // Check grid boundaries
            if cell_x < 0 || cell_x >= self.grid_width_cells as i32 {
                return false;
            }
            if cell_y < 0 || cell_y >= self.grid_height_cells as i32 {
                return false;
            }

            // Check if this cell is occupied
            if grid.is_cell_occupied(cell_x, cell_y) {
                return false;
            }
        }

        true
    }

    /// Get occupied cells at a specific position (for testing rotations)
    fn get_occupied_cells_at_position(&self, cell_x: i32, cell_y: i32) -> Vec<(i32, i32)> {
        let dimensions = self.shape_name.get_dimensions();
        let mut cells = Vec::new();

        for dim in dimensions {
            let offset_x = dim.position.x.round() as i32;
            let offset_y = dim.position.y.round() as i32;

            cells.push((cell_x + offset_x, cell_y + offset_y));
        }

        cells
    }

    /// Rotate the shape clockwise with wall kick (try shifting if rotation would be invalid)
    pub fn rotate_clockwise_with_wall_kick(&mut self, grid: &crate::grid::Grid) -> bool {
        // Try rotation at current position
        self.shape_name.rotate_clockwise();

        if self.is_position_valid(self.cell_x, self.cell_y, grid) {
            return true; // Rotation is valid
        }

        // Try wall kicks: shift left, then right
        const WALL_KICK_OFFSETS: [i32; 5] = [-1, 1, -2, 2, 0]; // Try -1, +1, -2, +2, then revert

        for &offset in &WALL_KICK_OFFSETS {
            if offset == 0 {
                // Last attempt: revert rotation if no valid position found
                self.shape_name.rotate_counter_clockwise();
                return false;
            }

            let test_x = self.cell_x + offset;
            if self.is_position_valid(test_x, self.cell_y, grid) {
                self.cell_x = test_x;
                return true; // Found valid position
            }
        }

        // Shouldn't reach here, but revert rotation just in case
        self.shape_name.rotate_counter_clockwise();
        false
    }

    /// Get all occupied cell positions in grid coordinates
    pub fn get_occupied_cells(&self) -> Vec<(i32, i32)> {
        let dimensions = self.shape_name.get_dimensions();
        let mut cells = Vec::new();

        for dim in dimensions {
            // Shape dimensions are already in cell units (1.0 = 1 cell)
            // Just round to get integer cell offsets
            let offset_x = dim.position.x.round() as i32;
            let offset_y = dim.position.y.round() as i32;

            cells.push((self.cell_x + offset_x, self.cell_y + offset_y));
        }

        cells
    }

    /// Get all occupied cell positions with their color
    pub fn get_occupied_cells_with_color(&self) -> Vec<(i32, i32, Color)> {
        self.get_occupied_cells()
            .iter()
            .map(|&(x, y)| (x, y, self.color))
            .collect()
    }

    /// Check if the shape can move left/right without going off grid or colliding
    fn can_move_horizontal(&self, direction: i32, grid: &crate::grid::Grid) -> bool {
        // Get current occupied cells and offset x by direction
        let current_cells = self.get_occupied_cells();

        for (cell_x, cell_y) in current_cells {
            let test_cell_x = cell_x + direction;

            // Check grid boundaries
            if test_cell_x < 0 || test_cell_x >= self.grid_width_cells as i32 {
                return false;
            }

            // Check if this cell is occupied
            if grid.is_cell_occupied(test_cell_x, cell_y) {
                return false;
            }
        }

        true
    }

    /// Update the shape - handles input and movement
    pub fn update(
        &mut self,
        input: &Input,
        fixed_delta: f32,
        grid: &mut crate::grid::Grid,
        sound_manager: &mut SoundManager,
        mobile_controller: &mut TetrisMobileController,
        screen_width: f32,
        screen_height: f32,
        grid_bottom_y: Option<f32>,
    ) {
        // Update mobile controller
        // Get piece info for touch following and rotation
        let piece_world_pos = self.world_position();
        let piece_world_x = Some(piece_world_pos.x);
        let piece_cell_size = Some(self.cell_size);
        mobile_controller.update(
            input,
            screen_width,
            screen_height,
            piece_world_x,
            Some(piece_world_pos),
            piece_cell_size,
            grid_bottom_y,
        );

        // Handle rotation with wall kick (keyboard or tap on piece)
        if input.key_pressed(KeyCode::Space) || mobile_controller.rotate_pressed() {
            if self.rotate_clockwise_with_wall_kick(grid) {
                // Play shuffle sound only if rotation succeeded
                sound_manager.play_shuffle();
            }
        }

        // Handle horizontal movement with DAS (Delayed Auto Shift)
        // DAS: Initial press moves immediately, then delay, then continuous movement
        const DAS_DELAY: f32 = 0.133; // Delay before auto-repeat starts (seconds)
        const ARR_SPEED: f32 = 20.0;  // Auto-Repeat Rate (cells per second after DAS activates)

        if !self.stopped {
            let moving_left = input.key_pressed(KeyCode::ArrowLeft)
                || input.key_held(KeyCode::ArrowLeft)
                || mobile_controller.left_held();
            let moving_right = input.key_pressed(KeyCode::ArrowRight)
                || input.key_held(KeyCode::ArrowRight)
                || mobile_controller.right_held();

            // Determine direction: if both are held, don't move (prioritize neither)
            let direction = if moving_left && !moving_right {
                Some(-1)
            } else if moving_right && !moving_left {
                Some(1)
            } else {
                None // Both held or neither held
            };

            if let Some(dir) = direction {
                // Check if direction changed
                if dir != self.last_direction {
                    // Direction changed - reset DAS and move immediately
                    self.last_direction = dir;
                    self.das_timer = 0.0;
                    self.das_active = false;
                    self.horizontal_move_timer = 0.0;

                    // Initial move on direction press
                    if self.can_move_horizontal(dir, grid) {
                        self.cell_x += dir;
                    }
                } else {
                    // Same direction held - update DAS
                    if !self.das_active {
                        // In DAS delay phase
                        self.das_timer += fixed_delta;
                        if self.das_timer >= DAS_DELAY {
                            // DAS delay complete - activate auto-repeat
                            self.das_active = true;
                            self.horizontal_move_timer = 0.0;
                        }
                    } else {
                        // DAS active - continuous movement at ARR speed
                        let time_per_cell = 1.0 / ARR_SPEED;
                        self.horizontal_move_timer += fixed_delta;

                        // Process horizontal movement timer
                        while self.horizontal_move_timer >= time_per_cell {
                            // Check if we can move in this direction
                            if self.can_move_horizontal(dir, grid) {
                                self.cell_x += dir;
                                self.horizontal_move_timer -= time_per_cell;
                            } else {
                                // Hit wall - keep DAS active but stop moving
                                self.horizontal_move_timer = 0.0;
                                break;
                            }
                        }
                    }
                }
            } else {
                // No direction held - reset DAS
                self.last_direction = 0;
                self.das_timer = 0.0;
                self.das_active = false;
                self.horizontal_move_timer = 0.0;
            }
        }

        // Handle downward movement - discrete grid movement
        // Velocity is in cells per second, so we move one cell every (1.0 / velocity) seconds
        if !self.stopped && self.velocity > 0 {
            // Triple speed when holding down arrow
            let effective_velocity =
                if input.key_held(KeyCode::ArrowDown) || mobile_controller.red_button_pressed() {
                    self.velocity * 5
                } else {
                    self.velocity
                };

            let time_per_cell = 1.0 / effective_velocity as f32;
            self.fall_timer += fixed_delta;

            // Process fall timer - check collision before each movement
            while self.fall_timer >= time_per_cell {
                // Get current occupied cells
                let shape_cells = self.get_occupied_cells();

                // Check with grid if can move down
                if !grid.can_move_down(&shape_cells) {
                    // Can't move down, stop (Game will handle transferring to grid)
                    self.stopped = true;
                    self.fall_timer = 0.0;
                    break;
                }

                // Move down one cell
                self.cell_y += 1;
                self.fall_timer -= time_per_cell;
            }
        }
    }

    /// Draw the shape
    pub fn draw(
        &mut self,
        gfx: &mut Graphics,
        _alpha: f32,
        mobile_controller: &mut TetrisMobileController,
    ) {
        // Draw mobile controller
        mobile_controller.draw(gfx);
        const BORDER_WIDTH: f32 = 1.0;

        // Get the world position of the piece's cell position (top-left of cell_x, cell_y)
        let mut world_pos = self.world_position();

        // Add floating animation if stopped (for title screen)
        if self.stopped {
            // Gentle sine wave bobbing motion
            const FLOAT_AMPLITUDE: f32 = 8.0; // pixels
            const FLOAT_SPEED: f32 = 1.5; // cycles per second
            let float_offset =
                (self.fall_timer * FLOAT_SPEED * std::f32::consts::TAU).sin() * FLOAT_AMPLITUDE;
            world_pos.y += float_offset;
        }

        // Draw each block using dimension offsets
        // dimension.position is in cell units (1.0 = 1 cell), so multiply by cell_size for pixels
        for dimension in self.shape_name.get_dimensions() {
            let block_world_pos = world_pos + dimension.position * self.cell_size;

            // Draw fill (smaller rectangle, inset by border width)
            let fill_size = vec2(
                self.cell_size - BORDER_WIDTH * 2.0,
                self.cell_size - BORDER_WIDTH * 2.0,
            );
            let fill_pos = block_world_pos + vec2(BORDER_WIDTH, BORDER_WIDTH);
            gfx.rect().size(fill_size).at(fill_pos).color(self.color);
        }
    }
}
