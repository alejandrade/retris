use egor::input::Input;
use egor::render::Graphics;
use crate::tetris_shape::TetrisShapeNode;
use crate::grid::Grid;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Game {
    active_piece: Option<TetrisShapeNode>,
    grid: Grid
}

impl Game {
    pub fn new() -> Self {
        const GRID_WIDTH_CELLS: usize = 10;
        const GRID_HEIGHT_CELLS: usize = 20;
        const MIN_PADDING: f32 = 40.0;

        Self {
            active_piece: None,
            grid: Grid::new(
                SCREEN_WIDTH as f32,
                SCREEN_HEIGHT as f32,
                GRID_WIDTH_CELLS,
                GRID_HEIGHT_CELLS,
                MIN_PADDING,
            )
        }
    }

    pub fn update(&mut self, input: &Input, fixed_delta: f32) {
        self.grid.update(input, fixed_delta);
        
        // Check if we need to spawn a new piece first (before updating)
        let needs_spawn = self.active_piece.is_none();
        if needs_spawn {
            self.spawn_new_piece();
        }
        
        // Update the active piece if it exists and isn't stopped
        if let Some(ref mut piece) = self.active_piece {
            if !piece.stopped {
                piece.update(input, fixed_delta, &mut self.grid);
            }
        }
        
        // Check if the piece stopped and transfer it to the grid
        if let Some(piece) = self.active_piece.take() {
            if piece.stopped {
                let cells_with_colors = piece.get_occupied_cells_with_color();
                println!("Piece stopped at ({}, {}), marking cells: {:?}", 
                         piece.cell_x, piece.cell_y, 
                         cells_with_colors.iter().map(|(x, y, _)| (*x, *y)).collect::<Vec<_>>());
                self.grid.mark_cells_occupied(&cells_with_colors);
                
                self.grid.clear_completed_lines();
            } else {
                self.active_piece = Some(piece);
            }
        }
    }
    

    fn spawn_new_piece(&mut self) {
        const SPAWN_VELOCITY: f32 = 1.0; // 1 cell per second
        const SPAWN_ROW: i32 = 0; // Spawn at the top row (in spawn area)
        
        // Calculate spawn position (centered horizontally, at top of grid)
        let grid_pos = self.grid.position();
        let cell_size = self.grid.cell_size();
        let grid_width = self.grid.width_cells();
        
        // Center the piece horizontally (assuming pieces are roughly centered)
        let spawn_cell_x = (grid_width / 2) as i32;
        let spawn_cell_y = SPAWN_ROW;
        
        let new_piece = TetrisShapeNode::new(
            SPAWN_VELOCITY,
            spawn_cell_x,
            spawn_cell_y,
            cell_size,
            grid_pos,
            grid_width,
            self.grid.height_cells(),
        );
        
        self.active_piece = Some(new_piece);
    }

    pub fn draw(&mut self, gfx: &mut Graphics, alpha: f32) {
        self.grid.draw(gfx, alpha);
        
        // Draw active piece if it exists and isn't stopped
        if let Some(ref mut piece) = self.active_piece {
            if !piece.stopped {
                piece.draw(gfx, alpha);
            }
        }
        
        // Draw occupied cells from grid (stopped pieces are now part of the grid)
        self.grid.draw_occupied_cells(gfx, alpha);
    }
}
