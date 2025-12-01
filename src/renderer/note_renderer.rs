// src/renderer/note_renderer.rs

use crate::config::AppConfig;
use crate::midi::Note;
use crate::renderer::pipeline::RenderPipeline;
use wgpu::util::DeviceExt;

/// Instance data for GPU rendering of notes
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NoteInstance {
    /// Position (x, y) in normalized screen coordinates
    pub position: [f32; 2],
    /// Size (width, height) in normalized screen coordinates
    pub size: [f32; 2],
    /// RGBA color
    pub color: [f32; 4],
}

impl NoteInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<NoteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Size
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Renderer for MIDI notes with instancing support for Black MIDI
pub struct NoteRenderer {
    /// Instance buffer for note data
    instance_buffer: Option<wgpu::Buffer>,
    /// Current number of visible instances
    instance_count: u32,
    /// Maximum buffer capacity
    max_instances: u32,
    /// Time window for visible notes (in seconds)
    time_window: f32,
    /// Note height in normalized coordinates
    note_height: f32,
    /// Batch size for rendering
    batch_size: u32,
}

impl NoteRenderer {
    /// Create a new note renderer
    pub fn new(config: &AppConfig) -> Self {
        NoteRenderer {
            instance_buffer: None,
            instance_count: 0,
            max_instances: config.quality.max_note_count,
            time_window: 5.0, // 5 seconds visible at once
            note_height: config.display.note_height / 127.0, // Normalize to pitch range
            batch_size: 10_000,
        }
    }

    /// Update visible notes based on current playback time
    pub fn update(
        &mut self,
        pipeline: &RenderPipeline,
        notes: &[Note],
        current_time: f32,
        config: &AppConfig,
    ) {
        // Collect visible notes with frustum culling
        let visible_notes: Vec<NoteInstance> = notes
            .iter()
            .filter(|note| {
                if !config.quality.frustum_culling {
                    return true;
                }
                note.is_visible(current_time, self.time_window)
            })
            .take(self.max_instances as usize)
            .map(|note| self.note_to_instance(note, current_time))
            .collect();

        self.instance_count = visible_notes.len() as u32;

        if self.instance_count == 0 {
            return;
        }

        // Create or update instance buffer
        let buffer_size = (self.instance_count as usize * std::mem::size_of::<NoteInstance>()) as u64;
        
        // Check if we need to recreate the buffer
        let needs_new_buffer = match &self.instance_buffer {
            None => true,
            Some(buffer) => buffer.size() < buffer_size,
        };

        if needs_new_buffer {
            // Create a new buffer with some extra capacity
            let _capacity = ((self.instance_count as f32 * 1.5) as u32).max(self.batch_size);
            self.instance_buffer = Some(pipeline.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Note Instance Buffer"),
                    contents: bytemuck::cast_slice(&visible_notes),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else if let Some(buffer) = &self.instance_buffer {
            // Update existing buffer
            pipeline.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&visible_notes));
        }
    }

    /// Convert a Note to NoteInstance for GPU rendering
    fn note_to_instance(&self, note: &Note, current_time: f32) -> NoteInstance {
        let x = note.get_x_position(current_time, self.time_window);
        let y = note.get_y_position();
        let width = note.get_width(self.time_window);
        let color = note.get_color();

        NoteInstance {
            position: [x, y],
            size: [width, self.note_height],
            color,
        }
    }

    /// Render all visible notes
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

    /// Get the current visible note count
    pub fn visible_count(&self) -> u32 {
        self.instance_count
    }

    /// Set the time window for visibility
    pub fn set_time_window(&mut self, seconds: f32) {
        self.time_window = seconds.max(1.0);
    }

    /// Get the time window
    pub fn time_window(&self) -> f32 {
        self.time_window
    }
}
