use crate::game_data::GameTable;
use crate::retris_colors::*;
use egor::input::Input;
use egor::math::{Vec2, vec2};
use egor::render::{Color, Graphics};

/// Number of hidden rows above the visible playfield where pieces spawn
pub const SPAWN_ROWS: usize = 4;

/// Represents a cell that's cascading down during level transition
struct CascadingCell {
    col: i32,
    row: i32,
    color: Color,
    offset_y: f32,      // Vertical offset in pixels from normal position
    velocity: f32,      // Fall speed in pixels per second
}

pub struct Grid {
    position: Vec2,
    width: usize,
    height: usize,         // Total height including spawn area
    visible_height: usize, // Visible height on screen
    cell_size: f32,
    visible_position: Vec2, // Position of visible area (for drawing border)
    occupied_cells: GameTable<Color>, // Track which cells are occupied and their colors
    cascading_cells: Vec<CascadingCell>, // Cells that are animating during level transition
    is_cascading: bool,    // True when cascade animation is active
}

impl Grid {
    pub fn new(
        screen_width: f32,
        screen_height: f32,
        width_cells: usize,
        visible_height_cells: usize,
        min_padding: f32,
    ) -> Self {
        // Step 1: Calculate available space for the grid (screen minus padding on all sides)
        let available_width_pixels = screen_width - (min_padding * 2.0);
        let available_height_pixels = screen_height - (min_padding * 2.0);

        // Step 2: Calculate cell size - must fit both width and height constraints
        // Use the smaller value to ensure the grid fits in both dimensions
        let cell_size_from_width = available_width_pixels / width_cells as f32;
        let cell_size_from_height = available_height_pixels / visible_height_cells as f32;
        let cell_size_pixels = cell_size_from_width.min(cell_size_from_height).floor();

        // Step 3: Calculate grid dimensions in pixels
        let grid_width_pixels = width_cells as f32 * cell_size_pixels;
        let visible_grid_height_pixels = visible_height_cells as f32 * cell_size_pixels;

        // Step 4: Calculate total grid height (visible + spawn area)
        let total_height_cells = visible_height_cells + SPAWN_ROWS;

        // Step 5: Position the grid in world coordinates
        // Coordinate system has (0,0) at center of screen
        // visible_position is the top-left corner of the visible playfield area
        let visible_area_top_left = vec2(
            -grid_width_pixels / 2.0,          // Center horizontally
            -visible_grid_height_pixels / 2.0, // Center vertically
        );

        // total_grid_top_left is the top-left corner of the entire grid (including spawn area above)
        // It's positioned SPAWN_ROWS above the visible area
        let spawn_area_height_pixels = SPAWN_ROWS as f32 * cell_size_pixels;
        let total_grid_top_left = vec2(
            -grid_width_pixels / 2.0,                           // Same X as visible area
            visible_area_top_left.y - spawn_area_height_pixels, // Above visible area
        );

        Self {
            position: total_grid_top_left,
            width: width_cells,
            height: total_height_cells,
            visible_height: visible_height_cells,
            cell_size: cell_size_pixels,
            visible_position: visible_area_top_left,
            occupied_cells: GameTable::new(width_cells, total_height_cells),
            cascading_cells: Vec::new(),
            is_cascading: false,
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

    pub fn is_cell_occupied(&self, cell_x: i32, cell_y: i32) -> bool {
        // Check bounds
        if cell_x < 0 || cell_x >= self.width as i32 || cell_y < 0 || cell_y >= self.height as i32 {
            return true; // Out of bounds counts as occupied
        }
        self.occupied_cells.has(cell_x, cell_y)
    }

    pub fn mark_cells_occupied(&mut self, cells: &[(i32, i32, Color)]) {
        for &(x, y, color) in cells {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                self.occupied_cells.set(x, y, color);
            } else {
                println!(
                    "WARNING: Filtered out out-of-bounds cell: ({}, {}) - bounds: width={}, height={}",
                    x, y, self.width, self.height
                );
            }
        }
    }

    pub fn can_move_down(&self, shape_cells: &[(i32, i32)]) -> bool {
        let has_cells_above_grid = shape_cells.iter().any(|&(_, y)| y < 0);
        if has_cells_above_grid {
            return true; // Always allow movement if any cells are above grid
        }

        // Check if any cell would move into an occupied cell or out of bounds
        for &(cell_x, cell_y) in shape_cells {
            let new_y = cell_y + 1;

            // Check if at bottom of grid
            if new_y >= self.height as i32 {
                println!(
                    "can_move_down: false - at bottom (new_y={} >= height={})",
                    new_y, self.height
                );
                return false;
            }

            // Check if the cell below is occupied
            if self.occupied_cells.has(cell_x, new_y) {
                println!(
                    "can_move_down: false - cell ({}, {}) is occupied",
                    cell_x, new_y
                );
                return false;
            }
        }
        true
    }

    pub fn update(&mut self, _input: &Input, _fixed_delta: f32) {
        // Blank for now
    }

    pub fn draw(&mut self, gfx: &mut Graphics, alpha: f32) {
        let grid_width = self.width as f32 * self.cell_size;
        let visible_grid_height = self.visible_height as f32 * self.cell_size;

        // Draw green border around only the visible grid area
        const BORDER_THICKNESS: f32 = 2.0;

        // Top border (of visible area)
        gfx.rect()
            .size(vec2(grid_width, BORDER_THICKNESS))
            .at(self.visible_position)
            .color(COLOR_BORDER_GREEN);

        // Bottom border (of visible area)
        gfx.rect()
            .size(vec2(grid_width, BORDER_THICKNESS))
            .at(vec2(
                self.visible_position.x,
                self.visible_position.y + visible_grid_height - BORDER_THICKNESS,
            ))
            .color(COLOR_BORDER_GREEN);

        // Left border (of visible area)
        gfx.rect()
            .size(vec2(BORDER_THICKNESS, visible_grid_height))
            .at(self.visible_position)
            .color(COLOR_BORDER_GREEN);

        // Right border (of visible area)
        gfx.rect()
            .size(vec2(BORDER_THICKNESS, visible_grid_height))
            .at(vec2(
                self.visible_position.x + grid_width - BORDER_THICKNESS,
                self.visible_position.y,
            ))
            .color(COLOR_BORDER_GREEN);

        self.draw_occupied_cells(gfx, alpha);
    }

    /// Draw all occupied cells with their stored colors
    fn draw_occupied_cells(&self, gfx: &mut Graphics, _alpha: f32) {
        const BORDER_WIDTH: f32 = 1.0;

        let cell_size_vec = vec2(self.cell_size, self.cell_size);

        if self.is_cascading {
            // Draw cascading cells with animation offset
            for cell in &self.cascading_cells {
                let world_pos = vec2(
                    self.position.x + cell.col as f32 * self.cell_size,
                    self.position.y + cell.row as f32 * self.cell_size + cell.offset_y,
                );

                // Draw border (larger rectangle)
                gfx.rect()
                    .size(cell_size_vec)
                    .at(world_pos)
                    .color(COLOR_CELL_BORDER);

                // Draw fill (smaller rectangle, inset by border width)
                let fill_size = vec2(
                    self.cell_size - BORDER_WIDTH * 2.0,
                    self.cell_size - BORDER_WIDTH * 2.0,
                );
                let fill_pos = world_pos + vec2(BORDER_WIDTH, BORDER_WIDTH);
                gfx.rect().size(fill_size).at(fill_pos).color(cell.color);
            }
        } else {
            // Normal drawing of occupied cells
            for (cell_x, cell_y, color) in self.occupied_cells.iter() {
                let world_pos = vec2(
                    self.position.x + cell_x as f32 * self.cell_size,
                    self.position.y + cell_y as f32 * self.cell_size,
                );

                // Draw border (larger rectangle)
                gfx.rect()
                    .size(cell_size_vec)
                    .at(world_pos)
                    .color(COLOR_CELL_BORDER);

                // Draw fill (smaller rectangle, inset by border width)
                let fill_size = vec2(
                    self.cell_size - BORDER_WIDTH * 2.0,
                    self.cell_size - BORDER_WIDTH * 2.0,
                );
                let fill_pos = world_pos + vec2(BORDER_WIDTH, BORDER_WIDTH);
                gfx.rect().size(fill_size).at(fill_pos).color(*color);
            }
        }
    }

    /// Clear completed lines and shift cells above down
    /// Returns the number of lines cleared
    pub fn clear_completed_lines(&mut self) -> usize {
        let mut cleared_count = 0;
        let mut row_y = (self.height - 1) as i32;

        // Iterate from bottom to top, checking each row
        // After removing a row, check the same index again (it now contains what was above)
        while row_y >= SPAWN_ROWS as i32 {
            // Check if this row is complete
            if self.occupied_cells.is_row_full(row_y) {
                if self.occupied_cells.remove_row_and_shift_down(row_y) {
                    cleared_count += 1;
                    // Don't decrement row_y - check the same row again
                    // because the row above has shifted down into this position
                    continue;
                }
            }
            // Move to the row above
            row_y -= 1;
        }

        cleared_count
    }

    /// Start the cascade animation for level transition
    /// Each cell gets a different fall velocity based on its column for a cascading effect
    pub fn start_cascade_animation(&mut self) {
        self.is_cascading = true;
        self.cascading_cells.clear();

        // Convert all occupied cells to cascading cells
        for (col, row, color) in self.occupied_cells.iter() {
            // Give each column a different delay/velocity for cascade effect
            // Columns further to the right start falling later (lower velocity initially)
            let base_velocity = 800.0; // Base fall speed in pixels per second
            let column_delay_factor = col as f32 * 0.15; // Each column is slightly slower
            let velocity = base_velocity * (1.0 - column_delay_factor);
            
            self.cascading_cells.push(CascadingCell {
                col,
                row,
                color: *color,
                offset_y: 0.0,
                velocity,
            });
        }

        // Clear the occupied cells table since we've moved them to cascading
        self.occupied_cells.clear();
    }

    /// Update the cascade animation
    /// progress: 0.0 to 1.0 representing animation progress
    pub fn update_cascade_animation(&mut self, progress: f32) {
        if !self.is_cascading {
            return;
        }

        // Update each cascading cell's offset based on velocity and progress
        let visible_height_pixels = self.visible_height as f32 * self.cell_size;
        let drop_distance = visible_height_pixels + self.cell_size * 2.0; // Fall off screen
        
        for cell in &mut self.cascading_cells {
            // Calculate how far this cell should have fallen
            // Cells with higher velocity fall faster
            cell.offset_y = progress * drop_distance * (cell.velocity / 800.0);
        }
    }

    /// Clear the cascade animation and reset to normal gameplay
    pub fn clear_cascade_animation(&mut self) {
        self.is_cascading = false;
        self.cascading_cells.clear();
        // Grid is now empty and ready for next level
    }
}
