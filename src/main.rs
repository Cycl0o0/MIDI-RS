// src/main.rs

use midi_rs::config::AppConfig;
use midi_rs::midi::{MidiParser, MidiPlayer, Note};
use midi_rs::performance::PerformanceMonitor;
use midi_rs::renderer::{NoteRenderer, PerformanceOverlay, RenderPipeline};
use midi_rs::ui::{InputAction, InputHandler};

use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const CONFIG_FILE: &str = "config.json";

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting MIDI-RS Black MIDI Visualizer");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let midi_file_path = args.get(1).cloned();

    if let Some(ref path) = midi_file_path {
        log::info!("MIDI file specified: {}", path);
    } else {
        log::info!("No MIDI file specified. Drag and drop a .mid file to load.");
    }

    // Print controls
    println!("\n=== MIDI-RS Controls ===");
    println!("Space      - Play/Pause");
    println!("Up/Down    - Adjust speed (0.5x - 2.0x)");
    println!("P          - Toggle performance overlay");
    println!("S          - Toggle slow mode (30 FPS)");
    println!("R          - Reset to start");
    println!("F11        - Toggle fullscreen");
    println!("Q/ESC      - Quit");
    println!("Drag & Drop - Load MIDI file");
    println!("========================\n");

    // Load or create config
    let config = AppConfig::load_from_file(CONFIG_FILE).unwrap_or_else(|_| {
        log::info!("No config file found, using defaults");
        let default = AppConfig::default();
        if let Err(e) = default.save_to_file(CONFIG_FILE) {
            log::warn!("Failed to save default config: {}", e);
        }
        default
    });

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create window
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("MIDI-RS Visualizer")
            .with_inner_size(winit::dpi::LogicalSize::new(
                config.display.width,
                config.display.height,
            ))
            .build(&event_loop)
            .unwrap(),
    );

    // Initialize render pipeline
    let mut pipeline = pollster::block_on(RenderPipeline::new(window.clone(), &config));

    // Initialize renderers
    let mut note_renderer = NoteRenderer::new(&config);
    let mut overlay = PerformanceOverlay::new(&config);

    // Initialize player and input handler
    let mut player = MidiPlayer::new();
    let mut input_handler = InputHandler::new();
    let mut monitor = PerformanceMonitor::new();

    // Notes storage
    let mut notes: Vec<Note> = Vec::new();

    // Load MIDI file if provided
    if let Some(path) = midi_file_path {
        load_midi_file(&path, &mut notes, &mut player);
    }

    // Timing
    let mut last_frame = Instant::now();
    let mut frame_accumulator = Duration::ZERO;

    // Config needs to be mutable for slow mode toggle
    let mut config = config;

    log::info!("Application initialized, entering event loop");

    // Run event loop
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => {
                    // Process input
                    let action = input_handler.process_event(&event);

                    match action {
                        InputAction::Quit => {
                            // Save config before exit
                            if let Err(e) = config.save_to_file(CONFIG_FILE) {
                                log::warn!("Failed to save config: {}", e);
                            }
                            elwt.exit();
                        }
                        InputAction::Resize(width, height) => {
                            pipeline.resize(winit::dpi::PhysicalSize::new(width, height));
                        }
                        InputAction::FileDropped(path) => {
                            if let Some(path_str) = path.to_str() {
                                if path_str.ends_with(".mid") || path_str.ends_with(".midi") {
                                    load_midi_file(path_str, &mut notes, &mut player);
                                } else {
                                    log::warn!("Dropped file is not a MIDI file: {}", path_str);
                                }
                            }
                        }
                        InputAction::ToggleFullscreen => {
                            if input_handler.is_fullscreen() {
                                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }
                        InputAction::OpenFile => {
                            log::info!("File open dialog not implemented - drag and drop a MIDI file instead");
                        }
                        _ => {
                            InputHandler::apply_action(&action, &mut player, &mut overlay, &mut config);
                        }
                    }

                    // Handle RedrawRequested
                    if matches!(event, WindowEvent::RedrawRequested) {
                        // Calculate delta time
                        let now = Instant::now();
                        let delta = now - last_frame;
                        last_frame = now;

                        // FPS limiting
                        frame_accumulator += delta;
                        let fps = config.performance.frame_lock.unwrap_or(config.display.target_fps);
                        let target = Duration::from_secs_f64(1.0 / fps as f64);

                        if frame_accumulator >= target {
                            frame_accumulator -= target;

                            // Update player
                            player.update(target.as_secs_f32());

                            // Render
                            let render_start = Instant::now();

                            match render_frame(
                                &pipeline,
                                &mut note_renderer,
                                &notes,
                                &player,
                                &config,
                            ) {
                                Ok(_) => {
                                    // Update performance monitor
                                    let render_duration = render_start.elapsed();
                                    monitor.frame_rendered(render_duration);

                                    // Update overlay
                                    overlay.update(
                                        &monitor,
                                        note_renderer.visible_count(),
                                        player.get_playback_speed(),
                                        player.is_playing(),
                                    );

                                    // Update window title with overlay info
                                    let title = overlay.get_title_text(&monitor, note_renderer.visible_count());
                                    window.set_title(&title);
                                }
                                Err(wgpu::SurfaceError::Lost) => {
                                    pipeline.resize(pipeline.size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("Out of GPU memory");
                                    elwt.exit();
                                }
                                Err(e) => {
                                    log::error!("Render error: {:?}", e);
                                }
                            }
                        }
                    }

                    // Request redraw for continuous rendering
                    window.request_redraw();
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();

    log::info!("MIDI-RS shut down cleanly");
}

/// Load a MIDI file
fn load_midi_file(path: &str, notes: &mut Vec<Note>, player: &mut MidiPlayer) {
    log::info!("Loading MIDI file: {}", path);

    let parser = MidiParser::new();
    match parser.parse_file(path) {
        Ok(parsed_notes) => {
            log::info!("Loaded {} notes", parsed_notes.len());
            *notes = parsed_notes;
            player.reset();
        }
        Err(e) => {
            log::error!("Failed to load MIDI file: {}", e);
        }
    }
}

/// Render a frame
fn render_frame(
    pipeline: &RenderPipeline,
    note_renderer: &mut NoteRenderer,
    notes: &[Note],
    player: &MidiPlayer,
    config: &AppConfig,
) -> Result<(), wgpu::SurfaceError> {
    // Update uniforms
    pipeline.update_uniforms(0.1, player.get_current_time());

    // Update note renderer
    note_renderer.update(pipeline, notes, player.get_current_time(), config);

    // Begin render
    let (output, mut encoder) = pipeline.begin_render(config.display.background_color)?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: config.display.background_color[0] as f64,
                        g: config.display.background_color[1] as f64,
                        b: config.display.background_color[2] as f64,
                        a: config.display.background_color[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Render notes
        note_renderer.render(&mut render_pass, pipeline);
    }

    // Submit and present
    pipeline.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
