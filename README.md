# CargoTap

A modern text rendering engine built with Rust and Vulkan, designed to display and analyze code with font rendering capabilities.

## Features

- **Vulkan-based Rendering**: High-performance graphics rendering using the Vulkan API
- **Font Rendering**: Support for TrueType fonts with glyph analysis and positioning
- **Multi-language Support**: Handles both ASCII and Unicode characters (including Cyrillic)
- **Code Display**: Specialized for displaying and analyzing source code
- **Real-time Text Processing**: Dynamic text loading and vertex buffer management

## Architecture

The application consists of several key components:

- **VulkanRenderer**: Core graphics engine handling Vulkan initialization, device management, and rendering pipeline
- **TextSystem**: Font loading, glyph rasterization, and text layout management
- **InputHandler**: User input processing and event handling
- **Demo Code**: Sample Rust code for demonstration purposes

## Dependencies

- `vulkano`: Vulkan API bindings for Rust
- `winit`: Cross-platform window creation and event handling
- `ab_glyph`: Font loading and glyph rasterization
- `bytemuck`: Safe transmutation between data types
- `anyhow`: Error handling utilities
- `log` & `simple_logger`: Logging infrastructure

## Font Support

The application includes JetBrains Mono font for code display, providing:
- Monospace character spacing
- Clear distinction between similar characters
- Support for programming ligatures
- Unicode character support

## Usage

```bash
cargo run
```

The application will:
1. Initialize the Vulkan rendering engine
2. Load the JetBrains Mono font from the `fonts/` directory
3. Parse and analyze the demo code from `demo_code.rs`
4. Display character positioning and bounds information
5. Open a graphics window for rendering

## Project Structure

```
CargoTap/
├── src/
│   ├── main.rs          # Application entry point
│   ├── renderer.rs      # Vulkan rendering engine
│   ├── text.rs          # Text rendering system
│   ├── input.rs         # Input handling
│   └── demo_code.rs     # Sample code for display
├── fonts/
│   └── JetBrainsMono-Light.ttf  # Font file
└── Cargo.toml           # Project dependencies
```

## Technical Details

- **Graphics API**: Vulkan 1.3 with dynamic rendering
- **Font Format**: TrueType (.ttf)
- **Character Encoding**: UTF-8 with full Unicode support
- **Rendering Pipeline**: Vertex/fragment shaders for text rendering
- **Memory Management**: Vulkan memory allocators for optimal performance

## Development

The codebase is structured for extensibility:
- Modular design with separate concerns
- Clean API boundaries between components
- Error handling with `Result` types
- Memory-safe Rust practices throughout

## Future Enhancements

- Syntax highlighting for different programming languages
- Interactive text editing capabilities
- Multiple font support
- Performance optimizations for large files
- Advanced text layout features