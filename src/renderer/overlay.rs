// src/renderer/overlay.rs

use crate::config::AppConfig;
use crate::performance::PerformanceMonitor;

/// Position for the performance overlay
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Performance overlay that displays FPS and other stats
pub struct PerformanceOverlay {
    /// Whether the overlay is visible
    visible: bool,
    /// Position of the overlay
    position: OverlayPosition,
    /// Cached display text lines
    lines: Vec<String>,
}

impl PerformanceOverlay {
    /// Create a new performance overlay
    pub fn new(config: &AppConfig) -> Self {
        PerformanceOverlay {
            visible: config.performance.enable_performance_overlay,
            position: OverlayPosition::TopLeft,
            lines: Vec::new(),
        }
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Set position
    pub fn set_position(&mut self, position: OverlayPosition) {
        self.position = position;
    }

    /// Get position
    pub fn position(&self) -> OverlayPosition {
        self.position
    }

    /// Update the overlay with current performance data
    pub fn update(
        &mut self,
        monitor: &PerformanceMonitor,
        note_count: u32,
        playback_speed: f32,
        is_playing: bool,
    ) {
        self.lines.clear();
        
        let fps = monitor.get_fps();
        let render_time = monitor.get_render_time();
        
        self.lines.push(format!("FPS: {:.1}", fps));
        self.lines.push(format!("Frame Time: {:.2}ms", render_time.as_secs_f32() * 1000.0));
        self.lines.push(format!("Notes: {}", note_count));
        self.lines.push(format!("Speed: {:.1}x", playback_speed));
        self.lines.push(format!("Status: {}", if is_playing { "Playing" } else { "Paused" }));
    }

    /// Get the lines to display
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }

    /// Render the overlay text to the window title (simple fallback)
    /// In a full implementation, this would render text using a texture atlas
    pub fn get_title_text(&self, monitor: &PerformanceMonitor, note_count: u32) -> String {
        if !self.visible {
            return "MIDI-RS Visualizer".to_string();
        }
        
        format!(
            "MIDI-RS | FPS: {:.0} | Notes: {} | Frame: {:.2}ms",
            monitor.get_fps(),
            note_count,
            monitor.get_render_time().as_secs_f32() * 1000.0
        )
    }
}

impl Default for PerformanceOverlay {
    fn default() -> Self {
        PerformanceOverlay {
            visible: false,
            position: OverlayPosition::TopLeft,
            lines: Vec::new(),
        }
    }
}
