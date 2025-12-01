// src/renderer/mod.rs

pub mod pipeline;
pub mod note_renderer;
pub mod overlay;
pub mod piano_renderer;

pub use pipeline::RenderPipeline;
pub use note_renderer::NoteRenderer;
pub use overlay::PerformanceOverlay;
pub use piano_renderer::PianoRenderer;