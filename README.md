# CargoTap

A modern typing game built with Rust and Vulkan, designed to help you practice typing code with real-time feedback and visual rendering.

## Features

- **Timed Session State**: Practice typing in timed sessions with configurable duration (default 3 minutes)
- **Session Statistics & History**: Track typing speed (CPM/WPM), accuracy, errors, and time for each session with persistent storage
- **Statistics Dashboard**: View detailed statistics including all-time bests, recent performance, and progress trends (Ctrl+T / Cmd+T)
- **Performance Tracking**: Automatic tracking of accuracy percentage, typing errors, and improvement over time
- **Continuous Progress**: After a session ends, start a new one and continue from where you left off
- **Live Timer Display**: Real-time countdown timer showing remaining session time with live CPM updates
- **Typing Game Engine**: Interactive code typing game with real-time feedback
- **Configuration System**: Comprehensive TOML-based configuration for all settings
- **Code State Management**: Sophisticated tracking of typed vs. remaining code
- **Vulkan-based Rendering**: High-performance graphics rendering using the Vulkan API
- **Font Rendering**: Support for TrueType fonts with glyph analysis and positioning
- **Colored Text System**: Per-character color support for syntax highlighting and visual effects
- **Multi-language Support**: Handles both ASCII and Unicode characters (including Cyrillic)
- **Progress Tracking**: Real-time progress monitoring and statistics
- **Backspace Support**: Ability to correct mistakes and move characters back
- **Line Numbers**: IDE-style line numbers displayed in the left column with current line highlighting in bright color
- **Tab Whitespace Consumption**: Press Tab to skip all whitespace (spaces, tabs, newlines) until the next non-whitespace character
- **Untypeable Character Handling**: Auto-skip emoji, Arabic, CJK, and other characters that can't be typed on US keyboards, or manually skip with Ctrl+S / Cmd+S
- **Code Scrolling**: Navigate view through code using keyboard shortcuts (Command+J / Ctrl+J) - view-only, doesn't affect typing state
- **Command-line Demo**: Interactive terminal-based demo mode
- **Debug Options**: Extensive debugging and logging configuration

## Architecture

The application consists of several key components:

- **Config**: TOML-based configuration system for all application settings
- **CodeState**: Manages the state of code being typed, tracking `current_code` (remaining) and `printed_code` (typed)
- **SessionState**: Manages timed typing sessions with statistics tracking (CPM, WPM, accuracy, errors)
- **SessionHistory**: Persistent storage and analysis of completed sessions with performance trends
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

## Session-Based Typing Practice

CargoTap now includes a session system to help you practice typing in focused time blocks:

### How It Works

1. **Start Typing**: When you type your first character, a session begins automatically
2. **Timer Countdown**: The remaining time is displayed at the top of the screen
3. **Live Statistics**: See your current typing speed (CPM - Characters Per Minute) as you type
4. **Session Complete**: When time expires, you'll see detailed statistics:
   - Total time elapsed
   - Total characters typed
   - Characters per minute (CPM)
   - Words per minute (WPM) - calculated as CPM / 5
5. **Continue Practice**: Press **SPACE** to start a new session and continue typing from where you left off

### Configuration

Edit `config.toml` to customize session duration:

```toml
[gameplay]
session_duration_minutes = 3.0  # Default is 3 minutes
```

### Session Statistics

After each session, you'll see:
- **Time**: Total session duration
- **Characters**: Number of characters typed (start position → end position)
- **Speed**: CPM (Characters Per Minute) and WPM (Words Per Minute)

## Keyboard Shortcuts

- **SPACE**: Start a new typing session (when previous session is finished)
- **Ctrl+T** / **Cmd+T**: Toggle statistics dashboard to view session history and performance trends
- **Tab**: Consume all whitespace characters (spaces, tabs, newlines) until the next non-whitespace character
- **Ctrl+S** / **Cmd+S**: Skip the current character (useful for emoji, Arabic, or other untypeable characters)
- **Command+J** (macOS) / **Ctrl+J** (Windows/Linux): Scroll view down by configured number of lines (view-only - doesn't change typing state)
- **Backspace**: Undo last typed character (if enabled in config)
- **Escape**: Close statistics screen (if open) or quit the application
- **Command+W**: Quit the application

**Important**: Scrolling changes what you SEE, not what you've TYPED. Your typing position stays the same.

## Handling Untypeable Characters

CargoTap can automatically handle characters that cannot be typed on a standard US keyboard (emoji 🦀, Arabic ا, Chinese 中, etc.):

- **Auto-skip mode** (default: enabled): Automatically skips untypeable characters
- **Manual skip**: Press **Ctrl+S** or **Cmd+S** to skip any character
- **Transparent logging**: Shows which characters were skipped and why

For detailed information, see [UNTYPEABLE_CHARACTERS.md](UNTYPEABLE_CHARACTERS.md).

## Configuration

CargoTap uses a `config.toml` file for configuration.

### Configuration Options

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
scroll_lines = 5  # Lines to shift VIEW (not typing position)
session_duration_minutes = 5.0  # Duration for each typing session
auto_skip_untypeable = true  # Auto-skip emoji, Arabic, CJK, etc.
enable_manual_skip = true  # Allow Ctrl+S / Cmd+S to skip characters

[debug]
log_level = "debug"
```

Run `cargo run gen-config` to create a config file with all available options and inline documentation.

### Graphical Mode Features:
1. Initialize the Vulkan rendering engine
2. Load the JetBrains Mono font from the `fonts/` directory
3. Load demo code for typing practice
4. Track your typing progress in real-time
5. Provide visual feedback for correct/incorrect keystrokes
6. Display progress statistics and completion status

### Graphical Mode Session Flow:
1. Launch the application with `cargo run`
2. Start typing to begin your first session
3. Watch the timer count down at the top of the screen
4. See your typing speed (CPM) and accuracy update in real-time
5. When the session ends, review your statistics (CPM, WPM, accuracy, errors)
6. Press SPACE to start a new session and continue
7. Press **Ctrl+T** / **Cmd+T** to view your statistics dashboard with:
   - All-time performance statistics
   - Best performances (highest speed and accuracy)
   - Recent session performance (last 5 sessions)
   - Improvement trends and analysis
   - Individual session details

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
│   ├── session_state.rs     # Session timer and statistics tracking
│   ├── session_history.rs   # Session history storage and analysis
│   ├── progress_storage.rs  # File progress persistence
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

## SessionState API

The `SessionState` struct provides comprehensive session management:

```rust
// Create a session with 3 minute duration
let mut session = SessionState::new(3.0);

// Start the session at current position
session.start(current_position, file_path);

// Record typed characters and errors
session.record_char_typed();
session.record_backspace(); // Records an error

// Check if session is finished
if session.update(current_position) {
    // Session just finished
    if let Some(stats) = session.last_stats() {
        println!("CPM: {:.0}", stats.chars_per_minute);
        println!("WPM: {:.0}", stats.words_per_minute);
        println!("Accuracy: {:.1}%", stats.accuracy);
        println!("Errors: {}", stats.errors);
    }
}

// Start a new session continuing from current position
session.start_new_session(current_position, file_path);
```

## SessionHistory API

The `SessionHistory` struct provides persistent statistics storage and analysis:

```rust
// Create and load session history
let mut history = SessionHistory::default();
history.load()?;

// Add a completed session
history.add_session(session_stats);
history.save()?;

// Get recent sessions
let recent = history.get_recent_sessions(5);

// Get summary statistics
let summary = history.get_summary();
println!("Total sessions: {}", summary.total_sessions);
println!("Average CPM: {:.0}", summary.avg_cpm);
println!("Best WPM: {:.0}", summary.best_wpm);
println!("Average accuracy: {:.1}%", summary.avg_accuracy);

// Analyze improvement
let (improved, percentage) = history.analyze_improvement(5);
if improved {
    println!("You've improved by {:.1}%!", percentage);
}

// Get recent performance summary
let recent_summary = history.get_recent_summary(5);
```

## Future Enhancements

- **Visual Charts**: Graph performance trends over time with visual charts
- **Export Statistics**: Export session history to CSV or JSON for external analysis
- **Difficulty Levels**: Multiple code complexity levels and programming languages
- **Enhanced Syntax Highlighting**: More sophisticated color schemes and themes
- **Leaderboards**: Score tracking and comparison features
- **Multiple Languages**: Support for different programming languages beyond Rust
- **Advanced Feedback**: Real-time typing technique analysis and suggestions
- **Animation Effects**: Text animations and transitions using the colored text system
- **Pause/Resume**: Ability to pause sessions and resume later
- **Custom Goals**: Set personal targets for CPM/WPM improvements

## License

MIT