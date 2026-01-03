# CargoTap Configuration Guide

This guide explains how to configure CargoTap using the `config.toml` configuration file.

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration Sections](#configuration-sections)
  - [Window Settings](#window-settings)
  - [Text Rendering](#text-rendering)
  - [Gameplay Options](#gameplay-options)
  - [Debug Settings](#debug-settings)
  - [Color Scheme](#color-scheme)
- [Common Use Cases](#common-use-cases)
- [Troubleshooting](#troubleshooting)
- [Advanced Configuration](#advanced-configuration)

## Quick Start

### Step 1: Generate Configuration File

Run the following command to generate a default `config.toml` file:

```bash
cargo run gen-config
```

This creates a `config.toml` file in your project root with all default values and comprehensive documentation.

### Step 2: Edit Configuration

Open `config.toml` in your favorite text editor and modify the values you want to change. You can delete any sections you don't need to customize - the application will use defaults for missing values.

### Step 3: Run the Application

```bash
cargo run
```

Your configuration will be loaded automatically on startup.

## Configuration Sections

### Window Settings

Controls the application window appearance and behavior.

```toml
[window]
title = "CargoTap - Typing Game"  # Window title
width = 1280                      # Window width in pixels
height = 720                      # Window height in pixels
vsync = true                      # Enable vertical sync
target_fps = 60                   # Target frame rate (0 = unlimited)
```

**Tips:**
- For fullscreen-like experience, set width and height to your monitor resolution
- Disable vsync (`vsync = false`) for maximum performance in benchmarking
- Set `target_fps = 0` for unlimited frame rate

### Text Rendering

Configures how text is displayed on screen.

```toml
[text]
font_path = "fonts/JetBrainsMono-Light.ttf"  # Path to font file
font_size = 64.0                              # Font size in points
position_x = 20.0                             # Horizontal position
position_y = 50.0                             # Vertical position
line_spacing = 1.2                            # Line height multiplier
char_spacing = 0.0                            # Extra character spacing
syntax_highlighting = true                    # Enable syntax colors
rainbow_effects = true                        # Enable rainbow text
```

**Tips:**
- **Larger text**: Increase `font_size` to 96.0 or 128.0
- **Smaller text**: Decrease `font_size` to 32.0 or 48.0
- **Center text**: Adjust `position_x` and `position_y` values
- **Tighter lines**: Set `line_spacing = 1.0`
- **More spacing**: Set `line_spacing = 1.5` or higher
- **Disable effects**: Set `syntax_highlighting = false` and `rainbow_effects = false` for plain text

### Gameplay Options

Controls game mechanics and features.

```toml
[gameplay]
custom_code_path = "my_code.rs"   # Optional: custom code file to practice
allow_backspace = true             # Allow correcting mistakes
show_statistics = true             # Show progress statistics
audio_feedback = false             # Audio on keypress (future)
strict_mode = false                # No backspace, no mistakes
show_next_char_hint = true         # Show next character hint
```

**Tips:**
- **Practice your code**: Set `custom_code_path = "path/to/your/file.rs"`
- **Hard mode**: Enable `strict_mode = true` and `allow_backspace = false`
- **Quiet mode**: Disable `show_statistics = false` for minimal output
- **Challenge yourself**: Use `strict_mode = true` for no-mistake typing

### Debug Settings

Controls logging and debugging features.

```toml
[debug]
log_level = "info"              # Log verbosity level
vulkan_validation = false       # Enable Vulkan validation layers
show_frame_times = false        # Display frame timing
show_memory_usage = false       # Display memory stats
verbose_input = false           # Log every keystroke
debug_text_system = false       # Text system debug output
log_code_state = true           # Log typing progress
show_fps = false                # Show FPS counter
save_logs_to_file = false       # Save logs to file
log_file_path = "debug.log"     # Log file location
```

**Log Levels:**
- `"trace"` - Most verbose, shows everything
- `"debug"` - Detailed debugging information
- `"info"` - General information (recommended)
- `"warn"` - Only warnings and errors
- `"error"` - Only errors

**Tips:**
- **Debugging issues**: Set `log_level = "debug"` or `"trace"`
- **Performance testing**: Enable `show_frame_times = true` and `show_fps = true`
- **Graphics debugging**: Enable `vulkan_validation = true` (warning: slow!)
- **Input debugging**: Enable `verbose_input = true` to see every keystroke
- **Clean console**: Set `log_level = "warn"` and `log_code_state = false`

### Color Scheme

Customizes all colors in the application. Colors use RGBA format with values from 0.0 to 1.0.

```toml
[colors]
background = [0.1, 0.1, 0.1, 1.0]       # Dark gray background
text_default = [0.9, 0.9, 0.9, 1.0]     # Light gray text
text_correct = [0.0, 1.0, 0.0, 1.0]     # Green for correct
text_incorrect = [1.0, 0.0, 0.0, 1.0]   # Red for errors
text_current = [1.0, 1.0, 0.0, 1.0]     # Yellow for current char
text_header = [0.0, 1.0, 1.0, 1.0]      # Cyan for headers

# Syntax highlighting colors
syntax_keyword = [1.0, 0.3, 0.5, 1.0]   # Pink-ish for keywords
syntax_type = [0.3, 0.8, 1.0, 1.0]      # Light blue for types
syntax_string = [0.5, 1.0, 0.5, 1.0]    # Light green for strings
syntax_comment = [0.5, 0.5, 0.5, 1.0]   # Gray for comments
syntax_number = [1.0, 0.8, 0.4, 1.0]    # Orange for numbers
syntax_function = [0.8, 0.6, 1.0, 1.0]  # Purple for functions
```

**Color Format:**
- `[R, G, B, A]` where each value is 0.0 to 1.0
- R = Red, G = Green, B = Blue, A = Alpha (opacity)
- Examples:
  - `[1.0, 0.0, 0.0, 1.0]` = Bright red, fully opaque
  - `[0.0, 1.0, 0.0, 0.5]` = Green, 50% transparent
  - `[1.0, 1.0, 1.0, 1.0]` = White
  - `[0.0, 0.0, 0.0, 1.0]` = Black

**Popular Color Schemes:**

**Dark Mode (Default):**
```toml
background = [0.1, 0.1, 0.1, 1.0]
text_default = [0.9, 0.9, 0.9, 1.0]
```

**Dracula Theme:**
```toml
background = [0.16, 0.16, 0.21, 1.0]
text_default = [0.95, 0.95, 0.98, 1.0]
syntax_keyword = [1.0, 0.47, 0.78, 1.0]
syntax_string = [0.94, 0.98, 0.55, 1.0]
```

**Solarized Dark:**
```toml
background = [0.0, 0.17, 0.21, 1.0]
text_default = [0.51, 0.58, 0.59, 1.0]
```

**Light Mode:**
```toml
background = [0.95, 0.95, 0.95, 1.0]
text_default = [0.1, 0.1, 0.1, 1.0]
text_correct = [0.0, 0.6, 0.0, 1.0]
text_incorrect = [0.8, 0.0, 0.0, 1.0]
```

## Common Use Cases

### Large Display / Presentation Mode

```toml
[window]
width = 1920
height = 1080

[text]
font_size = 96.0
position_x = 100.0
position_y = 100.0
line_spacing = 1.5
```

### Practice Custom Code

```toml
[gameplay]
custom_code_path = "examples/my_practice.rs"
show_next_char_hint = true
```

Create `examples/my_practice.rs` with the code you want to practice.

### Speedrun / Challenge Mode

```toml
[gameplay]
allow_backspace = false
strict_mode = true
show_statistics = true
show_next_char_hint = false

[debug]
log_code_state = false
log_level = "warn"
```

### Debug Graphics Issues

```toml
[debug]
log_level = "debug"
vulkan_validation = true
debug_text_system = true
show_frame_times = true
show_memory_usage = true
```

### Performance Benchmarking

```toml
[window]
vsync = false
target_fps = 0

[debug]
show_frame_times = true
show_fps = true
log_level = "warn"
```

### Minimal / Distraction-Free

```toml
[text]
syntax_highlighting = false
rainbow_effects = false

[gameplay]
show_statistics = false
show_next_char_hint = false

[debug]
log_code_state = false
log_level = "error"
```

## Troubleshooting

### Configuration Not Loading

**Problem**: Changes to `config.toml` don't seem to take effect.

**Solutions:**
1. Make sure the file is named exactly `config.toml` (not `config.toml.txt`)
2. Place it in the project root directory (same level as `Cargo.toml`)
3. Check for syntax errors - the app will log warnings
4. Restart the application after making changes

### Font Not Found

**Problem**: Error message about missing font file.

**Solutions:**
1. Verify `text.font_path` points to an existing `.ttf` file
2. Use relative path from project root: `"fonts/YourFont.ttf"`
3. Check that the font file actually exists at that location
4. Try using the default: `"fonts/JetBrainsMono-Light.ttf"`

### Custom Code Not Loading

**Problem**: Custom code file specified but not appearing.

**Solutions:**
1. Check that `gameplay.custom_code_path` is correctly set
2. Verify the file exists and is readable
3. Use relative path from project root
4. Check logs for error messages about file loading

### Colors Look Wrong

**Problem**: Colors appear incorrect or too bright/dark.

**Solutions:**
1. Ensure all color values are between 0.0 and 1.0
2. Check alpha (4th) value - should be 1.0 for fully opaque
3. Reset to defaults by removing `[colors]` section
4. Try a preset color scheme from this guide

### Performance Issues

**Problem**: Low frame rate or stuttering.

**Solutions:**
1. Disable Vulkan validation: `vulkan_validation = false`
2. Reduce `font_size` to something smaller (e.g., 48.0)
3. Disable `show_frame_times` and other debug options
4. Enable vsync: `vsync = true`
5. Set reasonable `target_fps` (60 or 144)

## Advanced Configuration

### Partial Configuration

You don't need to specify all settings. Any missing values will use defaults:

```toml
# Minimal config - only customize font size
[text]
font_size = 80.0
```

### Environment-Specific Configs

You can maintain multiple configuration files:

```bash
# Development
cp config.dev.toml config.toml
cargo run

# Production/Presentation
cp config.prod.toml config.toml
cargo run
```

### Configuration Validation

The application validates your configuration on startup and logs warnings for:
- Invalid color values (not in 0.0-1.0 range)
- Missing font files
- Missing custom code files
- Very small or very large window dimensions
- Very small or very large font sizes

Check the logs for validation messages when starting the app.

### Programmatic Access

In the code, configuration is accessed via the `Config` struct:

```rust
use crate::config::Config;

let config = Config::load();
println!("Font size: {}", config.text.font_size);
println!("Allow backspace: {}", config.gameplay.allow_backspace);
```

### Default Values

To see all default values, either:
1. Run `cargo run gen-config` to generate a file with defaults
2. Check `config.toml.example` in the project root
3. Look at the `Default` implementations in `src/config.rs`

## Command-Line Options

- `cargo run` - Normal mode with configuration
- `cargo run demo` - Command-line demo mode
- `cargo run gen-config` - Generate default `config.toml`

## Examples

See `config.toml.example` for a fully documented example configuration file with all available options.

## Support

For issues or questions:
1. Check this documentation
2. Look at example configurations
3. Enable debug logging: `log_level = "debug"`
4. Check the application logs for error messages

---

**Last Updated**: 2024
**CargoTap Version**: 0.1.0