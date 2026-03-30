//! Event System - Handle mouse/touch interactions (pan, zoom, click)

use super::chart_state::ChartState;

/// Mouse button enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Mouse event types
#[derive(Debug, Clone)]
pub enum MouseEvent {
    Down { x: f64, y: f64, button: MouseButton },
    Up { x: f64, y: f64, button: MouseButton },
    Move { x: f64, y: f64 },
    Wheel { x: f64, y: f64, delta_y: f64 },
    Leave,
    DoubleClick { x: f64, y: f64 },
}

/// Touch event types (for mobile)
#[derive(Debug, Clone)]
pub enum TouchEvent {
    Start { x: f64, y: f64 },
    Move { x: f64, y: f64 },
    End { x: f64, y: f64 },
    Cancel,
}

/// Keyboard event types
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    KeyDown { key: String },
    KeyUp { key: String },
}

/// Event handler for chart interactions
pub struct EventHandler {
    /// Track if we're currently dragging
    is_dragging: bool,
    /// Last mouse position during drag
    last_drag_x: f64,
    last_drag_y: f64,
    /// Track double-click timing
    last_click_time: f64,
    double_click_threshold: f64, // milliseconds
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            last_drag_x: 0.0,
            last_drag_y: 0.0,
            last_click_time: 0.0,
            double_click_threshold: 300.0,
        }
    }

    /// Handle mouse event and update chart state
    pub fn handle_mouse_event(&mut self, event: MouseEvent, state: &mut ChartState) {
        match event {
            MouseEvent::Down { x, y, button } => {
                self.handle_mouse_down(x, y, button, state);
            }
            MouseEvent::Up { x, y, button } => {
                self.handle_mouse_up(x, y, button, state);
            }
            MouseEvent::Move { x, y } => {
                self.handle_mouse_move(x, y, state);
            }
            MouseEvent::Wheel { x, y, delta_y } => {
                self.handle_wheel(x, y, delta_y, state);
            }
            MouseEvent::Leave => {
                self.handle_mouse_leave(state);
            }
            MouseEvent::DoubleClick { x: _, y: _ } => {
                self.handle_double_click(state);
            }
        }
    }

    fn handle_mouse_down(&mut self, x: f64, y: f64, button: MouseButton, state: &mut ChartState) {
        if button == MouseButton::Left {
            self.is_dragging = true;
            self.last_drag_x = x;
            self.last_drag_y = y;
            state.start_pan(x, y);
        }
    }

    fn handle_mouse_up(&mut self, _x: f64, _y: f64, button: MouseButton, state: &mut ChartState) {
        if button == MouseButton::Left {
            self.is_dragging = false;
            state.end_interaction();
        }
    }

    fn handle_mouse_move(&mut self, x: f64, y: f64, state: &mut ChartState) {
        if self.is_dragging {
            // Calculate delta from last position
            let delta_x = (x - self.last_drag_x) as i32;
            let delta_y = (y - self.last_drag_y) as i32;

            // Pan the chart
            state.pan(delta_x, delta_y);

            // Update last position
            self.last_drag_x = x;
            self.last_drag_y = y;
        } else {
            // Update crosshair position
            state.update_crosshair(x, y);
        }
    }

    fn handle_wheel(&mut self, x: f64, _y: f64, delta_y: f64, state: &mut ChartState) {
        // Zoom in/out based on wheel delta
        // Positive delta = zoom out, negative = zoom in
        let zoom_factor = if delta_y > 0.0 { 1.1 } else { 0.9 };

        // Zoom centered on mouse position
        state.zoom(zoom_factor, Some(x as u32));
    }

    fn handle_mouse_leave(&mut self, state: &mut ChartState) {
        self.is_dragging = false;
        state.hide_crosshair();
        state.end_interaction();
    }

    fn handle_double_click(&mut self, state: &mut ChartState) {
        // Reset view to fit all data
        state.fit_to_data();
    }

    /// Handle touch event (for mobile support)
    pub fn handle_touch_event(&mut self, event: TouchEvent, state: &mut ChartState) {
        match event {
            TouchEvent::Start { x, y } => {
                self.is_dragging = true;
                self.last_drag_x = x;
                self.last_drag_y = y;
                state.start_pan(x, y);
            }
            TouchEvent::Move { x, y } => {
                if self.is_dragging {
                    let delta_x = (x - self.last_drag_x) as i32;
                    let delta_y = (y - self.last_drag_y) as i32;

                    state.pan(delta_x, delta_y);

                    self.last_drag_x = x;
                    self.last_drag_y = y;
                }
            }
            TouchEvent::End { x: _, y: _ } | TouchEvent::Cancel => {
                self.is_dragging = false;
                state.end_interaction();
            }
        }
    }

    /// Handle keyboard event
    pub fn handle_keyboard_event(&mut self, event: KeyboardEvent, state: &mut ChartState) {
        match event {
            KeyboardEvent::KeyDown { key } => match key.as_str() {
                "ArrowLeft" => state.pan(50, 0),
                "ArrowRight" => state.pan(-50, 0),
                "ArrowUp" => state.pan(0, 50),
                "ArrowDown" => state.pan(0, -50),
                "+" | "=" => state.zoom(0.9, None),
                "-" | "_" => state.zoom(1.1, None),
                "Home" | "h" => state.fit_to_data(),
                _ => {}
            },
            KeyboardEvent::KeyUp { key: _ } => {
                // No action on key up for now
            }
        }
    }

    /// Check if we're detecting a double-click (based on timing)
    pub fn is_double_click(&mut self, current_time: f64) -> bool {
        let is_double = current_time - self.last_click_time < self.double_click_threshold;
        self.last_click_time = current_time;
        is_double
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Timeframe;

    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new();
        assert!(!handler.is_dragging);
    }

    #[test]
    fn test_mouse_drag() {
        let mut handler = EventHandler::new();
        let mut state = ChartState::new(800, 600, Timeframe::M5);

        // Start drag
        handler.handle_mouse_event(
            MouseEvent::Down {
                x: 100.0,
                y: 100.0,
                button: MouseButton::Left,
            },
            &mut state,
        );
        assert!(handler.is_dragging);

        // Move during drag
        handler.handle_mouse_event(MouseEvent::Move { x: 150.0, y: 100.0 }, &mut state);

        // End drag
        handler.handle_mouse_event(
            MouseEvent::Up {
                x: 150.0,
                y: 100.0,
                button: MouseButton::Left,
            },
            &mut state,
        );
        assert!(!handler.is_dragging);
    }

    #[test]
    fn test_crosshair_update() {
        let mut handler = EventHandler::new();
        let mut state = ChartState::new(800, 600, Timeframe::M5);

        handler.handle_mouse_event(MouseEvent::Move { x: 400.0, y: 300.0 }, &mut state);

        assert!(state.crosshair.visible);
        assert_eq!(state.crosshair.x, 400.0);
        assert_eq!(state.crosshair.y, 300.0);
    }

    #[test]
    fn test_mouse_leave() {
        let mut handler = EventHandler::new();
        let mut state = ChartState::new(800, 600, Timeframe::M5);

        // First show crosshair
        handler.handle_mouse_event(MouseEvent::Move { x: 400.0, y: 300.0 }, &mut state);
        assert!(state.crosshair.visible);

        // Then leave
        handler.handle_mouse_event(MouseEvent::Leave, &mut state);
        assert!(!state.crosshair.visible);
    }

    #[test]
    fn test_keyboard_navigation() {
        let mut handler = EventHandler::new();
        let mut state = ChartState::new(800, 600, Timeframe::M5);

        // Test arrow key panning
        handler.handle_keyboard_event(
            KeyboardEvent::KeyDown {
                key: "ArrowLeft".to_string(),
            },
            &mut state,
        );
        assert!(state.is_dirty());
    }
}
