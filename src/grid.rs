use egor::math::{vec2, Vec2};
use egor::input::Input;
use egor::render::{Color, Graphics};
use std::collections::HashMap;

pub struct node {
    position: Vec2,
    size: Vec2,
}

pub struct Grid {
    position: Vec2,
    rows: Vec<node>,
    cols: Vec<node>,
    width: usize,
    height: usize,  // Total height including spawn area
    visible_height: usize,  // Visible height on screen
    cell_size: f32,
    visible_position: Vec2,  // Position of visible area (for drawing border)
    occupied_cells: HashMap<(i32, i32), Color>,  // Track which cells are occupied and their colors
}

impl Grid {
    pub fn new(screen_width: f32, screen_height: f32, width_cells: usize, visible_height_cells: usize, min_padding: f32) -> Self {
        const SPAWN_ROWS: usize = 4; // Extra rows above screen for spawning
        
        // Calculate available space for the grid (screen minus padding on all sides)
        let available_width = screen_width - (min_padding * 2.0);
        let available_height = screen_height - (min_padding * 2.0);

        // Calculate cell size based on both constraints - use the smaller one to ensure it fits
        let cell_size_by_width = available_width / width_cells as f32;
        let cell_size_by_height = available_height / visible_height_cells as f32;
        let cell_size = cell_size_by_width.min(cell_size_by_height).floor();
        
        // Calculate grid dimensions
        let grid_width = width_cells as f32 * cell_size;
        let visible_grid_height = visible_height_cells as f32 * cell_size;
        let total_height_cells = visible_height_cells + SPAWN_ROWS;

        // Coordinate system has (0,0) at center of screen
        // Position the visible area centered, but extend total grid upward
        let visible_position = vec2(-grid_width / 2.0, -visible_grid_height / 2.0);
        // Total grid starts higher up (spawn rows above visible area)
        let position = vec2(-grid_width / 2.0, visible_position.y - (SPAWN_ROWS as f32 * cell_size));

        let mut rows = Vec::new();
        let cols = Vec::new();

        // Create nodes for each cell in the total grid (including spawn area)
        for row in 0..total_height_cells {
            for col in 0..width_cells {
                let node_pos = vec2(
                    position.x + col as f32 * cell_size,
                    position.y + row as f32 * cell_size,
                );
                rows.push(node {
                    position: node_pos,
                    size: vec2(cell_size, cell_size),
                });
            }
        }

        Self {
            position,
            rows,
            cols,
            width: width_cells,
            height: total_height_cells,
            visible_height: visible_height_cells,
            cell_size,
            visible_position,
            occupied_cells: HashMap::new(),
        }
    }

    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn width_cells(&self) -> usize {
        self.width
    }

    pub fn height_cells(&self) -> usize {
        self.height
    }

    pub fn visible_height_cells(&self) -> usize {
        self.visible_height
    }

    /// Convert world position to grid cell coordinates
    pub fn world_to_cell(&self, world_pos: Vec2) -> (i32, i32) {
        let cell_x = ((world_pos.x - self.position.x) / self.cell_size).round() as i32;
        let cell_y = ((world_pos.y - self.position.y) / self.cell_size).round() as i32;
        (cell_x, cell_y)
    }

    /// Check if a cell is occupied
    pub fn is_cell_occupied(&self, cell_x: i32, cell_y: i32) -> bool {
        // Check bounds
        if cell_x < 0 || cell_x >= self.width as i32 || cell_y < 0 || cell_y >= self.height as i32 {
            return true; // Out of bounds counts as occupied
        }
        self.occupied_cells.contains_key(&(cell_x, cell_y))
    }

    /// Check if a world position would be in an occupied cell
    pub fn is_position_occupied(&self, world_pos: Vec2) -> bool {
        let (cell_x, cell_y) = self.world_to_cell(world_pos);
        self.is_cell_occupied(cell_x, cell_y)
    }

    /// Mark cells as occupied with their colors (called when a tetris piece stops)
    pub fn mark_cells_occupied(&mut self, cells: &[(i32, i32, Color)]) {
        for &(x, y, color) in cells {
            if x >= 0 && x < self.width as i32 && 
               y >= 0 && y < self.height as i32 {
                self.occupied_cells.insert((x, y), color);
            } else {
                println!("WARNING: Filtered out out-of-bounds cell: ({}, {}) - bounds: width={}, height={}", 
                         x, y, self.width, self.height);
            }
        }
    }

    /// Check if a shape at the given position can move down one cell
    pub fn can_move_down(&self, shape_cells: &[(i32, i32)]) -> bool {
        // First, check if any cells are still above the grid (negative Y)
        // If so, the piece must continue falling - don't allow it to stop
        let has_cells_above_grid = shape_cells.iter().any(|&(_, y)| y < 0);
        if has_cells_above_grid {
            return true; // Always allow movement if any cells are above grid
        }
        
        // Check if any cell would move into an occupied cell or out of bounds
        for &(cell_x, cell_y) in shape_cells {
            let new_y = cell_y + 1;
            
            // Check if at bottom of grid
            if new_y >= self.height as i32 {
                println!("can_move_down: false - at bottom (new_y={} >= height={})", new_y, self.height);
                return false;
            }
            
            // Check if the cell below is occupied
            if self.occupied_cells.contains_key(&(cell_x, new_y)) {
                println!("can_move_down: false - cell ({}, {}) is occupied", cell_x, new_y);
                return false;
            }
        }
        true
    }

    pub fn update(&mut self, _input: &Input, _fixed_delta: f32) {
        // Blank for now
    }

    pub fn draw(&mut self, gfx: &mut Graphics, _alpha: f32) {
        let grid_width = self.width as f32 * self.cell_size;
        let visible_grid_height = self.visible_height as f32 * self.cell_size;

        // Draw green border around only the visible grid area
        const BORDER_THICKNESS: f32 = 2.0;
        let green = Color::GREEN;

        // Top border (of visible area)
        gfx.rect()
            .size(vec2(grid_width, BORDER_THICKNESS))
            .at(self.visible_position)
            .color(green);

        // Bottom border (of visible area)
        gfx.rect()
            .size(vec2(grid_width, BORDER_THICKNESS))
            .at(vec2(self.visible_position.x, self.visible_position.y + visible_grid_height - BORDER_THICKNESS))
            .color(green);

        // Left border (of visible area)
        gfx.rect()
            .size(vec2(BORDER_THICKNESS, visible_grid_height))
            .at(self.visible_position)
            .color(green);

        // Right border (of visible area)
        gfx.rect()
            .size(vec2(BORDER_THICKNESS, visible_grid_height))
            .at(vec2(self.visible_position.x + grid_width - BORDER_THICKNESS, self.visible_position.y))
            .color(green);

        // Draw red squares for each node (debug)
        // let red = Color::RED;
        // for node in &self.rows {
        //     gfx.rect()
        //         .size(node.size)
        //         .at(node.position)
        //         .color(red);
        // }
    }

    /// Draw all occupied cells with their stored colors
    pub fn draw_occupied_cells(&self, gfx: &mut Graphics, _alpha: f32) {
        const BORDER_WIDTH: f32 = 1.0;
        const BORDER_COLOR: Color = Color::new([0.0, 0.0, 0.0, 1.0]); // Black border
        
        let cell_size_vec = vec2(self.cell_size, self.cell_size);
        
        for ((cell_x, cell_y), color) in &self.occupied_cells {
            let world_pos = vec2(
                self.position.x + *cell_x as f32 * self.cell_size,
                self.position.y + *cell_y as f32 * self.cell_size,
            );
            
            // Draw border (larger rectangle)
            gfx.rect()
                .size(cell_size_vec)
                .at(world_pos)
                .color(BORDER_COLOR);
            
            // Draw fill (smaller rectangle, inset by border width)
            let fill_size = vec2(self.cell_size - BORDER_WIDTH * 2.0, self.cell_size - BORDER_WIDTH * 2.0);
            let fill_pos = world_pos + vec2(BORDER_WIDTH, BORDER_WIDTH);
            gfx.rect()
                .size(fill_size)
                .at(fill_pos)
                .color(*color);
        }
    }

    /// Clear completed lines and shift cells above down
    /// Returns the number of lines cleared
    pub fn clear_completed_lines(&mut self) -> usize {
        const SPAWN_ROWS: usize = 4;
        let mut rows_to_clear = Vec::new();
        
        // Find full rows (in visible area, not spawn area)
        for row_y in SPAWN_ROWS..self.height {
            let mut row_full = true;
            for col_x in 0..self.width {
                if !self.occupied_cells.contains_key(&(col_x as i32, row_y as i32)) {
                    row_full = false;
                    break;
                }
            }
            if row_full {
                rows_to_clear.push(row_y);
            }
        }
        
        if rows_to_clear.is_empty() {
            return 0;
        }
        
        // Remove all cells in cleared rows
        for &row_y in &rows_to_clear {
            for col_x in 0..self.width {
                self.occupied_cells.remove(&(col_x as i32, row_y as i32));
            }
        }
        
        // Shift all cells above cleared rows down
        // For each cell above the highest cleared row, count how many cleared rows are below it
        let highest_cleared_row = *rows_to_clear.iter().max().unwrap();
        
        // Collect all cells that need to be moved (above the highest cleared row)
        let cells_to_move: Vec<((i32, i32), Color)> = self.occupied_cells
            .iter()
            .filter(|((_, y), _)| (*y as usize) < highest_cleared_row)
            .map(|(&pos, &color)| (pos, color))
            .collect();
        
        // Remove old positions and insert at new positions (shifted down)
        for ((x, y), color) in cells_to_move {
            self.occupied_cells.remove(&(x, y));
            // Count how many cleared rows are below this cell
            let cleared_rows_below = rows_to_clear.iter().filter(|&&r| r > y as usize).count();
            let new_y = y + cleared_rows_below as i32;
            self.occupied_cells.insert((x, new_y), color);
        }
        
        rows_to_clear.len()
    }
}