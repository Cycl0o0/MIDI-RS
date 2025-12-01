// src/ui/controls.rs

use crate::config::AppConfig;
use crate::midi::MidiPlayer;
use crate::renderer::overlay::PerformanceOverlay;
use crate::renderer::pipeline::RenderPipeline;
use crate::renderer::note_renderer::NoteInstance;
use wgpu::util::DeviceExt;

/// UI Button definition
#[derive(Debug, Clone, Copy)]
pub struct Button {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub action: ButtonAction,
    pub is_hovered: bool,
    pub is_active: bool,
}

/// Actions that can be triggered by UI buttons
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonAction {
    PlayPause,
    Reset,
    IncreaseSpeed,
    DecreaseSpeed,
    ToggleSlowMode,
    ToggleOverlay,
}

impl Button {
    pub fn new(x: f32, y: f32, width: f32, height: f32, action: ButtonAction) -> Self {
        Button {
            x,
            y,
            width,
            height,
            action,
            is_hovered: false,
            is_active: false,
        }
    }

    /// Check if a point is inside the button
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Get the button color based on state
    pub fn get_color(&self) -> [f32; 4] {
        if self.is_active {
            match self.action {
                ButtonAction::PlayPause => [0.2, 0.7, 0.2, 0.9], // Green when playing
                ButtonAction::ToggleSlowMode => [0.7, 0.5, 0.2, 0.9], // Orange when slow mode
                ButtonAction::ToggleOverlay => [0.2, 0.5, 0.7, 0.9], // Blue when overlay visible
                _ => [0.4, 0.4, 0.6, 0.9],
            }
        } else if self.is_hovered {
            [0.5, 0.5, 0.5, 0.9] // Lighter when hovered
        } else {
            [0.3, 0.3, 0.35, 0.85] // Default dark gray
        }
    }
}

/// UI Controls manager
pub struct UIControls {
    buttons: Vec<Button>,
    instance_buffer: Option<wgpu::Buffer>,
    instance_count: u32,
    visible: bool,
    /// Screen size for coordinate conversion
    screen_width: f32,
    screen_height: f32,
}

impl UIControls {
    /// Create a new UI controls manager
    pub fn new(_config: &AppConfig) -> Self {
        let button_width = 0.04;
        let button_height = 0.035;
        let button_spacing = 0.01;
        let start_x = 0.02;
        let start_y = 0.94; // Near top of screen

        let buttons = vec![
            // Play/Pause button
            Button::new(start_x, start_y, button_width, button_height, ButtonAction::PlayPause),
            // Reset button
            Button::new(start_x + button_width + button_spacing, start_y, button_width, button_height, ButtonAction::Reset),
            // Decrease speed button
            Button::new(start_x + 2.0 * (button_width + button_spacing), start_y, button_width, button_height, ButtonAction::DecreaseSpeed),
            // Increase speed button  
            Button::new(start_x + 3.0 * (button_width + button_spacing), start_y, button_width, button_height, ButtonAction::IncreaseSpeed),
            // Slow mode toggle
            Button::new(start_x + 4.0 * (button_width + button_spacing), start_y, button_width, button_height, ButtonAction::ToggleSlowMode),
            // Overlay toggle
            Button::new(start_x + 5.0 * (button_width + button_spacing), start_y, button_width, button_height, ButtonAction::ToggleOverlay),
        ];

        UIControls {
            buttons,
            instance_buffer: None,
            instance_count: 0,
            visible: true,
            screen_width: 1920.0,
            screen_height: 1080.0,
        }
    }

    /// Update screen size for coordinate conversion
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    /// Toggle visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Handle mouse move to update hover states
    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        // Convert screen coordinates to normalized coordinates
        let norm_x = x / self.screen_width;
        let norm_y = 1.0 - (y / self.screen_height); // Flip Y coordinate

        for button in &mut self.buttons {
            button.is_hovered = button.contains(norm_x, norm_y);
        }
    }

    /// Handle mouse click and return the action if any button was clicked
    pub fn handle_mouse_click(&self, x: f32, y: f32) -> Option<ButtonAction> {
        if !self.visible {
            return None;
        }

        // Convert screen coordinates to normalized coordinates
        let norm_x = x / self.screen_width;
        let norm_y = 1.0 - (y / self.screen_height); // Flip Y coordinate

        for button in &self.buttons {
            if button.contains(norm_x, norm_y) {
                return Some(button.action);
            }
        }
        None
    }

    /// Update button active states based on current application state
    pub fn update_states(&mut self, is_playing: bool, slow_mode: bool, overlay_visible: bool) {
        for button in &mut self.buttons {
            button.is_active = match button.action {
                ButtonAction::PlayPause => is_playing,
                ButtonAction::ToggleSlowMode => slow_mode,
                ButtonAction::ToggleOverlay => overlay_visible,
                _ => false,
            };
        }
    }

    /// Apply a button action to the application state
    pub fn apply_action(
        action: ButtonAction,
        player: &mut MidiPlayer,
        overlay: &mut PerformanceOverlay,
        config: &mut AppConfig,
    ) {
        match action {
            ButtonAction::PlayPause => {
                player.toggle_playback();
                log::debug!("UI: Playback: {}", if player.is_playing() { "Playing" } else { "Paused" });
            }
            ButtonAction::Reset => {
                player.reset();
                log::debug!("UI: Playback reset to start");
            }
            ButtonAction::IncreaseSpeed => {
                player.increase_speed();
                log::debug!("UI: Speed: {:.1}x", player.get_playback_speed());
            }
            ButtonAction::DecreaseSpeed => {
                player.decrease_speed();
                log::debug!("UI: Speed: {:.1}x", player.get_playback_speed());
            }
            ButtonAction::ToggleSlowMode => {
                config.performance.slow_mode = !config.performance.slow_mode;
                if config.performance.slow_mode {
                    config.performance.frame_lock = Some(30);
                    config.quality.particle_density = 0.5;
                } else {
                    config.performance.frame_lock = None;
                    config.quality.particle_density = 1.0;
                }
                log::debug!("UI: Slow mode: {}", if config.performance.slow_mode { "Enabled" } else { "Disabled" });
            }
            ButtonAction::ToggleOverlay => {
                overlay.toggle();
                config.performance.enable_performance_overlay = overlay.is_visible();
                log::debug!("UI: Overlay: {}", if overlay.is_visible() { "Visible" } else { "Hidden" });
            }
        }
    }

    /// Update the instance buffer for rendering
    pub fn update(&mut self, pipeline: &RenderPipeline) {
        if !self.visible {
            self.instance_count = 0;
            return;
        }

        let instances: Vec<NoteInstance> = self.buttons
            .iter()
            .map(|button| NoteInstance {
                position: [button.x, button.y],
                size: [button.width, button.height],
                color: button.get_color(),
            })
            .collect();

        self.instance_count = instances.len() as u32;

        if self.instance_count == 0 {
            return;
        }

        let buffer_size = (self.instance_count as usize * std::mem::size_of::<NoteInstance>()) as u64;

        let needs_new_buffer = match &self.instance_buffer {
            None => true,
            Some(buffer) => buffer.size() < buffer_size,
        };

        if needs_new_buffer {
            self.instance_buffer = Some(pipeline.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("UI Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else if let Some(buffer) = &self.instance_buffer {
            pipeline.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&instances));
        }
    }

    /// Render the UI controls
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, pipeline: &'a RenderPipeline) {
        if !self.visible || self.instance_count == 0 {
            return;
        }

        if let Some(instance_buffer) = &self.instance_buffer {
            render_pass.set_pipeline(&pipeline.note_pipeline);
            render_pass.set_bind_group(0, &pipeline.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, pipeline.quad_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(pipeline.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..pipeline.quad_index_count(), 0, 0..self.instance_count);
        }
    }

    /// Get button labels for display (can be used for text rendering in the future)
    pub fn get_button_labels(&self) -> Vec<(&'static str, f32, f32)> {
        self.buttons
            .iter()
            .map(|button| {
                let label = match button.action {
                    ButtonAction::PlayPause => if button.is_active { "⏸" } else { "▶" },
                    ButtonAction::Reset => "⏮",
                    ButtonAction::IncreaseSpeed => "+",
                    ButtonAction::DecreaseSpeed => "-",
                    ButtonAction::ToggleSlowMode => "🐢",
                    ButtonAction::ToggleOverlay => "📊",
                };
                (label, button.x + button.width / 2.0, button.y + button.height / 2.0)
            })
            .collect()
    }
}

impl Default for UIControls {
    fn default() -> Self {
        Self::new(&AppConfig::default())
    }
}
