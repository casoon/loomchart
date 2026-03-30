use crate::drawings::drawing::Point;
use crate::drawings::{Drawing, DrawingStyle};
use serde::{Deserialize, Serialize};

/// Represents a reversible command that modifies the drawing state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Add a new drawing
    AddDrawing(Drawing),
    /// Remove a drawing (stores the drawing for undo)
    RemoveDrawing { id: String, drawing: Drawing },
    /// Update drawing points (stores old and new points)
    UpdateDrawingPoints {
        id: String,
        old_points: Vec<Point>,
        new_points: Vec<Point>,
    },
    /// Update drawing style (stores old and new style)
    UpdateDrawingStyle {
        id: String,
        old_style: DrawingStyle,
        new_style: DrawingStyle,
    },
}

impl Command {
    /// Returns the inverse of this command for undo operations
    pub fn inverse(&self) -> Command {
        match self {
            Command::AddDrawing(drawing) => Command::RemoveDrawing {
                id: drawing.id.clone(),
                drawing: drawing.clone(),
            },
            Command::RemoveDrawing { id: _, drawing } => Command::AddDrawing(drawing.clone()),
            Command::UpdateDrawingPoints {
                id,
                old_points,
                new_points,
            } => Command::UpdateDrawingPoints {
                id: id.clone(),
                old_points: new_points.clone(),
                new_points: old_points.clone(),
            },
            Command::UpdateDrawingStyle {
                id,
                old_style,
                new_style,
            } => Command::UpdateDrawingStyle {
                id: id.clone(),
                old_style: new_style.clone(),
                new_style: old_style.clone(),
            },
        }
    }
}

/// Manages command history for undo/redo functionality
#[derive(Debug, Default)]
pub struct CommandHistory {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
    max_history: usize,
}

impl CommandHistory {
    /// Create a new command history with default max size (100)
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }

    /// Create a new command history with custom max size
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Execute a command and add it to the history
    pub fn execute(&mut self, command: Command) {
        // Clear redo stack when a new command is executed
        self.redo_stack.clear();

        // Add to undo stack
        self.undo_stack.push(command);

        // Limit history size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last command and return it
    pub fn undo(&mut self) -> Option<Command> {
        if let Some(command) = self.undo_stack.pop() {
            let inverse = command.inverse();
            self.redo_stack.push(command);
            Some(inverse)
        } else {
            None
        }
    }

    /// Redo the last undone command and return it
    pub fn redo(&mut self) -> Option<Command> {
        if let Some(command) = self.redo_stack.pop() {
            self.undo_stack.push(command.clone());
            Some(command)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get the number of commands in undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of commands in redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawings::drawing::Point;
    use crate::drawings::DrawingType;

    #[test]
    fn test_command_inverse() {
        let drawing = Drawing::new(DrawingType::TrendLine, vec![]);

        let add = Command::AddDrawing(drawing.clone());
        let remove = add.inverse();

        match remove {
            Command::RemoveDrawing { id, .. } => assert_eq!(id, "test"),
            _ => panic!("Expected RemoveDrawing"),
        }
    }

    #[test]
    fn test_undo_redo() {
        let mut history = CommandHistory::new();
        let drawing = Drawing::new(DrawingType::TrendLine, vec![]);

        history.execute(Command::AddDrawing(drawing));

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undo_cmd = history.undo();
        assert!(undo_cmd.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let redo_cmd = history.redo();
        assert!(redo_cmd.is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_max_history() {
        let mut history = CommandHistory::with_max_history(3);
        let drawing = Drawing::new(DrawingType::TrendLine, vec![]);

        for i in 0..5 {
            let mut d = drawing.clone();
            d.id = format!("test{}", i);
            history.execute(Command::AddDrawing(d));
        }

        assert_eq!(history.undo_count(), 3);
    }
}
