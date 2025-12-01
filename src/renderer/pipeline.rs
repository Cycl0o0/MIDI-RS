// src/renderer/pipeline.rs

use crate::config::AppConfig;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

/// Vertex data for rendering quads
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Unit quad vertices (0,0 to 1,1)
const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [1.0, 0.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [0.0, 1.0], tex_coords: [0.0, 1.0] },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

/// Main rendering pipeline for the MIDI visualizer
pub struct RenderPipeline {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub note_pipeline: wgpu::RenderPipeline,
    pub quad_vertex_buffer: wgpu::Buffer,
    pub quad_index_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
}

/// Uniform data passed to shaders
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub screen_size: [f32; 2],
    pub playhead_position: f32,
    pub time: f32,
}

impl RenderPipeline {
    /// Create a new render pipeline
    pub async fn new(window: Arc<Window>, _app_config: &AppConfig) -> Self {
        let size = window.inner_size();

        // Create the instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create the surface
        let surface = instance.create_surface(window).unwrap();

        // Request an adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::info!("Using adapter: {:?}", adapter.get_info().name);

        // Request a device
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: Some("MIDI-RS Device"),
                },
                None,
            )
            .await
            .unwrap();

        // Configure the surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Note Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/shaders/note.wgsl").into()),
        });

        // Create uniform buffer
        let uniforms = Uniforms {
            screen_size: [size.width as f32, size.height as f32],
            playhead_position: 0.1,
            time: 0.0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Uniform Bind Group Layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Note Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create the render pipeline
        let note_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Note Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                    crate::renderer::note_renderer::NoteInstance::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create quad buffers
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        RenderPipeline {
            surface,
            device,
            queue,
            config,
            size,
            note_pipeline,
            quad_vertex_buffer,
            quad_index_buffer,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    /// Handle window resize
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Update uniform buffer with new size
            let uniforms = Uniforms {
                screen_size: [new_size.width as f32, new_size.height as f32],
                playhead_position: 0.1,
                time: 0.0,
            };
            self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }
    }

    /// Update uniforms
    pub fn update_uniforms(&self, playhead_position: f32, time: f32) {
        let uniforms = Uniforms {
            screen_size: [self.size.width as f32, self.size.height as f32],
            playhead_position,
            time,
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Begin a render pass
    pub fn begin_render(&self, _background_color: [f32; 4]) -> Result<(wgpu::SurfaceTexture, wgpu::CommandEncoder), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        Ok((output, encoder))
    }

    /// Get the number of indices in the quad
    pub fn quad_index_count(&self) -> u32 {
        QUAD_INDICES.len() as u32
    }
}
