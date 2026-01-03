# CargoTap

A modern typing game built with Rust and Vulkan, designed to help you practice typing code with real-time feedback and visual rendering.

## Features

- **Typing Game Engine**: Interactive code typing game with real-time feedback
- **Configuration System**: Comprehensive TOML-based configuration for all settings
- **Code State Management**: Sophisticated tracking of typed vs. remaining code
- **Vulkan-based Rendering**: High-performance graphics rendering using the Vulkan API
- **Font Rendering**: Support for TrueType fonts with glyph analysis and positioning
- **Colored Text System**: Per-character color support for syntax highlighting and visual effects
- **Multi-language Support**: Handles both ASCII and Unicode characters (including Cyrillic)
- **Progress Tracking**: Real-time progress monitoring and statistics
- **Backspace Support**: Ability to correct mistakes and move characters back
- **Command-line Demo**: Interactive terminal-based demo mode
- **Debug Options**: Extensive debugging and logging configuration

## Architecture

The application consists of several key components:

- **Config**: TOML-based configuration system for all application settings
- **CodeState**: Manages the state of code being typed, tracking `current_code` (remaining) and `printed_code` (typed)
- **VulkanRenderer**: Core graphics engine handling Vulkan initialization, device management, and rendering pipeline
- **TextSystem**: Font loading, glyph rasterization, and text layout management with per-character color support
- **ColoredText System**: Advanced text rendering with individual character colors for syntax highlighting
- **InputHandler**: Advanced input processing with character-by-character validation
- **Demo System**: Both graphical and command-line demo modes

## Dependencies

- `vulkano`: Vulkan API bindings for Rust
- `winit`: Cross-platform window creation and event handling
- `ab_glyph`: Font loading and glyph rasterization
- `bytemuck`: Safe transmutation between data types
- `anyhow`: Error handling utilities
- `log` & `simple_logger`: Logging infrastructure
- `serde` & `toml`: Configuration file serialization

## Font Support

The application includes JetBrains Mono font for code display, providing:
- Monospace character spacing
- Clear distinction between similar characters
- Support for programming ligatures
- Unicode character support

## Usage

### Generate Configuration File (First Time)
```bash
cargo run gen-config
```

This creates a `config.toml` file with all available settings and documentation.

### Graphical Mode (Default)
```bash
cargo run
```

### Command-line Demo Mode
```bash
cargo run demo
```

## Configuration

CargoTap uses a `config.toml` file for configuration. See [CONFIG.md](CONFIG.md) for detailed documentation.

### Quick Configuration

Edit `config.toml` to customize:
- **Window settings**: Size, title, FPS
- **Text rendering**: Font, size, position, colors
- **Gameplay**: Custom code files, backspace, difficulty
- **Debug options**: Logging levels, performance metrics
- **Color scheme**: All colors including syntax highlighting

Example configuration:
```toml
[text]
font_size = 80.0
position_x = 50.0
position_y = 100.0

[gameplay]
custom_code_path = "my_code.rs"
allow_backspace = true

[debug]
log_level = "debug"
```

For complete documentation, see [CONFIG.md](CONFIG.md).

### Graphical Mode Features:
1. Initialize the Vulkan rendering engine
2. Load the JetBrains Mono font from the `fonts/` directory
3. Load demo code for typing practice
4. Track your typing progress in real-time
5. Provide visual feedback for correct/incorrect keystrokes
6. Display progress statistics and completion status

### Demo Mode Features:
- Interactive command-line typing practice
- Character-by-character validation
- Backspace support (type 'b' to backspace)
- Progress tracking and statistics
- Visual representation of typed vs. remaining code

## Project Structure

```
CargoTap/
├── src/
│   ├── main.rs              # Application entry point and game logic
│   ├── config.rs            # Configuration system
│   ├── code_state.rs        # Code state management
│   ├── demo_code_state.rs   # Command-line demo
│   ├── renderer.rs          # Vulkan rendering engine
│   ├── text.rs              # Text rendering system with colored text support
│   ├── input.rs             # Enhanced input handling
│   ├── demo_code.rs         # Sample code for typing practice
│   └── examples/
│       ├── mod.rs           # Examples module
│       └── colored_text_demo.rs  # Colored text demonstrations
├── fonts/
│   └── JetBrainsMono-Light.ttf  # Font file
├── config.toml.example      # Example configuration file
├── CONFIG.md                # Configuration documentation
└── Cargo.toml               # Project dependencies
```

## Technical Details

### Core Game Engine
- **CodeState Structure**: Manages `current_code` (remaining) and `printed_code` (typed)
- **Input Validation**: Character-by-character comparison with expected input
- **Progress Tracking**: Real-time calculation of typing progress and statistics
- **Error Handling**: Graceful handling of incorrect keystrokes with feedback

### Graphics & Rendering
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

## CodeState API

The `CodeState` struct provides the following methods:

```rust
// Create new code state with initial code
let mut code_state = CodeState::new("fn main() {}".to_string());

// Type the next character (moves from current_code to printed_code)
if let Some(ch) = code_state.type_character() {
    println!("Typed: {}", ch);
}

// Undo last character (moves from printed_code back to current_code)
if let Some(ch) = code_state.backspace() {
    println!("Moved back: {}", ch);
}

// Check progress
println!("Progress: {:.1}%", code_state.get_progress() * 100.0);
println!("Complete: {}", code_state.is_complete());

// Get next character to type
if let Some(next) = code_state.peek_next_character() {
    println!("Next: {}", next);
}
```

## Future Enhancements

- **Typing Metrics**: WPM calculation, accuracy statistics, and performance tracking
- **Difficulty Levels**: Multiple code complexity levels and programming languages
- **Enhanced Syntax Highlighting**: More sophisticated color schemes and themes
- **Leaderboards**: Score tracking and comparison features
- **Custom Code**: Ability to load and practice with custom code files
- **Multiple Languages**: Support for different programming languages
- **Advanced Feedback**: Real-time typing technique analysis and suggestions
- **Animation Effects**: Text animations and transitions using the colored text system

## Colored Text System

The application now features a sophisticated colored text rendering system that allows individual characters to have their own colors. This enables:

### Features
- **Per-character Colors**: Each character can have its own RGBA color value
- **Syntax Highlighting**: Automatic color coding for Rust keywords, types, and operators
- **Rainbow Effects**: Create colorful text patterns and gradients
- **Visual Feedback**: Enhanced user interface with colored elements

### API Usage

#### Basic ColoredText Creation
```rust
use crate::text::{ColoredText, ColoredChar};

// Create empty colored text
let mut colored_text = ColoredText::new();

// Add individual colored characters
colored_text.push('H', [1.0, 0.0, 0.0, 1.0]); // Red 'H'
colored_text.push('i', [0.0, 1.0, 0.0, 1.0]); // Green 'i'

// Add multiple characters with same color
colored_text.push_str("Hello", [0.0, 0.0, 1.0, 1.0]); // Blue "Hello"
```

#### Syntax Highlighting
```rust
use crate::examples::ColoredTextDemo;

let code = r#"fn main() {
    println!("Hello, World!");
}"#;

let highlighted = ColoredTextDemo::create_syntax_highlighted_rust(code);
```

#### Rainbow Text Effect
```rust
let rainbow = ColoredTextDemo::create_rainbow_text("RAINBOW TEXT");
```

#### Gradient Text Effect
```rust
let gradient = ColoredTextDemo::create_gradient_text(
    "Gradient Text",
    [1.0, 0.0, 0.0, 1.0], // Start: Red
    [0.0, 0.0, 1.0, 1.0], // End: Blue
);
```

### Color Format
Colors use RGBA format with values from 0.0 to 1.0:
- `[1.0, 0.0, 0.0, 1.0]` - Red, fully opaque
- `[0.0, 1.0, 0.0, 0.5]` - Green, 50% transparent
- `[0.5, 0.5, 0.5, 1.0]` - Gray, fully opaque

### Integration with TextSystem
```rust
// Update text system with colored text
text_system.update_text_with_settings(&colored_text)?;

// Backward compatibility with plain strings
text_system.update_text_with_settings_str("Plain text")?;
```

For detailed examples and advanced usage, see `COLORED_TEXT_USAGE.md` and the `examples/` directory.