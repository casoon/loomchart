//! Drawing Manager
//! Phase 4: Task 7.2

use super::{Drawing, DrawingType, Point};
use crate::commands::{Command, CommandHistory};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn get_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Manages all drawings on the chart
pub struct DrawingManager {
    /// All drawings by ID
    drawings: HashMap<String, Drawing>,
    /// Currently active drawing (being created)
    active_drawing: Option<String>,
    /// Currently selected drawing
    selected_drawing: Option<String>,
    /// Command history for undo/redo
    history: CommandHistory,
}

impl DrawingManager {
    /// Create a new drawing manager
    pub fn new() -> Self {
        Self {
            drawings: HashMap::new(),
            active_drawing: None,
            selected_drawing: None,
            history: CommandHistory::new(),
        }
    }

    /// Start creating a new drawing
    pub fn start_drawing(&mut self, drawing_type: DrawingType, start_point: Point) -> String {
        let drawing = Drawing::new(drawing_type, vec![start_point]);
        let id = drawing.id.clone();

        self.drawings.insert(id.clone(), drawing);
        self.active_drawing = Some(id.clone());

        id
    }

    /// Update the active drawing with a new point
    pub fn update_active_drawing(&mut self, point: Point) -> Result<(), String> {
        let id = self.active_drawing.as_ref().ok_or("No active drawing")?;

        let drawing = self
            .drawings
            .get_mut(id)
            .ok_or("Active drawing not found")?;

        // For most drawings, we only need 2 points
        match drawing.drawing_type {
            DrawingType::TrendLine | DrawingType::Rectangle | DrawingType::FibonacciRetracement => {
                if drawing.points.len() == 1 {
                    drawing.points.push(point);
                } else {
                    drawing.points[1] = point;
                }
            }
            DrawingType::HorizontalLine | DrawingType::VerticalLine => {
                if drawing.points.len() == 1 {
                    drawing.points.push(point);
                } else {
                    drawing.points[1] = point;
                }
            }
        }

        drawing.updated_at = get_timestamp();
        Ok(())
    }

    /// Finalize the active drawing
    pub fn finalize_drawing(&mut self) -> Result<String, String> {
        let id = self.active_drawing.take().ok_or("No active drawing")?;

        // Validate drawing has enough points
        let drawing = self.drawings.get(&id).ok_or("Drawing not found")?;

        if !drawing.is_valid() {
            self.drawings.remove(&id);
            return Err("Not enough points".to_string());
        }

        // Add to command history
        self.history.execute(Command::AddDrawing(drawing.clone()));

        Ok(id)
    }

    /// Cancel the active drawing
    pub fn cancel_drawing(&mut self) {
        if let Some(id) = self.active_drawing.take() {
            self.drawings.remove(&id);
        }
    }

    /// Add a completed drawing
    pub fn add_drawing(&mut self, drawing: Drawing) -> String {
        let id = drawing.id.clone();
        self.drawings.insert(id.clone(), drawing);
        id
    }

    /// Remove a drawing
    pub fn remove_drawing(&mut self, id: &str) -> Result<Drawing, String> {
        // Clear selection if removing selected drawing
        if self.selected_drawing.as_deref() == Some(id) {
            self.selected_drawing = None;
        }

        let drawing = self
            .drawings
            .remove(id)
            .ok_or_else(|| "Drawing not found".to_string())?;

        // Add to command history
        self.history.execute(Command::RemoveDrawing {
            id: id.to_string(),
            drawing: drawing.clone(),
        });

        Ok(drawing)
    }

    /// Update drawing points
    pub fn update_drawing(&mut self, id: &str, points: Vec<Point>) -> Result<(), String> {
        let drawing = self.drawings.get_mut(id).ok_or("Drawing not found")?;

        if drawing.locked {
            return Err("Drawing is locked".to_string());
        }

        let old_points = drawing.points.clone();
        drawing.update_points(points.clone());

        // Add to command history
        self.history.execute(Command::UpdateDrawingPoints {
            id: id.to_string(),
            old_points,
            new_points: points,
        });

        Ok(())
    }

    /// Hit test: find drawing under cursor (screen coordinates)
    pub fn hit_test(&self, x: f64, y: f64, tolerance: f64) -> Option<String> {
        // Check all drawings, return first hit
        for (id, drawing) in &self.drawings {
            if !drawing.visible {
                continue;
            }

            if self.hit_test_drawing(drawing, x, y, tolerance) {
                return Some(id.clone());
            }
        }

        None
    }

    /// Hit test a single drawing
    fn hit_test_drawing(&self, drawing: &Drawing, x: f64, y: f64, tolerance: f64) -> bool {
        match drawing.drawing_type {
            DrawingType::TrendLine => self.hit_test_line(&drawing.points, x, y, tolerance),
            DrawingType::HorizontalLine => {
                if drawing.points.is_empty() {
                    return false;
                }
                (y - drawing.points[0].price).abs() < tolerance
            }
            DrawingType::VerticalLine => {
                if drawing.points.is_empty() {
                    return false;
                }
                (x - drawing.points[0].timestamp as f64).abs() < tolerance
            }
            DrawingType::Rectangle => self.hit_test_rectangle(&drawing.points, x, y, tolerance),
            DrawingType::FibonacciRetracement => {
                self.hit_test_fibonacci(&drawing.points, x, y, tolerance)
            }
        }
    }

    fn hit_test_line(&self, points: &[Point], x: f64, y: f64, tolerance: f64) -> bool {
        if points.len() < 2 {
            return false;
        }

        let p1 = &points[0];
        let p2 = &points[1];

        let distance = point_to_line_distance(
            x,
            y,
            p1.timestamp as f64,
            p1.price,
            p2.timestamp as f64,
            p2.price,
        );

        distance < tolerance
    }

    fn hit_test_rectangle(&self, points: &[Point], x: f64, y: f64, tolerance: f64) -> bool {
        if points.len() < 2 {
            return false;
        }

        let x1 = points[0].timestamp as f64;
        let y1 = points[0].price;
        let x2 = points[1].timestamp as f64;
        let y2 = points[1].price;

        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);

        // Check if point is on any edge
        let on_top = (y - max_y).abs() < tolerance && x >= min_x && x <= max_x;
        let on_bottom = (y - min_y).abs() < tolerance && x >= min_x && x <= max_x;
        let on_left = (x - min_x).abs() < tolerance && y >= min_y && y <= max_y;
        let on_right = (x - max_x).abs() < tolerance && y >= min_y && y <= max_y;

        on_top || on_bottom || on_left || on_right
    }

    fn hit_test_fibonacci(&self, points: &[Point], _x: f64, y: f64, tolerance: f64) -> bool {
        // Hit test on any of the fibonacci levels
        if points.len() < 2 {
            return false;
        }

        let levels = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];
        let y1 = points[0].price;
        let y2 = points[1].price;

        for level in levels {
            let level_y = y1 + (y2 - y1) * level;
            if (y - level_y).abs() < tolerance {
                return true;
            }
        }

        false
    }

    /// Select a drawing
    pub fn select(&mut self, id: &str) {
        self.selected_drawing = Some(id.to_string());
    }

    /// Deselect current drawing
    pub fn deselect(&mut self) {
        self.selected_drawing = None;
    }

    /// Get all drawings
    pub fn drawings(&self) -> &HashMap<String, Drawing> {
        &self.drawings
    }

    /// Get selected drawing
    pub fn selected(&self) -> Option<&Drawing> {
        self.selected_drawing
            .as_ref()
            .and_then(|id| self.drawings.get(id))
    }

    /// Get selected drawing ID
    pub fn selected_id(&self) -> Option<&str> {
        self.selected_drawing.as_deref()
    }

    /// Check if a drawing is selected
    pub fn is_selected(&self, id: &str) -> bool {
        self.selected_drawing.as_deref() == Some(id)
    }

    /// Undo the last command
    pub fn undo(&mut self) -> Result<(), String> {
        let command = self.history.undo().ok_or("Nothing to undo")?;
        self.apply_command(command)?;
        Ok(())
    }

    /// Redo the last undone command
    pub fn redo(&mut self) -> Result<(), String> {
        let command = self.history.redo().ok_or("Nothing to redo")?;
        self.apply_command(command)?;
        Ok(())
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Apply a command to the drawing state
    fn apply_command(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::AddDrawing(drawing) => {
                self.drawings.insert(drawing.id.clone(), drawing);
                Ok(())
            }
            Command::RemoveDrawing { id, .. } => {
                self.drawings.remove(&id);
                if self.selected_drawing.as_deref() == Some(&id) {
                    self.selected_drawing = None;
                }
                Ok(())
            }
            Command::UpdateDrawingPoints { id, new_points, .. } => {
                let drawing = self.drawings.get_mut(&id).ok_or("Drawing not found")?;
                drawing.points = new_points;
                Ok(())
            }
            Command::UpdateDrawingStyle { id, new_style, .. } => {
                let drawing = self.drawings.get_mut(&id).ok_or("Drawing not found")?;
                drawing.style = new_style;
                Ok(())
            }
        }
    }
}

impl Default for DrawingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate distance from point to line segment
fn point_to_line_distance(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;

    if dx == 0.0 && dy == 0.0 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }

    let t = ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy);
    let t = t.clamp(0.0, 1.0);

    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;

    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}
