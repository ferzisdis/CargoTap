# Performance Profiling Guide for CargoTap

This guide covers various methods to trace and profile CargoTap to identify performance bottlenecks and optimization opportunities.

## Quick Start: Built-in Diagnostics

CargoTap now includes on-screen diagnostics showing:
- **FPS**: Current frames per second
- **Key Processing Time**: How long each key press takes to process (in milliseconds)

These metrics appear at the top of the screen and update in real-time.

## 1. Flamegraph Profiling (Recommended for CPU)

Flamegraphs provide a visual representation of where your CPU time is spent.

### Installation
```bash
cargo install flamegraph
```

### Usage (macOS)
```bash
# Profile the application (run for ~30 seconds, then quit)
sudo cargo flamegraph --release

# Open the generated flamegraph.svg in your browser
open flamegraph.svg
```

### Usage (Linux with perf)
```bash
# Ensure perf is available
sudo apt-get install linux-tools-common linux-tools-generic

# Profile the application
cargo flamegraph --release

# View the flamegraph
firefox flamegraph.svg
```

### What to Look For
- **Wide bars** = functions taking lots of time
- **Tall stacks** = deep call chains (might indicate excessive abstraction)
- **Vulkan/GPU operations** = rendering overhead
- **Text processing** = glyph rasterization, string manipulation
- **Event handling** = input processing bottlenecks

## 2. Instruments (macOS Only)

Apple's Instruments provides detailed profiling on macOS.

### Usage
```bash
# Build with debug symbols
cargo build --release

# Run with Instruments Time Profiler
xcrun xctrace record --template 'Time Profiler' --launch ./target/release/CargoTap

# Or use cargo-instruments
cargo install cargo-instruments
cargo instruments --release --template time
```

### Key Templates
- **Time Profiler**: CPU usage and hot functions
- **Allocations**: Memory allocation patterns
- **System Trace**: Comprehensive system-level tracing
- **Metal System Trace**: GPU performance (Vulkan uses Metal on macOS)

## 3. perf (Linux Only)

Linux's perf tool provides detailed performance analysis.

### Basic Usage
```bash
# Record performance data
perf record --call-graph dwarf cargo run --release

# View the report
perf report

# Generate a detailed report
perf report --stdio > perf_report.txt
```

### Advanced Analysis
```bash
# Record with higher sampling rate
perf record -F 999 --call-graph dwarf cargo run --release

# Analyze cache misses
perf stat -e cache-references,cache-misses cargo run --release

# Monitor specific events
perf stat -e cycles,instructions,branches,branch-misses cargo run --release
```

## 4. Tracy Profiler (Real-time Visual Profiling)

Tracy provides real-time frame-by-frame profiling with microsecond precision.

### Setup
1. Download Tracy from: https://github.com/wolfpld/tracy
2. Add tracy-client to Cargo.toml:
```toml
[dependencies]
tracy-client = "0.17"
```

3. Add profiling zones to code:
```rust
use tracy_client::span;

fn handle_typing_input(app: &mut CargoTapApp) {
    let _span = span!("handle_typing_input");
    // ... function code
}
```

### Usage
```bash
# Run Tracy profiler GUI
./tracy

# Run your application with Tracy enabled
cargo run --release --features tracy
```

## 5. Manual Instrumentation with `tracing` Crate

Add detailed logging and timing to specific code sections.

### Add to Cargo.toml
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-chrome = "0.7"  # For Chrome trace format
```

### Example Usage
```rust
use tracing::{info_span, instrument};

#[instrument]
fn expensive_function() {
    let _span = info_span!("expensive_operation").entered();
    // code here
}
```

### Generate Chrome Trace
```rust
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::prelude::*;

let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
tracing_subscriber::registry().with(chrome_layer).init();
```

Then open `chrome://tracing` in Chrome and load the generated JSON file.

## 6. Custom Timing Points

Add detailed timing measurements to specific operations:

### Example: Measure Rendering Time
```rust
// In src/app.rs
pub struct CargoTapApp {
    // ... existing fields
    pub render_time_ms: f64,
    pub text_update_time_ms: f64,
}

// In update_text method
pub fn update_text(&mut self) {
    let start = Instant::now();
    
    if let Some(ref text_system) = self.text_system {
        if let Ok(mut text_system) = text_system.lock() {
            let colored_text = crate::ui::create_colored_text(self);
            if let Err(e) = text_system.update_text_with_settings(&colored_text) {
                log::error!("Failed to update main text: {}", e);
            }
        }
    }
    
    self.text_update_time_ms = start.elapsed().as_secs_f64() * 1000.0;
}
```

## 7. Memory Profiling with valgrind (Linux)

Analyze memory usage and potential leaks:

```bash
# Install valgrind
sudo apt-get install valgrind

# Run memory profiler
valgrind --tool=massif cargo run --release

# Visualize results
ms_print massif.out.* > memory_profile.txt
```

## 8. GPU Profiling

Since CargoTap uses Vulkan, GPU performance is critical:

### RenderDoc (All Platforms)
1. Download from: https://renderdoc.org/
2. Launch RenderDoc
3. Set executable to: `./target/release/CargoTap`
4. Capture frames and analyze GPU commands

### Vulkan Validation Layers
Enable validation layers for performance warnings:

```rust
// Add to vulkan_init.rs
let instance = Instance::new(
    library,
    InstanceCreateInfo {
        enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_owned()],
        // ... other settings
    },
)?;
```

## Common Performance Bottlenecks to Check

### 1. Text Rasterization
- **Symptom**: Slow updates when typing
- **Check**: Time spent in `rasterize_text_to_console()`
- **Solution**: Cache rasterized glyphs, use texture atlas more efficiently

### 2. String Allocations
- **Symptom**: High allocation rate
- **Check**: Memory profiler showing string allocations
- **Solution**: Use `String::with_capacity()`, reuse buffers

### 3. Vulkan Command Buffer Recording
- **Symptom**: Low FPS, high CPU usage
- **Check**: Time spent in `draw()` and command buffer building
- **Solution**: Reduce draw calls, batch rendering

### 4. Lock Contention
- **Symptom**: Stuttering, inconsistent frame times
- **Check**: Time spent waiting on `text_system.lock()`
- **Solution**: Reduce lock scope, use message passing

### 5. Event Processing
- **Symptom**: High key processing time (>5ms)
- **Check**: Built-in diagnostic showing key processing time
- **Solution**: Defer expensive operations, update asynchronously

## Recommended Profiling Workflow

1. **Start with built-in diagnostics**: Check FPS and key processing time
2. **Run flamegraph**: Get overview of CPU hotspots (5 min)
3. **Identify top 3 functions**: Focus on biggest time consumers
4. **Add targeted instrumentation**: Use `Instant::now()` around suspects
5. **Profile iteratively**: Make change → profile → verify improvement
6. **Check GPU**: Use RenderDoc if rendering is slow
7. **Memory check**: Use massif/Instruments if allocations are suspect

## Example: Profiling Session

```bash
# 1. Check current performance
cargo run --release
# Note the FPS and key processing time

# 2. Generate flamegraph
sudo cargo flamegraph --release
# Run for 30 seconds, type some code, then quit
open flamegraph.svg

# 3. If text rendering is hot:
#    - Check glyph rasterization time
#    - Profile texture atlas updates
#    - Measure vertex buffer uploads

# 4. If event handling is hot:
#    - Add timing around input processing
#    - Check string manipulation overhead
#    - Profile code state updates

# 5. Compare before/after
#    - Capture baseline metrics
#    - Make optimization
#    - Run flamegraph again
#    - Verify improvement
```

## Performance Goals

For a smooth typing experience:
- **Target FPS**: 60+ (16.67ms per frame)
- **Key Processing**: <2ms per keypress
- **Text Update**: <5ms for full screen redraw
- **Memory**: Stable (no continuous growth)

## Additional Resources

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph.pl Documentation](http://www.brendangregg.com/flamegraphs.html)
- [Vulkan Performance Tips](https://developer.nvidia.com/blog/vulkan-dos-donts/)
- [Tracy Profiler Manual](https://github.com/wolfpld/tracy/releases/latest/download/tracy.pdf)