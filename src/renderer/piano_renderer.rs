// src/renderer/piano_renderer.rs

use crate::config::AppConfig;
use crate::renderer::pipeline::RenderPipeline;
use crate::renderer::note_renderer::NoteInstance;
use wgpu::util::DeviceExt;

/// Renderer for the piano keyboard at the bottom of the screen
pub struct PianoRenderer {
    /// Instance buffer for piano key data
    instance_buffer: Option<wgpu::Buffer>,
    /// Number of piano key instances
    instance_count: u32,
    /// Keys currently being played (for lighting up)
    active_keys: [bool; 128],
    /// Height of the piano area (normalized 0-1)
    piano_height: f32,
}

impl PianoRenderer {
    /// Create a new piano renderer
    pub fn new(_config: &AppConfig) -> Self {
        PianoRenderer {
            instance_buffer: None,
            instance_count: 0,
            active_keys: [false; 128],
            piano_height: 0.12, // 12% of screen height
        }
    }

    /// Check if a key is black (sharp/flat)
    fn is_black_key(pitch: u8) -> bool {
        // Pattern within an octave: C, C#, D, D#, E, F, F#, G, G#, A, A#, B
        // Black keys are: C#(1), D#(3), F#(6), G#(8), A#(10)
        let note_in_octave = pitch % 12;
        matches!(note_in_octave, 1 | 3 | 6 | 8 | 10)
    }

    /// Set which keys are currently active (being played)
    pub fn set_active_keys(&mut self, active_pitches: &[u8]) {
        self.active_keys = [false; 128];
        for &pitch in active_pitches {
            if pitch < 128 {
                self.active_keys[pitch as usize] = true;
            }
        }
    }

    /// Update the piano keyboard buffer
    pub fn update(&mut self, pipeline: &RenderPipeline, active_pitches: &[u8]) {
        self.set_active_keys(active_pitches);

        // 128 MIDI notes total: 75 white keys + 53 black keys
        const WHITE_KEY_COUNT: usize = 75;
        const BLACK_KEY_COUNT: usize = 53;
        let mut instances: Vec<NoteInstance> = Vec::with_capacity(WHITE_KEY_COUNT + BLACK_KEY_COUNT);
        let key_width = 1.0 / 128.0;

        // Draw keys at their MIDI pitch positions
        // Each MIDI pitch maps directly to a horizontal position
        // This creates a linear mapping where all 128 keys fill the screen width
        
        // First draw white keys (they go behind black keys)
        for pitch in 0..128u8 {
            if Self::is_black_key(pitch) {
                continue;
            }
            let x = pitch as f32 / 128.0;
            let y = 0.0;
            let height = self.piano_height;

            // Color: white key, or lit up if active
            let color = if self.active_keys[pitch as usize] {
                [0.6, 0.8, 1.0, 1.0] // Light blue when active
            } else {
                [0.95, 0.95, 0.95, 1.0] // Off-white
            };

            instances.push(NoteInstance {
                position: [x, y],
                size: [key_width, height],
                color,
            });
        }

        // Then draw black keys (on top of white keys)
        for pitch in 0..128u8 {
            if !Self::is_black_key(pitch) {
                continue;
            }
            let x = pitch as f32 / 128.0;
            let y = self.piano_height * 0.35; // Black keys start 35% up the piano area
            let height = self.piano_height * 0.65; // Black keys are 65% of piano height

            // Color: dark gray/black, or lit up if active
            let color = if self.active_keys[pitch as usize] {
                [0.4, 0.6, 0.9, 1.0] // Light blue when active
            } else {
                [0.1, 0.1, 0.1, 1.0] // Dark gray/black
            };

            instances.push(NoteInstance {
                position: [x, y],
                size: [key_width * 0.7, height],
                color,
            });
        }

        self.instance_count = instances.len() as u32;

        if self.instance_count == 0 {
            return;
        }

        // Create or update instance buffer
        let buffer_size = (self.instance_count as usize * std::mem::size_of::<NoteInstance>()) as u64;

        let needs_new_buffer = match &self.instance_buffer {
            None => true,
            Some(buffer) => buffer.size() < buffer_size,
        };

        if needs_new_buffer {
            self.instance_buffer = Some(pipeline.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Piano Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else if let Some(buffer) = &self.instance_buffer {
            pipeline.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&instances));
        }
    }

    /// Render the piano keyboard
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, pipeline: &'a RenderPipeline) {
        if self.instance_count == 0 {
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

    /// Get the height of the piano area
    pub fn height(&self) -> f32 {
        self.piano_height
    }
}
