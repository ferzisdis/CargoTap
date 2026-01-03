# Configuration Quick Start Guide

Get started with CargoTap configuration in 3 easy steps!

## 🚀 Quick Setup (30 seconds)

### Step 1: Generate Config File

```bash
cargo run gen-config
```

This creates `config.toml` with all settings documented.

### Step 2: Edit Settings (Optional)

Open `config.toml` and customize what you need. You can delete any sections you don't want to change - defaults will be used.

### Step 3: Run the App

```bash
cargo run
```

Your settings are automatically loaded!

---

## 📋 Most Common Configurations

### 🔤 Change Text Size

```toml
[text]
font_size = 80.0  # Larger text
```

### 📁 Practice Your Own Code

```toml
[gameplay]
custom_code_path = "examples/practice_code.rs"
```

### 🎨 Change Colors

```toml
[colors]
background = [0.0, 0.0, 0.0, 1.0]  # Pure black
text_default = [1.0, 1.0, 1.0, 1.0]  # Pure white
```

### 🐛 Enable Debug Mode

```toml
[debug]
log_level = "debug"
log_code_state = true
verbose_input = true
```

### 💪 Challenge Mode

```toml
[gameplay]
allow_backspace = false
strict_mode = true
show_next_char_hint = false
```

### 🎯 Performance Mode

```toml
[window]
vsync = false
target_fps = 0  # Unlimited

[debug]
show_frame_times = true
show_fps = true
```

---

## 🎨 Color Reference

Colors use `[R, G, B, A]` format (0.0 to 1.0):

```toml
[1.0, 0.0, 0.0, 1.0]  # Red
[0.0, 1.0, 0.0, 1.0]  # Green
[0.0, 0.0, 1.0, 1.0]  # Blue
[1.0, 1.0, 0.0, 1.0]  # Yellow
[1.0, 0.0, 1.0, 1.0]  # Magenta
[0.0, 1.0, 1.0, 1.0]  # Cyan
[1.0, 1.0, 1.0, 1.0]  # White
[0.0, 0.0, 0.0, 1.0]  # Black
[0.5, 0.5, 0.5, 1.0]  # Gray
```

---

## 📊 Complete Configuration Template

Here's a minimal config with the most useful settings:

```toml
# Window settings
[window]
width = 1280
height = 720
vsync = true
target_fps = 60

# Text appearance
[text]
font_path = "fonts/JetBrainsMono-Light.ttf"
font_size = 64.0
position_x = 20.0
position_y = 50.0
syntax_highlighting = true

# Game options
[gameplay]
# custom_code_path = "examples/practice_code.rs"  # Uncomment to use
allow_backspace = true
show_statistics = true
show_next_char_hint = true

# Debug options
[debug]
log_level = "info"  # "trace" | "debug" | "info" | "warn" | "error"
vulkan_validation = false
log_code_state = true

# Colors (optional - uses good defaults if omitted)
[colors]
background = [0.1, 0.1, 0.1, 1.0]
text_default = [0.9, 0.9, 0.9, 1.0]
text_correct = [0.0, 1.0, 0.0, 1.0]
text_incorrect = [1.0, 0.0, 0.0, 1.0]
```

---

## 🎓 Learning Path

1. **Beginner**: Start with defaults, no config file needed
2. **Customize**: Run `cargo run gen-config` and adjust text size
3. **Practice**: Add your own code file with `custom_code_path`
4. **Theme**: Customize colors to match your preference
5. **Advanced**: Explore all options in `CONFIG.md`

---

## 💡 Pro Tips

### Tip 1: Partial Configuration
You don't need all sections! This is valid:
```toml
[text]
font_size = 72.0
```

### Tip 2: Multiple Configs
Keep different configs for different purposes:
```bash
cp config.toml config.backup
cp config.debug.toml config.toml
cargo run
```

### Tip 3: Reset to Defaults
Delete `config.toml` and the app uses built-in defaults:
```bash
rm config.toml
cargo run
```

### Tip 4: Validate Your Config
Check logs on startup for validation warnings:
```bash
cargo run 2>&1 | grep -i warning
```

### Tip 5: Custom Practice Files
Create a practice folder:
```bash
mkdir practice
echo "fn main() { println!(\"test\"); }" > practice/easy.rs
```

Then in config.toml:
```toml
[gameplay]
custom_code_path = "practice/easy.rs"
```

---

## 🔧 Troubleshooting

### Config Not Loading?
- Check filename is exactly `config.toml`
- Place in project root (same folder as `Cargo.toml`)
- Look for error messages in console

### Font Not Found?
```toml
[text]
font_path = "fonts/JetBrainsMono-Light.ttf"  # Check this path exists
```

### Colors Wrong?
- Ensure values are 0.0 to 1.0, not 0-255
- Wrong: `[255, 0, 0, 1]`
- Right: `[1.0, 0.0, 0.0, 1.0]`

### Custom Code Not Loading?
- Check file path is relative to project root
- Verify file exists and is readable
- Look for error message in logs

---

## 📚 More Information

- **Full Documentation**: See [CONFIG.md](CONFIG.md)
- **Example File**: See [config.toml.example](config.toml.example)
- **Main README**: See [README.md](README.md)

---

## ⚡ Common Use Cases

### 1. Large Screen Demo
```toml
[window]
width = 1920
height = 1080

[text]
font_size = 96.0
position_x = 100.0
position_y = 100.0
```

### 2. Minimal Distraction Mode
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

### 3. Maximum Debugging
```toml
[debug]
log_level = "trace"
vulkan_validation = true
show_frame_times = true
show_memory_usage = true
verbose_input = true
debug_text_system = true
log_code_state = true
```

### 4. Light Theme
```toml
[colors]
background = [0.95, 0.95, 0.95, 1.0]
text_default = [0.1, 0.1, 0.1, 1.0]
text_correct = [0.0, 0.5, 0.0, 1.0]
text_incorrect = [0.8, 0.0, 0.0, 1.0]
text_current = [0.8, 0.5, 0.0, 1.0]
```

### 5. Speedrun Setup
```toml
[gameplay]
allow_backspace = false
strict_mode = true
show_statistics = true

[debug]
log_level = "warn"
log_code_state = false

[window]
target_fps = 144
```

---

## 🎯 Next Steps

1. ✅ Generate your config: `cargo run gen-config`
2. ✅ Try changing one setting (like `font_size`)
3. ✅ Run the app: `cargo run`
4. ✅ Experiment with colors
5. ✅ Try a custom code file
6. ✅ Read full docs: [CONFIG.md](CONFIG.md)

---

**Happy Typing! 🦀**