# Configuration System - Quick Reference

## 🚀 Get Started in 3 Steps

### 1. Generate Config File
```bash
cargo run gen-config
```

### 2. Edit (Optional)
Open `config.toml` and customize any settings you want.

### 3. Run
```bash
cargo run
```

That's it! Your settings are automatically applied.

---

## 📁 Configuration Files in This Project

| File | Purpose |
|------|---------|
| `config.toml` | Your actual configuration (git-ignored) |
| `config.toml.example` | Fully documented example with all options |
| `CONFIG.md` | Complete reference documentation |
| `CONFIGURATION_QUICKSTART.md` | Common use cases and examples |
| `src/config.rs` | Configuration system implementation |

---

## ⚡ Quick Examples

### Change Text Size
```toml
[text]
font_size = 80.0
```

### Practice Your Code
```toml
[gameplay]
custom_code_path = "examples/practice_code.rs"
```

### Enable Debug Mode
```toml
[debug]
log_level = "debug"
verbose_input = true
```

### Customize Colors
```toml
[colors]
background = [0.0, 0.0, 0.0, 1.0]  # Black
text_default = [1.0, 1.0, 1.0, 1.0]  # White
```

---

## 🎯 What Can You Configure?

### Window
- Size, title, VSync, FPS

### Text Rendering
- Font, size, position, spacing, colors, syntax highlighting

### Gameplay
- Custom code files, backspace, statistics, difficulty

### Debug
- Log levels, Vulkan validation, performance monitoring

### Colors
- Background, text, syntax highlighting (full RGBA control)

---

## 📚 Documentation

**Just Getting Started?**
→ Read `CONFIGURATION_QUICKSTART.md`

**Want All the Details?**
→ Read `CONFIG.md`

**Need Examples?**
→ Look at `config.toml.example`

**Want to See Use Cases?**
→ Read `FEATURE_CONFIG_SYSTEM.md`

---

## 🔧 Common Commands

```bash
# Generate default config
cargo run gen-config

# Run with your config
cargo run

# Run demo mode
cargo run demo
```

---

## 💡 Pro Tips

1. **No config needed** - App works great with defaults
2. **Partial config** - Only specify what you want to change
3. **Multiple configs** - Keep different configs for different uses
4. **Reset to defaults** - Just delete `config.toml`
5. **Check validation** - App logs warnings for config issues

---

## 🎨 Popular Presets

### Presentation Mode
```toml
[text]
font_size = 96.0
```

### Challenge Mode
```toml
[gameplay]
allow_backspace = false
strict_mode = true
```

### Debug Mode
```toml
[debug]
log_level = "debug"
show_frame_times = true
```

### Light Theme
```toml
[colors]
background = [0.95, 0.95, 0.95, 1.0]
text_default = [0.1, 0.1, 0.1, 1.0]
```

---

## ❓ Troubleshooting

**Config not loading?**
- Check filename is `config.toml`
- Must be in project root (same folder as `Cargo.toml`)

**Font not found?**
- Verify `font_path` points to existing `.ttf` file
- Default: `fonts/JetBrainsMono-Light.ttf`

**Colors look wrong?**
- Values must be 0.0 to 1.0 (not 0-255)
- Example: `[1.0, 0.0, 0.0, 1.0]` for red

**Custom code not loading?**
- Check path in `custom_code_path`
- Path is relative to project root
- File must exist and be readable

---

## 🎓 Learn More

Start with the quickstart guide:
```bash
cat CONFIGURATION_QUICKSTART.md
```

Or jump to full docs:
```bash
cat CONFIG.md
```

---

## ✅ Features

✅ TOML-based configuration
✅ Automatic loading with defaults
✅ Comprehensive validation
✅ Well-documented
✅ Type-safe
✅ Easy to extend
✅ Backward compatible

---

**Happy Configuring! 🦀**

For questions, see the documentation files listed above.