// src/ui/input.rs

use crate::config::AppConfig;
use crate::midi::MidiPlayer;
use crate::renderer::overlay::PerformanceOverlay;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{Key, NamedKey};

/// Actions that can be triggered by input
#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    /// No action
    None,
    /// Toggle play/pause
    TogglePlayback,
    /// Increase playback speed
    IncreaseSpeed,
    /// Decrease playback speed
    DecreaseSpeed,
    /// Toggle performance overlay
    ToggleOverlay,
    /// Toggle slow mode
    ToggleSlowMode,
    /// Reset playback to start
    Reset,
    /// Request to open a file
    OpenFile,
    /// Toggle fullscreen
    ToggleFullscreen,
    /// Quit application
    Quit,
    /// Window resize
    Resize(u32, u32),
    /// File dropped
    FileDropped(std::path::PathBuf),
    /// Mouse moved
    MouseMoved(f64, f64),
    /// Mouse clicked
    MouseClicked(f64, f64),
}

/// Handles all input for the application
pub struct InputHandler {
    /// Whether a file is being dragged over the window
    file_hovered: bool,
    /// Whether fullscreen is enabled
    fullscreen: bool,
    /// Current mouse position
    mouse_x: f64,
    mouse_y: f64,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        InputHandler {
            file_hovered: false,
            fullscreen: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }

    /// Process a window event and return the resulting action
    pub fn process_event(&mut self, event: &WindowEvent) -> InputAction {
        match event {
            WindowEvent::KeyboardInput { event, .. } => self.process_key_event(event),
            
            WindowEvent::Resized(size) => InputAction::Resize(size.width, size.height),
            
            WindowEvent::DroppedFile(path) => {
                self.file_hovered = false;
                InputAction::FileDropped(path.clone())
            }
            
            WindowEvent::HoveredFile(_) => {
                self.file_hovered = true;
                InputAction::None
            }
            
            WindowEvent::HoveredFileCancelled => {
                self.file_hovered = false;
                InputAction::None
            }
            
            WindowEvent::CloseRequested => InputAction::Quit,

            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_x = position.x;
                self.mouse_y = position.y;
                InputAction::MouseMoved(position.x, position.y)
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if *state == ElementState::Pressed && *button == MouseButton::Left {
                    InputAction::MouseClicked(self.mouse_x, self.mouse_y)
                } else {
                    InputAction::None
                }
            }
            
            _ => InputAction::None,
        }
    }

    /// Process a keyboard event
    fn process_key_event(&mut self, event: &KeyEvent) -> InputAction {
        // Only handle key press events
        if event.state != ElementState::Pressed {
            return InputAction::None;
        }

        match &event.logical_key {
            Key::Named(NamedKey::Space) => InputAction::TogglePlayback,
            Key::Named(NamedKey::ArrowUp) => InputAction::IncreaseSpeed,
            Key::Named(NamedKey::ArrowDown) => InputAction::DecreaseSpeed,
            Key::Named(NamedKey::Escape) => InputAction::Quit,
            Key::Named(NamedKey::F11) => {
                self.fullscreen = !self.fullscreen;
                InputAction::ToggleFullscreen
            }
            Key::Character(c) => match c.as_str() {
                "p" | "P" => InputAction::ToggleOverlay,
                "s" | "S" => InputAction::ToggleSlowMode,
                "r" | "R" => InputAction::Reset,
                "o" | "O" => InputAction::OpenFile,
                "q" | "Q" => InputAction::Quit,
                _ => InputAction::None,
            },
            _ => InputAction::None,
        }
    }

    /// Apply an action to the application state
    pub fn apply_action(
        action: &InputAction,
        player: &mut MidiPlayer,
        overlay: &mut PerformanceOverlay,
        config: &mut AppConfig,
    ) {
        match action {
            InputAction::TogglePlayback => {
                player.toggle_playback();
                log::debug!("Playback: {}", if player.is_playing() { "Playing" } else { "Paused" });
            }
            InputAction::IncreaseSpeed => {
                player.increase_speed();
                log::debug!("Speed: {:.1}x", player.get_playback_speed());
            }
            InputAction::DecreaseSpeed => {
                player.decrease_speed();
                log::debug!("Speed: {:.1}x", player.get_playback_speed());
            }
            InputAction::ToggleOverlay => {
                overlay.toggle();
                config.performance.enable_performance_overlay = overlay.is_visible();
                log::debug!("Overlay: {}", if overlay.is_visible() { "Visible" } else { "Hidden" });
            }
            InputAction::ToggleSlowMode => {
                config.performance.slow_mode = !config.performance.slow_mode;
                if config.performance.slow_mode {
                    config.performance.frame_lock = Some(30);
                    config.quality.particle_density = 0.5;
                } else {
                    config.performance.frame_lock = None;
                    config.quality.particle_density = 1.0;
                }
                log::debug!("Slow mode: {}", if config.performance.slow_mode { "Enabled" } else { "Disabled" });
            }
            InputAction::Reset => {
                player.reset();
                log::debug!("Playback reset to start");
            }
            _ => {}
        }
    }

    /// Check if a file is being hovered over the window
    pub fn is_file_hovered(&self) -> bool {
        self.file_hovered
    }

    /// Check if fullscreen is enabled
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Get current mouse position
    pub fn mouse_position(&self) -> (f64, f64) {
        (self.mouse_x, self.mouse_y)
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
