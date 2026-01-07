use egor::input::{Input, KeyCode};
use egor::math::{vec2, Vec2};
use egor::render::{Color, Graphics};

// Color constants for different Tetris shapes
const COLOR_CYAN: Color = Color::new([0.0, 1.0, 1.0, 1.0]);      // Straight (I-piece)
const COLOR_YELLOW: Color = Color::new([1.0, 1.0, 0.0, 1.0]);    // Square (O-piece)
const COLOR_MAGENTA: Color = Color::new([1.0, 0.0, 1.0, 1.0]);   // Tee (T-piece)
const COLOR_ORANGE: Color = Color::new([1.0, 0.65, 0.0, 1.0]);   // Ell (L-piece)

#[derive(Debug)]
pub struct ShapeDimension {
    pub position: Vec2,
    pub size: i16,
}

#[derive(Debug)]
pub enum ShapeName {
    Straight(Vec<ShapeDimension>),
    Square(Vec<ShapeDimension>),
    Tee(Vec<ShapeDimension>),
    Ell(Vec<ShapeDimension>),
    Slew(Vec<ShapeDimension>),
}

impl std::fmt::Display for ShapeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ShapeName::Straight(_) => "Straight",
            ShapeName::Square(_) => "Square",
            ShapeName::Tee(_) => "Tee",
            ShapeName::Ell(_) => "Ell",
            ShapeName::Slew(_) => "Slew",
        };
        write!(f, "{}", name)
    }
}

impl ShapeDimension {
    pub fn new(x: f32, y: f32, size: i16) -> Self {
        Self {
            position: vec2(x, y),
            size,
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
        }
    }

    /// Create Straight shape (I-piece): 4 blocks in a line
    /// Starts vertically: [0,0], [0,1], [0,2], [0,3] relative to center
    fn new_straight() -> Self {
        let block_size = 1;
        let mut dimensions = Vec::new();
        // Vertical line - use integer offsets to avoid rounding issues
        // Center at (0, 0) with blocks at -1, 0, 1, 2
        dimensions.push(ShapeDimension::new(0.0, -1.0, block_size));
        dimensions.push(ShapeDimension::new(0.0, 0.0, block_size));
        dimensions.push(ShapeDimension::new(0.0, 1.0, block_size));
        dimensions.push(ShapeDimension::new(0.0, 2.0, block_size));
        ShapeName::Straight(dimensions)
    }

    /// Create Square shape (O-piece): 2x2 square
    /// Doesn't need rotation, but we'll support it
    fn new_square() -> Self {
        let block_size = 1;
        let mut dimensions = Vec::new();
        // 2x2 square aligned to grid cells (integer offsets)
        // Top-left at (0,0) relative to the piece origin cell.
        dimensions.push(ShapeDimension::new(0.0, 0.0, block_size));
        dimensions.push(ShapeDimension::new(1.0, 0.0, block_size));
        dimensions.push(ShapeDimension::new(0.0, 1.0, block_size));
        dimensions.push(ShapeDimension::new(1.0, 1.0, block_size));
        ShapeName::Square(dimensions)
    }

    /// Create Tee shape (T-piece): T shape
    fn new_tee() -> Self {
        let block_size = 1;
        let mut dimensions = Vec::new();
        // T shape pointing up
        dimensions.push(ShapeDimension::new(-1.0, 0.0, block_size)); // left
        dimensions.push(ShapeDimension::new(0.0, 0.0, block_size));  // center
        dimensions.push(ShapeDimension::new(1.0, 0.0, block_size));  // right
        dimensions.push(ShapeDimension::new(0.0, 1.0, block_size));  // top
        ShapeName::Tee(dimensions)
    }

    /// Create Ell shape (L-piece): L shape
    fn new_ell() -> Self {
        let block_size = 1;
        let mut dimensions = Vec::new();
        // L shape pointing up-right
        dimensions.push(ShapeDimension::new(0.0, -1.0, block_size)); // bottom
        dimensions.push(ShapeDimension::new(0.0, 0.0, block_size));  // middle
        dimensions.push(ShapeDimension::new(0.0, 1.0, block_size));  // top
        dimensions.push(ShapeDimension::new(1.0, 1.0, block_size));  // right extension
        ShapeName::Ell(dimensions)
    }

    /// Create Slew shape (S-piece): S shape
    fn new_slew() -> Self {
        let block_size = 1;
        let mut dimensions = Vec::new();
        // S shape
        dimensions.push(ShapeDimension::new(-1.0, 0.0, block_size));
        dimensions.push(ShapeDimension::new(0.0, 0.0, block_size));
        dimensions.push(ShapeDimension::new(0.0, 1.0, block_size));
        dimensions.push(ShapeDimension::new(1.0, 1.0, block_size));
        ShapeName::Slew(dimensions)
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
    pub velocity: f32,  // Cells per second
    pub cell_x: i32,    // Grid cell X position
    pub cell_y: i32,    // Grid cell Y position
    pub stopped: bool,
    pub shape_name: ShapeName,
    pub color: Color,
    cell_size: f32,
    grid_position: Vec2,
    grid_width_cells: usize,
    grid_height_cells: usize,
    fall_timer: f32,  // Accumulator for downward movement
}

impl TetrisShapeNode {
    pub fn new(velocity: f32, cell_x: i32, cell_y: i32, cell_size: f32, grid_position: Vec2, grid_width_cells: usize, grid_height_cells: usize) -> TetrisShapeNode {
        let shape_index = rand::random_range(0..5);
        let random_shape = ShapeName::get_shape_by_index(shape_index);
        
        // Set color based on shape type
        let color = match &random_shape {
            ShapeName::Straight(_) => COLOR_CYAN,
            ShapeName::Square(_) => COLOR_YELLOW,
            ShapeName::Tee(_) => COLOR_MAGENTA,
            ShapeName::Ell(_) => COLOR_ORANGE,
            ShapeName::Slew(_) => Color::GREEN,
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
            fall_timer: 0.0,  // Start at 0 so piece doesn't immediately fall on spawn
        }
    }

    /// Convert grid cell position to world position
    fn world_position(&self) -> Vec2 {
        vec2(
            self.grid_position.x + self.cell_x as f32 * self.cell_size,
            self.grid_position.y + self.cell_y as f32 * self.cell_size,
        )
    }

    /// Check if the current position is valid (in bounds and not colliding)
    fn is_position_valid(&self, test_cell_x: i32, test_cell_y: i32, grid: &crate::grid::Grid) -> bool {
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

    /// Rotate the shape clockwise
    pub fn rotate_clockwise(&mut self) {
        self.shape_name.rotate_clockwise();
    }

    /// Rotate the shape counter-clockwise
    pub fn rotate_counter_clockwise(&mut self) {
        self.shape_name.rotate_counter_clockwise();
    }

    /// Get the absolute positions of all blocks in this shape
    pub fn get_block_positions(&self) -> Vec<Vec2> {
        let world_pos = self.world_position();
        self.shape_name
            .get_dimensions()
            .iter()
            .map(|dim| world_pos + dim.position * self.cell_size)
            .collect()
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
    pub fn update(&mut self, input: &Input, fixed_delta: f32, grid: &mut crate::grid::Grid) {
        // Handle rotation with wall kick
        if input.key_pressed(KeyCode::Space) {
            self.rotate_clockwise_with_wall_kick(grid);
        }

        // Handle horizontal movement - discrete, one cell at a time
        if input.key_pressed(KeyCode::ArrowLeft) {
            if self.can_move_horizontal(-1, grid) {
                self.cell_x -= 1;
            }
        }
        
        if input.key_pressed(KeyCode::ArrowRight) {
            if self.can_move_horizontal(1, grid) {
                self.cell_x += 1;
            }
        }

        // Handle downward movement - discrete grid movement
        // Velocity is in cells per second, so we move one cell every (1.0 / velocity) seconds
        if !self.stopped && self.velocity > 0.0 {
            // Triple speed when holding down arrow
            let effective_velocity = if input.key_held(KeyCode::ArrowDown) {
                self.velocity * 5.0
            } else {
                self.velocity
            };
            
            let time_per_cell = 1.0 / effective_velocity;
            self.fall_timer += fixed_delta;
            
            // Process fall timer - check collision before each movement
            while self.fall_timer >= time_per_cell {
                // Get current occupied cells
                let shape_cells = self.get_occupied_cells();
                
                // Check with grid if can move down
                if !grid.can_move_down(&shape_cells) {
                    // Can't move down, stop (Game will handle transferring to grid)
                    println!("Piece stopped! Position: ({}, {}), Cells: {:?}, fall_timer: {}, time_per_cell: {}", 
                             self.cell_x, self.cell_y, shape_cells, self.fall_timer, time_per_cell);
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
    pub fn draw(&mut self, gfx: &mut Graphics, _alpha: f32) {
        const BORDER_WIDTH: f32 = 1.0;
        const BORDER_COLOR: Color = Color::BLACK; 
        
        let block_size_vec = vec2(self.cell_size, self.cell_size);
        
        // Get the world position of the piece's cell position (top-left of cell_x, cell_y)
        let world_pos = self.world_position();

        // Draw each block using dimension offsets
        // dimension.position is in cell units (1.0 = 1 cell), so multiply by cell_size for pixels
        for dimension in self.shape_name.get_dimensions() {
            let block_world_pos = world_pos + dimension.position * self.cell_size;

            // Draw border (larger rectangle)
            gfx.rect()
                .size(block_size_vec)
                .at(block_world_pos)
                .color(BORDER_COLOR);
            
            // Draw fill (smaller rectangle, inset by border width)
            let fill_size = vec2(self.cell_size - BORDER_WIDTH * 2.0, self.cell_size - BORDER_WIDTH * 2.0);
            let fill_pos = block_world_pos + vec2(BORDER_WIDTH, BORDER_WIDTH);
            gfx.rect()
                .size(fill_size)
                .at(fill_pos)
                .color(self.color);
        }
    }
}
