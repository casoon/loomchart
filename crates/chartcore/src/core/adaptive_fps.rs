#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use js_sys;

#[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
mod js_sys {
    pub struct Date;
    impl Date {
        pub fn now() -> f64 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64
        }
    }
}

/// Configuration for adaptive FPS scheduling
#[derive(Debug, Clone)]
pub struct AdaptiveFPSConfig {
    /// Maximum FPS during active interaction (default: 60)
    pub max_fps: f64,
    /// Minimum FPS when rendering is needed (default: 10)
    pub min_fps: f64,
    /// FPS when idle (no interaction for >5s) (default: 1)
    pub idle_fps: f64,
    /// Reduce FPS when on battery power (default: true)
    pub battery_aware: bool,
    /// Automatically adjust target FPS based on performance (default: true)
    pub auto_adjust: bool,
}

impl Default for AdaptiveFPSConfig {
    fn default() -> Self {
        Self {
            max_fps: 60.0,
            min_fps: 10.0,
            idle_fps: 1.0,
            battery_aware: true,
            auto_adjust: true,
        }
    }
}

/// Statistics about rendering performance
#[derive(Debug, Clone)]
pub struct FPSStats {
    /// Current actual FPS (smoothed)
    pub current_fps: f64,
    /// Target FPS the scheduler is aiming for
    pub target_fps: f64,
    /// Average frame time in milliseconds
    pub avg_frame_time_ms: f64,
    /// Number of frames that took longer than target
    pub dropped_frames: usize,
    /// Time since last user interaction (ms)
    pub time_since_interaction: f64,
}

/// Complexity metrics for adaptive FPS adjustment
#[derive(Debug, Clone)]
pub struct RenderComplexity {
    /// Number of visible candles
    pub candle_count: usize,
    /// Number of active indicators
    pub indicator_count: usize,
    /// Number of drawing tools on chart
    pub drawing_count: usize,
    /// Number of visible panes
    pub pane_count: usize,
    /// Is user currently dragging/interacting
    pub is_animating: bool,
    /// Is crosshair active
    pub crosshair_active: bool,
    /// Time since last interaction (ms)
    pub time_since_interaction: f64,
}

/// Adaptive frame rate scheduler for battery-aware, performance-optimized rendering
pub struct AdaptiveFrameScheduler {
    config: AdaptiveFPSConfig,
    current_fps: f64,
    target_fps: f64,
    last_frame_time: f64,
    last_interaction_time: f64,
    frame_times: Vec<f64>,
    dropped_frames: usize,
    current_complexity: RenderComplexity,
}

impl AdaptiveFrameScheduler {
    /// Create a new adaptive frame scheduler with the given configuration
    pub fn new(config: AdaptiveFPSConfig) -> Self {
        let now = js_sys::Date::now();

        Self {
            target_fps: config.max_fps,
            current_fps: config.max_fps,
            config,
            last_frame_time: now,
            last_interaction_time: now,
            frame_times: Vec::with_capacity(60),
            dropped_frames: 0,
            current_complexity: RenderComplexity {
                candle_count: 0,
                indicator_count: 0,
                drawing_count: 0,
                pane_count: 1,
                is_animating: false,
                crosshair_active: false,
                time_since_interaction: 0.0,
            },
        }
    }

    /// Check if we should render this frame based on target FPS
    pub fn should_render(&self) -> bool {
        let now = js_sys::Date::now();
        let target_interval = 1000.0 / self.target_fps;

        (now - self.last_frame_time) >= target_interval
    }

    /// Start timing a frame (call before rendering)
    /// Returns the start timestamp for use in end_frame()
    pub fn start_frame(&mut self) -> f64 {
        js_sys::Date::now()
    }

    /// End timing a frame (call after rendering)
    /// Updates statistics and adjusts target FPS if needed
    pub fn end_frame(&mut self, start_time: f64) {
        let now = js_sys::Date::now();
        let frame_time = now - start_time;

        // Track frame times in rolling window (last 60 frames)
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        // Update current FPS with exponential smoothing
        if frame_time > 0.0 {
            let instant_fps = 1000.0 / frame_time;
            self.current_fps = self.current_fps * 0.9 + instant_fps * 0.1;
        }

        // Check if we dropped this frame
        let target_frame_time = 1000.0 / self.target_fps;
        if frame_time > target_frame_time * 1.5 {
            self.dropped_frames += 1;
        }

        // Auto-adjust target FPS based on performance and idle time
        if self.config.auto_adjust {
            self.auto_adjust_target();
        }

        self.last_frame_time = now;
    }

    /// Record a user interaction (mouse move, click, scroll, etc.)
    /// Boosts FPS to maximum for responsive interaction
    pub fn record_interaction(&mut self) {
        self.last_interaction_time = js_sys::Date::now();
        self.target_fps = self.config.max_fps;
    }

    /// Get time since last user interaction in milliseconds
    pub fn get_time_since_interaction(&self) -> f64 {
        js_sys::Date::now() - self.last_interaction_time
    }

    /// Update the render complexity metrics
    /// Used to adjust FPS based on scene complexity
    pub fn update_complexity(&mut self, complexity: RenderComplexity) {
        self.current_complexity = complexity;
    }

    /// Automatically adjust target FPS based on:
    /// - Idle time (reduce when no interaction)
    /// - Battery status (reduce on low battery)
    /// - Performance (reduce if dropping frames)
    /// - Complexity (reduce for complex scenes)
    fn auto_adjust_target(&mut self) {
        let time_since_interaction = self.get_time_since_interaction();

        // 1. Idle detection - reduce FPS when user is inactive
        let base_target = if time_since_interaction > 5000.0 {
            // Idle for 5+ seconds - very low FPS
            self.config.idle_fps
        } else if time_since_interaction > 1000.0 {
            // Idle for 1-5 seconds - minimum FPS
            self.config.min_fps
        } else {
            // Active interaction - maximum FPS
            self.config.max_fps
        };

        self.target_fps = base_target;

        // 2. Battery-aware adjustment
        if self.config.battery_aware {
            if let Some(battery) = get_battery_status() {
                // Reduce FPS on battery power
                if !battery.charging {
                    if battery.level < 0.2 {
                        // Critical battery - aggressive reduction
                        self.target_fps = self.target_fps.min(15.0);
                    } else if battery.level < 0.3 {
                        // Low battery - moderate reduction
                        self.target_fps = self.target_fps.min(30.0);
                    } else {
                        // On battery but okay - slight reduction
                        self.target_fps = self.target_fps.min(45.0);
                    }
                }
            }
        }

        // 3. Performance-based adjustment - reduce if we can't keep up
        if !self.frame_times.is_empty() {
            let avg_frame_time: f64 =
                self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
            let target_frame_time = 1000.0 / self.target_fps;

            // If we're consistently slow, reduce target
            if avg_frame_time > target_frame_time * 1.5 {
                self.target_fps = (self.target_fps * 0.9).max(self.config.min_fps);
            }

            // If we're doing well, we can increase (but not above base target)
            if avg_frame_time < target_frame_time * 0.7 {
                self.target_fps = (self.target_fps * 1.05).min(base_target);
            }
        }

        // 4. Complexity-based adjustment
        let complexity_factor = self.calculate_complexity_factor();
        if complexity_factor > 2.0 {
            // Very complex scene - reduce target
            self.target_fps = (self.target_fps * 0.8).max(self.config.min_fps);
        }

        // Clamp to valid range
        self.target_fps = self
            .target_fps
            .max(self.config.min_fps)
            .min(self.config.max_fps);
    }

    /// Calculate complexity factor (1.0 = baseline, >1.0 = more complex)
    fn calculate_complexity_factor(&self) -> f64 {
        let c = &self.current_complexity;

        let mut factor = 1.0;

        // More candles = more complex
        if c.candle_count > 500 {
            factor += 0.5;
        }
        if c.candle_count > 1000 {
            factor += 0.5;
        }

        // Each indicator adds complexity
        factor += (c.indicator_count as f64) * 0.3;

        // Each drawing adds complexity
        factor += (c.drawing_count as f64) * 0.2;

        // Each pane adds complexity
        factor += ((c.pane_count - 1) as f64) * 0.4;

        // Animation requires higher FPS, so increase complexity
        if c.is_animating {
            factor += 0.3;
        }

        factor
    }

    /// Get current statistics for monitoring/debugging
    pub fn get_stats(&self) -> FPSStats {
        let avg_frame_time = if self.frame_times.is_empty() {
            0.0
        } else {
            self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64
        };

        FPSStats {
            current_fps: self.current_fps,
            target_fps: self.target_fps,
            avg_frame_time_ms: avg_frame_time,
            dropped_frames: self.dropped_frames,
            time_since_interaction: self.get_time_since_interaction(),
        }
    }

    /// Reset statistics (useful when changing scenes or resetting state)
    pub fn reset_stats(&mut self) {
        self.frame_times.clear();
        self.dropped_frames = 0;
        self.current_fps = self.target_fps;
    }
}

/// Battery status information
#[derive(Debug, Clone)]
struct BatteryStatus {
    charging: bool,
    level: f64, // 0.0 to 1.0
}

/// Attempt to get battery status from browser API
/// Returns None if battery API is not available or fails
fn get_battery_status() -> Option<BatteryStatus> {
    // Try to access the Battery Status API
    // Note: This API is deprecated in some browsers, so it may not be available

    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    {
        use wasm_bindgen::JsCast;

        let window = web_sys::window()?;
        let _navigator = window.navigator();

        // Try to get battery via getBattery() promise
        // This is asynchronous, so we can't easily get it here
        // For now, return None - battery awareness can be added later
        // with proper async handling

        None
    }

    #[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
    {
        // Not running in WASM, no battery API
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_fps_creation() {
        let config = AdaptiveFPSConfig::default();
        let scheduler = AdaptiveFrameScheduler::new(config);

        assert_eq!(scheduler.target_fps, 60.0);
        assert_eq!(scheduler.current_fps, 60.0);
    }

    #[test]
    fn test_complexity_factor() {
        let config = AdaptiveFPSConfig::default();
        let mut scheduler = AdaptiveFrameScheduler::new(config);

        // Baseline complexity
        let complexity = RenderComplexity {
            candle_count: 100,
            indicator_count: 0,
            drawing_count: 0,
            pane_count: 1,
            is_animating: false,
            crosshair_active: false,
            time_since_interaction: 0.0,
        };
        scheduler.update_complexity(complexity);

        let factor = scheduler.calculate_complexity_factor();
        assert!(factor >= 1.0 && factor < 2.0);
    }

    #[test]
    fn test_high_complexity() {
        let config = AdaptiveFPSConfig::default();
        let mut scheduler = AdaptiveFrameScheduler::new(config);

        // High complexity scene
        let complexity = RenderComplexity {
            candle_count: 1500,
            indicator_count: 10,
            drawing_count: 20,
            pane_count: 4,
            is_animating: true,
            crosshair_active: true,
            time_since_interaction: 0.0,
        };
        scheduler.update_complexity(complexity);

        let factor = scheduler.calculate_complexity_factor();
        assert!(factor > 2.0);
    }
}
