# CargoTap

A modern typing game built with Rust and Vulkan, designed to help you practice typing code with real-time feedback and visual rendering.

## Features

- **Typing Game Engine**: Interactive code typing game with real-time feedback
- **Code State Management**: Sophisticated tracking of typed vs. remaining code
- **Vulkan-based Rendering**: High-performance graphics rendering using the Vulkan API
- **Font Rendering**: Support for TrueType fonts with glyph analysis and positioning
- **Multi-language Support**: Handles both ASCII and Unicode characters (including Cyrillic)
- **Progress Tracking**: Real-time progress monitoring and statistics
- **Backspace Support**: Ability to correct mistakes and move characters back
- **Command-line Demo**: Interactive terminal-based demo mode

## Architecture

The application consists of several key components:

- **CodeState**: Manages the state of code being typed, tracking `current_code` (remaining) and `printed_code` (typed)
- **VulkanRenderer**: Core graphics engine handling Vulkan initialization, device management, and rendering pipeline
- **TextSystem**: Font loading, glyph rasterization, and text layout management
- **InputHandler**: Advanced input processing with character-by-character validation
- **Demo System**: Both graphical and command-line demo modes

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

### Graphical Mode (Default)
```bash
cargo run
```

### Command-line Demo Mode
```bash
cargo run demo
```

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
│   ├── code_state.rs        # Code state management (NEW)
│   ├── demo_code_state.rs   # Command-line demo (NEW)
│   ├── renderer.rs          # Vulkan rendering engine
│   ├── text.rs              # Text rendering system
│   ├── input.rs             # Enhanced input handling
│   └── demo_code.rs         # Sample code for typing practice
├── fonts/
│   └── JetBrainsMono-Light.ttf  # Font file
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
- **Syntax Highlighting**: Visual distinction for different code elements
- **Leaderboards**: Score tracking and comparison features
- **Custom Code**: Ability to load and practice with custom code files
- **Multiple Languages**: Support for different programming languages
- **Advanced Feedback**: Real-time typing technique analysis and suggestions