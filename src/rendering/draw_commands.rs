use crate::game_math::Vec2;
use std::collections::HashMap;

/// ID for identifying draw commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DrawCommandId(pub u32);

/// A draw command representing something to be drawn
#[derive(Debug, Clone, Copy)]
pub enum DrawCommand {
    Cube {
        position: Vec2,
        size: f32,
    },
    // Future: Circle, Sprite, Line, etc.
}

/// Collection of draw commands with IDs for lookup and removal
#[derive(Debug, Default)]
pub struct DrawCommandList {
    commands: HashMap<DrawCommandId, DrawCommand>,
    next_id: u32,
}

impl DrawCommandList {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a draw command and return its ID
    pub fn add(&mut self, command: DrawCommand) -> DrawCommandId {
        let id = DrawCommandId(self.next_id);
        self.next_id += 1;
        self.commands.insert(id, command);
        id
    }

    /// Remove a draw command by ID
    pub fn remove(&mut self, id: DrawCommandId) -> bool {
        self.commands.remove(&id).is_some()
    }

    /// Get a draw command by ID
    pub fn get(&self, id: DrawCommandId) -> Option<&DrawCommand> {
        self.commands.get(&id)
    }

    /// Get a mutable reference to a draw command by ID
    pub fn get_mut(&mut self, id: DrawCommandId) -> Option<&mut DrawCommand> {
        self.commands.get_mut(&id)
    }

    /// Check if a command exists
    pub fn contains(&self, id: DrawCommandId) -> bool {
        self.commands.contains_key(&id)
    }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Get the number of commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Iterate over all commands (for drawing)
    pub fn iter(&self) -> impl Iterator<Item = &DrawCommand> {
        self.commands.values()
    }

    /// Get all command IDs
    pub fn ids(&self) -> impl Iterator<Item = DrawCommandId> + '_ {
        self.commands.keys().copied()
    }
}
