use std::collections::HashMap;

/// A generic table data structure for game grids
/// Stores data of type T at (column, row) positions
/// Uses a Vec of HashMaps for efficient row iteration while maintaining sparsity
pub struct GameTable<T> {
    columns: usize,
    rows: usize,
    // Each row is a HashMap of column -> value, allowing efficient row iteration
    data: Vec<HashMap<i32, T>>,
}

impl<T> GameTable<T> {
    /// Create a new GameTable with the specified number of columns and rows
    pub fn new(columns: usize, rows: usize) -> Self {
        let mut data = Vec::with_capacity(rows);
        for _ in 0..rows {
            data.push(HashMap::new());
        }
        Self {
            columns,
            rows,
            data,
        }
    }

    #[allow(dead_code)]
    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn is_valid_position(&self, col: i32, row: i32) -> bool {
        col >= 0 && col < self.columns as i32 && row >= 0 && row < self.rows as i32
    }
    #[allow(dead_code)]
    pub fn get(&self, col: i32, row: i32) -> Option<&T> {
        if !self.is_valid_position(col, row) {
            return None;
        }
        self.data.get(row as usize)?.get(&col)
    }
    #[allow(dead_code)]
    pub fn get_mut(&mut self, col: i32, row: i32) -> Option<&mut T> {
        if !self.is_valid_position(col, row) {
            return None;
        }
        self.data.get_mut(row as usize)?.get_mut(&col)
    }

    pub fn set(&mut self, col: i32, row: i32, value: T) -> Option<T> {
        if !self.is_valid_position(col, row) {
            return None;
        }
        self.data.get_mut(row as usize)?.insert(col, value)
    }

    pub fn has(&self, col: i32, row: i32) -> bool {
        if !self.is_valid_position(col, row) {
            return false;
        }
        self.data
            .get(row as usize)
            .map_or(false, |row_map| row_map.contains_key(&col))
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, col: i32, row: i32) -> Option<T> {
        if !self.is_valid_position(col, row) {
            return None;
        }
        self.data.get_mut(row as usize)?.remove(&col)
    }
    #[allow(dead_code)]

    pub fn clear(&mut self) {
        for row_map in &mut self.data {
            row_map.clear();
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, i32, &T)> {
        self.data.iter().enumerate().flat_map(|(row_idx, row_map)| {
            row_map
                .iter()
                .map(move |(col, value)| (*col, row_idx as i32, value))
        })
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (i32, i32, &mut T)> {
        self.data
            .iter_mut()
            .enumerate()
            .flat_map(|(row_idx, row_map)| {
                row_map
                    .iter_mut()
                    .map(move |(col, value)| (*col, row_idx as i32, value))
            })
    }

    #[allow(dead_code)]
    pub fn iter_row(&self, row: i32) -> impl Iterator<Item = (i32, &T)> {
        self.data
            .get(row as usize)
            .into_iter()
            .flat_map(|row_map| row_map.iter())
            .map(|(col, value)| (*col, value))
    }

    #[allow(dead_code)]
    pub fn iter_row_mut(&mut self, row: i32) -> Box<dyn Iterator<Item = (i32, &mut T)> + '_> {
        if self.is_valid_position(0, row) {
            Box::new(
                self.data[row as usize]
                    .iter_mut()
                    .map(|(col, value)| (*col, value)),
            )
        } else {
            Box::new(std::iter::empty())
        }
    }

    pub fn is_row_full(&self, row: i32) -> bool {
        if !self.is_valid_position(0, row) {
            return false;
        }
        self.data[row as usize].len() == self.columns
    }
    #[allow(dead_code)]
    pub fn clear_row(&mut self, row: i32) {
        if self.is_valid_position(0, row) {
            self.data[row as usize].clear();
        }
    }

    pub fn remove_row_and_shift_down(&mut self, row: i32) -> bool {
        if !self.is_valid_position(0, row) {
            return false;
        }

        // Clear the row we're removing first
        self.data[row as usize].clear();

        // Shift all rows above down by one
        // Process from the row above down to row 0
        for source_row in (0..row).rev() {
            // Collect all cells in this row
            let cells_in_row: Vec<(i32, T)> = self.data[source_row as usize].drain().collect();

            // Move cells down by one row (to source_row + 1)
            for (col, value) in cells_in_row {
                self.data[source_row as usize + 1].insert(col, value);
            }
        }

        true
    }
}

/// Manages the scoring system with EXPONENTIAL multipliers and level progression
/// 
/// ## Scoring Formula:
/// `points = 137 × rows_bonus × previous_multiplier × combo_multiplier × level_multiplier`
/// 
/// - **Base Points**: 137 (prime number for interesting scores)
/// 
/// - **Rows Bonus** (exponential - 2^n - 1):
///   - 1 row: 1x
///   - 2 rows: 3x
///   - 3 rows: 7x
///   - 4 rows (Tetris!): 15x 🔥
/// 
/// - **Previous Multiplier**: Carries over from last clear (starts at 1x)
///   - Clear a Tetris → next clear gets 15x multiplier!
///   - This compounds with the next clear's rows_bonus
/// 
/// - **Combo Multiplier** (exponential - 2^(combo-1)):
///   - 1st consecutive clear: 1x
///   - 2nd consecutive clear: 2x
///   - 3rd consecutive clear: 4x
///   - 4th consecutive clear: 8x
///   - 5th consecutive clear: 16x 💥
///   - Resets to 0 if a piece lands without clearing
/// 
/// - **Level Multiplier**: Rewards survival at high speeds!
///   - Level 0-4: 1x
///   - Level 5-9: 2x
///   - Level 10-14: 3x
///   - Level 15-19: 5x
///   - Level 20+: 8x 🚀
/// 
/// ## Example Scoring Sequences:
/// - Level 0, Clear 1 row: 137 × 1 × 1 × 1 × 1 = 137 points
/// - Level 0, Clear 4 rows (Tetris): 137 × 15 × 1 × 1 × 1 = 2,055 points
/// - Level 0, Clear 4 more rows (combo!): 137 × 15 × 15 × 2 × 1 = 61,650 points! 🤯
/// - Level 10, Clear 4 rows (Tetris): 137 × 15 × 1 × 1 × 3 = 6,165 points!
/// - Level 20, Clear 4 rows (Tetris): 137 × 15 × 1 × 1 × 8 = 16,440 points!! 💰
/// - Level 20, Clear 4 more (combo!): 137 × 15 × 15 × 2 × 8 = 493,200 points!!! 🔥💥🚀
/// 
/// - **Level progression**: Every 10 lines cleared increases level and drop speed
pub struct ScoreManager {
    score: u64,
    lines_cleared: u32,
    level: u32,
    current_multiplier: u32,  // Based on rows cleared in one drop
    combo_count: u32,         // Consecutive clears without missing
    high_score: u64,
    high_score_needs_sync: bool,  // True if high score needs to be uploaded to server
    base_points_per_row: u64,
    lines_per_level: u32,
}

impl ScoreManager {
    /// Create a new ScoreManager with default settings
    /// Base points per row is 137 by default (a prime number for interesting scores!)
    /// Level increases every 10 lines
    /// Loads high score from storage
    pub fn new() -> Self {
        use crate::storage::Storage;
        
        let game_data = Storage::load_game_data();
        println!("Loaded high score from storage: {}", game_data.high_score);
        
        Self {
            score: 0,
            lines_cleared: 0,
            level: 0,
            current_multiplier: 1,
            combo_count: 0,
            high_score: game_data.high_score,
            high_score_needs_sync: false,
            base_points_per_row: 137, // Prime number for more interesting scores
            lines_per_level: 10,
        }
    }


    /// Get the current score
    pub fn score(&self) -> u64 {
        self.score
    }

    /// Get the current level
    pub fn level(&self) -> u32 {
        self.level
    }

    /// Get total lines cleared
    pub fn lines_cleared(&self) -> u32 {
        self.lines_cleared
    }

    /// Get the current multiplier
    pub fn multiplier(&self) -> u32 {
        self.current_multiplier
    }

    /// Get the current combo count (consecutive clears without a break)
    pub fn combo_count(&self) -> u32 {
        self.combo_count
    }

    /// Get the high score
    pub fn high_score(&self) -> u64 {
        self.high_score
    }

    /// Call this when rows are cleared
    /// Returns the points awarded for this clear
    pub fn on_rows_cleared(&mut self, rows_cleared: u32) -> u64 {
        if rows_cleared == 0 {
            return 0;
        }

        // Capture current level BEFORE updating (for scoring)
        let level_at_clear = self.level;

        // Update lines and check for level up
        let old_level = self.level;
        self.lines_cleared += rows_cleared;
        self.level = self.lines_cleared / self.lines_per_level;

        // Increment combo count
        self.combo_count += 1;

        // Calculate points with EXPONENTIAL multipliers:
        // 
        // 1. Base points per row (137 by default - a prime number)
        // 
        // 2. Rows bonus - exponential based on rows cleared at once:
        //    - 1 row: 1x (base)
        //    - 2 rows: 3x (1.5x per row)
        //    - 3 rows: 7x (2.3x per row)
        //    - 4 rows (Tetris!): 15x (3.75x per row!)
        let rows_bonus_multiplier = match rows_cleared {
            1 => 1,
            2 => 3,
            3 => 7,
            4 => 15,
            _ => (1 << rows_cleared) - 1, // 2^n - 1 for higher
        };
        
        // 3. Previous clear multiplier (from last drop)
        let previous_multiplier = self.current_multiplier as u64;
        
        // 4. Combo multiplier - gets exponential with chain length:
        //    Combo 1: 1x
        //    Combo 2: 2x
        //    Combo 3: 4x
        //    Combo 4: 8x
        //    Combo 5: 16x (insane!)
        let combo_multiplier = 1u64 << (self.combo_count - 1); // 2^(combo-1)
        
        // 5. Level multiplier - reward players for surviving at high levels!
        //    Level 0-4: 1x
        //    Level 5-9: 2x
        //    Level 10-14: 3x
        //    Level 15-19: 5x
        //    Level 20+: 8x (crazy difficulty = crazy points!)
        let level_multiplier = match level_at_clear {
            0..=4 => 1,
            5..=9 => 2,
            10..=14 => 3,
            15..=19 => 5,
            _ => 8, // Level 20+
        };
        
        // Final formula: base × rows_bonus × previous_multiplier × combo_multiplier × level_multiplier
        let points = self.base_points_per_row 
            * rows_bonus_multiplier 
            * previous_multiplier 
            * combo_multiplier
            * level_multiplier;
        
        self.score += points;

        // Check if this is a new high score and save immediately
        if self.score > self.high_score {
            self.high_score = self.score;
            self.high_score_needs_sync = true;
            self.save_high_score();
        }

        // Set the multiplier for the NEXT clear based on this clear
        // This creates insane compounding:
        // - Clear 4 rows -> 15x multiplier for next clear
        // - Clear 4 more rows -> 15x on THAT too!
        self.current_multiplier = rows_bonus_multiplier as u32;

        // If we leveled up, you could trigger a callback here
        // For now, the game can check if level changed by comparing before/after
        if self.level > old_level {
            // Level up occurred!
        }

        points
    }

    /// Call this when a piece lands without clearing any rows
    /// Resets the multiplier back to 1x and breaks the combo
    pub fn on_piece_landed_no_clear(&mut self) {
        self.current_multiplier = 1;
        self.combo_count = 0;
    }

    /// Save the current high score to storage
    /// This is called automatically when a new high score is achieved
    fn save_high_score(&self) {
        use crate::storage::{Storage, GameData};
        Storage::save_game_data(&GameData {
            high_score: self.high_score,
        });
        println!("💾 Saved new high score: {}", self.high_score);
    }

    /// Manually set the high score (useful when loading from server)
    pub fn set_high_score(&mut self, high_score: u64) {
        self.high_score = high_score;
        self.high_score_needs_sync = false;
    }

}

impl Default for ScoreManager {
    fn default() -> Self {
        Self::new()
    }
}
