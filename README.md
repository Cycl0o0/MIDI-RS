# MIDI-RS - Black MIDI Visualizer

A high-performance Black MIDI visualizer written in Rust, designed to handle millions of notes with GPU-accelerated rendering using WGPU.

## Features

- **GPU-Accelerated Rendering**: Uses WGPU for efficient rendering of millions of notes
- **Instanced Rendering**: Optimized for Black MIDI files with 100k+ simultaneous notes
- **Real-time Performance Overlay**: FPS counter, note count, and frame time display
- **Configurable Quality Settings**: Multiple presets for different hardware capabilities
- **Slow Mode**: 30 FPS lock for stable YouTube recording
- **Drag & Drop**: Easy MIDI file loading via drag and drop
- **Cross-Platform**: Supports Windows, macOS, and Linux

## Building

### Prerequisites

- Rust 1.70 or later
- A GPU with Vulkan, Metal, or DirectX 12 support

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

## Usage

### Running the Application

```bash
# Run without a MIDI file (use drag & drop to load)
cargo run --release

# Run with a MIDI file
cargo run --release -- path/to/your/file.mid
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `Space` | Play/Pause |
| `↑` | Increase playback speed (+0.1x) |
| `↓` | Decrease playback speed (-0.1x) |
| `P` | Toggle performance overlay |
| `S` | Toggle slow mode (30 FPS) |
| `R` | Reset playback to start |
| `F11` | Toggle fullscreen |
| `Q` / `ESC` | Quit |

### Loading MIDI Files

- **Drag & Drop**: Simply drag a `.mid` or `.midi` file onto the application window
- **Command Line**: Pass the file path as an argument when launching

## Configuration

The application uses a `config.json` file for persistent settings. If not present, a default configuration will be created.

### Quality Presets

| Preset | Max Notes | Particle Density | Use Case |
|--------|-----------|------------------|----------|
| Low | 250,000 | 0.3 | Integrated GPUs |
| Medium | 500,000 | 0.6 | Mid-range GPUs |
| High | 1,000,000 | 1.0 | Dedicated GPUs |
| Ultra | 2,000,000 | 1.0 | High-end GPUs |

### Slow Mode

Enable slow mode (`S` key) for:
- Consistent 30 FPS for YouTube recording
- Reduced particle effects
- Better frame time consistency

## Architecture

```
src/
├── main.rs           # Application entry point
├── lib.rs            # Library exports
├── config.rs         # Configuration management
├── performance.rs    # Performance monitoring
├── midi/
│   ├── mod.rs        # MIDI module exports
│   ├── note.rs       # Note data structure
│   ├── parser.rs     # MIDI file parsing
│   └── player.rs     # Playback control
├── renderer/
│   ├── mod.rs        # Renderer module exports
│   ├── pipeline.rs   # WGPU render pipeline
│   ├── note_renderer.rs  # Note instance rendering
│   └── overlay.rs    # Performance overlay
└── ui/
    ├── mod.rs        # UI module exports
    └── input.rs      # Input handling

assets/
└── shaders/
    ├── note.wgsl     # Note rendering shader
    └── overlay.wgsl  # Overlay shader
```

## Performance Tips

1. **Use Release Mode**: Always run with `--release` for best performance
2. **Reduce Quality**: Lower quality presets help with complex MIDI files
3. **Use Slow Mode**: For consistent recording, enable slow mode
4. **Close Other Applications**: Free up GPU resources for the visualizer
5. **Monitor GPU Temperature**: Black MIDI visualization can be GPU-intensive

## Troubleshooting

### Application Won't Start
- Ensure your GPU drivers are up to date
- Check that your GPU supports Vulkan, Metal, or DirectX 12

### Low FPS
- Enable slow mode for consistent frame times
- Reduce quality settings in `config.json`
- Try a smaller MIDI file

### MIDI File Won't Load
- Ensure the file has a `.mid` or `.midi` extension
- Check that the file is a valid Standard MIDI File (SMF)

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests (`cargo test`)
5. Run clippy (`cargo clippy`)
6. Submit a pull request

## Acknowledgments

- [midly](https://crates.io/crates/midly) - MIDI parsing library
- [wgpu](https://crates.io/crates/wgpu) - Cross-platform graphics API
- [winit](https://crates.io/crates/winit) - Window management
