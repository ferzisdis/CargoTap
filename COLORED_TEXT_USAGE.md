# Colored Text System Usage Guide

This document describes how to use the new colored text functionality in CargoTap, which allows individual characters to have their own colors.

## Overview

The colored text system replaces the previous single-color text rendering with per-character color support. This enables syntax highlighting, rainbow text effects, and other advanced text visualizations.

## Key Components

### 1. ColoredChar Structure

```rust
pub struct ColoredChar {
    pub ch: char,
    pub color: [f32; 4], // RGBA values (0.0 to 1.0)
}
```

Represents a single character with its associated color.

### 2. ColoredText Structure

```rust
pub struct ColoredText {
    pub chars: Vec<ColoredChar>,
}
```

Contains a collection of colored characters that form the complete text.

### 3. Updated TextVertex Structure

```rust
pub struct TextVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],     // New field for per-vertex color
}
```

The vertex structure now includes a color field for each vertex.

## API Methods

### ColoredText Methods

#### `new()`
Creates an empty ColoredText instance.

```rust
let mut colored_text = ColoredText::new();
```

#### `from_str_with_color(text: &str, color: [f32; 4])`
Creates ColoredText from a string with uniform color.

```rust
let colored_text = ColoredText::from_str_with_color("Hello World", [1.0, 0.0, 0.0, 1.0]); // Red text
```

#### `push(ch: char, color: [f32; 4])`
Adds a single colored character.

```rust
colored_text.push('H', [1.0, 0.0, 0.0, 1.0]); // Red 'H'
```

#### `push_str(text: &str, color: [f32; 4])`
Adds multiple characters with the same color.

```rust
colored_text.push_str("Hello", [0.0, 1.0, 0.0, 1.0]); // Green "Hello"
```

### TextSystem Methods

#### `update_text_with_settings(colored_text: &ColoredText)`
The main method that updates the text system with colored text.

```rust
text_system.update_text_with_settings(&colored_text)?;
```

#### `update_text_with_settings_str(text: &str)` (Backward Compatibility)
Convenience method for simple text with default colors.

```rust
text_system.update_text_with_settings_str("Simple text")?;
```

## Usage Examples

### Example 1: Simple Rainbow Text

```rust
fn create_rainbow_text() -> ColoredText {
    let mut colored_text = ColoredText::new();
    let text = "Rainbow!";
    let colors = [
        [1.0, 0.0, 0.0, 1.0], // Red
        [1.0, 0.5, 0.0, 1.0], // Orange
        [1.0, 1.0, 0.0, 1.0], // Yellow
        [0.0, 1.0, 0.0, 1.0], // Green
        [0.0, 0.0, 1.0, 1.0], // Blue
        [0.5, 0.0, 1.0, 1.0], // Indigo
        [1.0, 0.0, 1.0, 1.0], // Violet
        [1.0, 1.0, 1.0, 1.0], // White
    ];
    
    for (i, ch) in text.chars().enumerate() {
        let color = colors[i % colors.len()];
        colored_text.push(ch, color);
    }
    
    colored_text
}
```

### Example 2: Syntax Highlighting

```rust
fn create_syntax_highlighted_text(code: &str) -> ColoredText {
    let mut colored_text = ColoredText::new();
    let mut current_word = String::new();
    
    for ch in code.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            current_word.push(ch);
        } else {
            if !current_word.is_empty() {
                let color = match current_word.as_str() {
                    // Keywords in blue
                    "fn" | "let" | "mut" | "pub" | "struct" | "impl" => [0.4, 0.6, 1.0, 1.0],
                    // Types in green
                    "String" | "i32" | "u32" | "f32" | "f64" => [0.4, 1.0, 0.4, 1.0],
                    // Functions in cyan
                    "println" | "main" => [0.0, 1.0, 1.0, 1.0],
                    // Default text color
                    _ => [0.9, 0.9, 0.9, 1.0],
                };
                
                for word_ch in current_word.chars() {
                    colored_text.push(word_ch, color);
                }
                current_word.clear();
            }
            
            // Color special characters
            let char_color = match ch {
                '{' | '}' | '(' | ')' | '[' | ']' => [1.0, 0.8, 0.4, 1.0], // Orange
                '"' => [1.0, 0.6, 0.6, 1.0], // Pink
                '!' => [1.0, 0.4, 1.0, 1.0], // Magenta
                _ => [0.8, 0.8, 0.8, 1.0], // Light gray
            };
            colored_text.push(ch, char_color);
        }
    }
    
    colored_text
}
```

### Example 3: Multi-colored Headers

```rust
fn create_header_text() -> ColoredText {
    let mut colored_text = ColoredText::new();
    
    // Title in large, bright color
    colored_text.push_str("CargoTap ", [0.0, 1.0, 1.0, 1.0]); // Cyan
    colored_text.push_str("Text ", [1.0, 1.0, 0.0, 1.0]);     // Yellow
    colored_text.push_str("Demo\n", [1.0, 0.5, 0.0, 1.0]);   // Orange
    
    // Subtitle in smaller, dimmer color
    colored_text.push_str("Advanced text rendering with per-character colors", 
                         [0.7, 0.7, 0.7, 1.0]); // Gray
    
    colored_text
}
```

## Color Format

Colors are specified as RGBA arrays with values from 0.0 to 1.0:

- `[1.0, 0.0, 0.0, 1.0]` - Pure red, fully opaque
- `[0.0, 1.0, 0.0, 0.5]` - Pure green, 50% transparent
- `[0.5, 0.5, 0.5, 1.0]` - Gray, fully opaque
- `[1.0, 1.0, 1.0, 1.0]` - White, fully opaque

## Technical Details

### Shader Changes

The vertex shader now accepts per-vertex colors:

```glsl
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in vec4 color;      // New color input

layout(location = 0) out vec2 frag_tex_coords;
layout(location = 1) out vec4 frag_color; // Pass color to fragment shader
```

The fragment shader applies the per-vertex color:

```glsl
layout(location = 0) in vec2 frag_tex_coords;
layout(location = 1) in vec4 frag_color;  // Receive color from vertex shader

void main() {
    float alpha = texture(glyph_texture, frag_tex_coords).r;
    f_color = vec4(frag_color.rgb * alpha, frag_color.a);
}
```

### Performance Considerations

- Each character now requires 6 vertices (2 triangles) with color data
- Memory usage increases by 16 bytes per vertex (4 floats for RGBA)
- GPU can efficiently process per-vertex colors in parallel
- No significant performance impact for typical text rendering loads

## Migration Guide

### From Old API to New API

**Old usage:**
```rust
text_system.update_text_with_settings("Hello World")?;
```

**New usage (backward compatible):**
```rust
text_system.update_text_with_settings_str("Hello World")?;
```

**New usage (with colors):**
```rust
let colored_text = ColoredText::from_str_with_color("Hello World", [1.0, 0.0, 0.0, 1.0]);
text_system.update_text_with_settings(&colored_text)?;
```

### Custom Color Implementation

For advanced use cases, you can manually construct ColoredText:

```rust
let mut colored_text = ColoredText::new();

// Add characters individually with custom logic
for (i, ch) in "Hello".chars().enumerate() {
    let intensity = (i as f32) / 4.0; // Fade effect
    let color = [intensity, 1.0 - intensity, 0.5, 1.0];
    colored_text.push(ch, color);
}
```

## Future Enhancements

Potential future improvements to the colored text system:

1. **Color Gradients**: Smooth color transitions between characters
2. **Animation Support**: Time-based color changes
3. **Style Attributes**: Bold, italic, underline with colors
4. **Theme Support**: Predefined color schemes for syntax highlighting
5. **Performance Optimization**: Batching for identical colors

## Troubleshooting

### Common Issues

1. **Colors appear too dark**: Ensure alpha values are set to 1.0 for fully opaque colors
2. **Text not visible**: Check that at least one color channel (R, G, or B) is greater than 0.0
3. **Performance issues**: Consider using fewer unique colors per frame for better batching

### Debug Tips

- Use the atlas debug output to verify glyph rendering
- Check console output for text system errors
- Verify color values are in the 0.0-1.0 range