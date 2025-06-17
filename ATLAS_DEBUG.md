# Atlas Debug Bitmap Feature

This document describes the atlas debug bitmap feature that allows you to save the font atlas as a PNG image for debugging purposes.

## Overview

The text rendering system in CargoTap creates a texture atlas containing all the glyphs (characters) used for rendering text. This atlas is a single texture that contains all the character bitmaps arranged in a grid-like pattern. The atlas debug feature allows you to save this atlas as a PNG image file to visualize how the glyphs are arranged and packed.

## Usage

### Environment Variable Control

You can control whether the atlas debug bitmap is saved using the `CARGOTAP_DEBUG_ATLAS` environment variable:

```bash
# Enable atlas debug saving (default behavior)
CARGOTAP_DEBUG_ATLAS=true cargo run

# or
CARGOTAP_DEBUG_ATLAS=1 cargo run

# Disable atlas debug saving
CARGOTAP_DEBUG_ATLAS=false cargo run
```

### Output Files

When enabled, the system will generate PNG files with the following naming pattern:
```
atlas_debug_{width}x{height}_size{font_size}_{timestamp}.png
```

For example:
- `atlas_debug_512x512_size64_1750189814.png`

Where:
- `512x512` is the atlas texture dimensions
- `size64` is the font size used
- `1750189814` is a Unix timestamp for uniqueness

## What You'll See

The generated PNG file will show:
- **White areas**: Glyph pixels (the actual character shapes)
- **Black areas**: Empty/unused space in the atlas
- **Layout**: How glyphs are packed row by row in the atlas

## Debug Information

The system also logs useful information about the atlas:

```
Atlas debug bitmap saved as: atlas_debug_512x512_size64_1750189814.png (utilization: 46.9%)
Text atlas created successfully with 94 glyphs, atlas utilization: 37.7%
```

This tells you:
- How many glyphs were successfully packed into the atlas
- The utilization percentage (how much of the atlas space is actually used)

## Use Cases

This feature is helpful for:

1. **Debugging rendering issues**: Verify that glyphs are being rasterized correctly
2. **Optimizing atlas size**: See if the atlas is too large or too small for your needs
3. **Understanding glyph packing**: Visualize how the atlas packing algorithm arranges glyphs
4. **Performance analysis**: Check atlas utilization to optimize memory usage
5. **Font debugging**: Ensure all required characters are present in the atlas

## Technical Details

- The atlas uses a single-channel (grayscale) format (`R8_UNORM` in Vulkan)
- Currently supports ASCII printable characters (32-126)
- Uses a 512x512 pixel atlas by default
- Glyphs are packed row by row with 1-pixel padding between them
- The bitmap is saved before uploading to GPU memory

## File Location

Debug atlas files are saved in the same directory where you run the application (typically the project root directory).

## Programming Example

If you want to control atlas debug saving programmatically, you can use the environment variable or modify the `should_save_debug_atlas()` method:

```rust
use std::env;

// Set environment variable programmatically
env::set_var("CARGOTAP_DEBUG_ATLAS", "true");

// Create text system and atlas
let mut text_system = TextSystem::new(/* parameters */)?;
text_system.create_text_atlas(pipeline_layout)?; // This will save debug bitmap if enabled

// Note: Manual saving after atlas creation requires GPU readback (not currently implemented)
// text_system.save_atlas_debug()?; // Limited functionality
```

You can also modify the default behavior by changing the `should_save_debug_atlas()` method in `src/text.rs`:

```rust
fn should_save_debug_atlas(&self) -> bool {
    // Always save in debug builds, never in release builds
    cfg!(debug_assertions)
    
    // Or check environment variable with different default:
    // env::var("CARGOTAP_DEBUG_ATLAS")
    //     .map(|val| val.to_lowercase() == "true" || val == "1")
    //     .unwrap_or(false) // Default to false instead of true
}
```

## Notes

- The feature is enabled by default but can be disabled for production builds
- Each atlas creation generates a new file with a unique timestamp
- The PNG files are relatively small (around 28KB for typical usage)
- Old debug files are not automatically cleaned up
- Manual atlas saving after creation requires GPU memory readback (not currently implemented)