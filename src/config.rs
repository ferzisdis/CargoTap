//! Configuration module for CargoTap
//!
//! This module provides a comprehensive configuration system that allows users to:
//! - Customize application settings
//! - Configure rendering parameters
//! - Set debug options
//! - Adjust text display settings
//! - Configure input behavior
//!
//! Configuration is loaded from a TOML file (config.toml) in the project root.
//! If the file doesn't exist, default values are used.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Main configuration structure for CargoTap application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Window and rendering settings
    #[serde(default)]
    pub window: WindowConfig,

    /// Text rendering settings
    #[serde(default)]
    pub text: TextConfig,

    /// Gameplay settings
    #[serde(default)]
    pub gameplay: GameplayConfig,

    /// Debug and logging settings
    #[serde(default)]
    pub debug: DebugConfig,

    /// Color scheme settings
    #[serde(default)]
    pub colors: ColorConfig,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window title
    pub title: String,

    /// Initial window width
    pub width: u32,

    /// Initial window height
    pub height: u32,

    /// Enable VSync
    pub vsync: bool,

    /// Target frame rate (0 = unlimited)
    pub target_fps: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "CargoTap - Typing Game".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            target_fps: 60,
        }
    }
}

/// Text rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    /// Font file path relative to project root
    pub font_path: String,

    /// Font size in points
    pub font_size: f32,

    /// Text position X coordinate
    pub position_x: f32,

    /// Text position Y coordinate
    pub position_y: f32,

    /// Line spacing multiplier
    pub line_spacing: f32,

    /// Character spacing adjustment
    pub char_spacing: f32,

    /// Enable syntax highlighting
    pub syntax_highlighting: bool,

    /// Enable rainbow effects
    pub rainbow_effects: bool,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font_path: "fonts/JetBrainsMono-Light.ttf".to_string(),
            font_size: 64.0,
            position_x: 20.0,
            position_y: 50.0,
            line_spacing: 1.2,
            char_spacing: 0.0,
            syntax_highlighting: true,
            rainbow_effects: true,
        }
    }
}

/// Gameplay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayConfig {
    /// Source code file to practice typing (if set, overrides demo code)
    pub custom_code_path: Option<String>,

    /// Enable backspace functionality
    pub allow_backspace: bool,

    /// Show typing statistics
    pub show_statistics: bool,

    /// Enable audio feedback (for future implementation)
    pub audio_feedback: bool,

    /// Strict mode (no backspace, no mistakes allowed)
    pub strict_mode: bool,

    /// Show next character hint
    pub show_next_char_hint: bool,

    /// Number of lines to scroll when using scroll shortcut (Command+J)
    pub scroll_lines: usize,

    /// Session duration in minutes (timer for typing sessions)
    pub session_duration_minutes: f64,

    /// Auto-skip characters that cannot be typed on a US keyboard (emoji, Arabic, etc.)
    pub auto_skip_untypeable: bool,

    /// Hotkey to manually skip the current character (Ctrl+S or Cmd+S)
    pub enable_manual_skip: bool,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            custom_code_path: None,
            allow_backspace: true,
            show_statistics: true,
            audio_feedback: false,
            strict_mode: false,
            show_next_char_hint: true,
            scroll_lines: 5,
            session_duration_minutes: 3.0,
            auto_skip_untypeable: true,
            enable_manual_skip: true,
        }
    }
}

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Log level: "trace", "debug", "info", "warn", "error"
    pub log_level: String,

    /// Enable Vulkan validation layers
    pub vulkan_validation: bool,

    /// Print frame timing information
    pub show_frame_times: bool,

    /// Print memory usage information
    pub show_memory_usage: bool,

    /// Enable verbose input logging
    pub verbose_input: bool,

    /// Enable text system debug output
    pub debug_text_system: bool,

    /// Print code state changes
    pub log_code_state: bool,

    /// Enable FPS counter
    pub show_fps: bool,

    /// Save debug logs to file
    pub save_logs_to_file: bool,

    /// Debug log file path
    pub log_file_path: String,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            vulkan_validation: false,
            show_frame_times: false,
            show_memory_usage: false,
            verbose_input: false,
            debug_text_system: false,
            log_code_state: true,
            show_fps: false,
            save_logs_to_file: false,
            log_file_path: "cargotap_debug.log".to_string(),
        }
    }
}

/// Color scheme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    /// Background color [R, G, B, A] (0.0 - 1.0)
    pub background: [f32; 4],

    /// Default text color [R, G, B, A]
    pub text_default: [f32; 4],

    /// Correct character color
    pub text_correct: [f32; 4],

    /// Incorrect character color
    pub text_incorrect: [f32; 4],

    /// Current character (to be typed) color
    pub text_current: [f32; 4],

    /// Header text color
    pub text_header: [f32; 4],

    /// Syntax highlighting colors
    pub syntax_keyword: [f32; 4],
    pub syntax_type: [f32; 4],
    pub syntax_string: [f32; 4],
    pub syntax_comment: [f32; 4],
    pub syntax_number: [f32; 4],
    pub syntax_function: [f32; 4],
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            background: [0.1, 0.1, 0.1, 1.0],
            text_default: [0.9, 0.9, 0.9, 1.0],
            text_correct: [0.0, 1.0, 0.0, 1.0],
            text_incorrect: [1.0, 0.0, 0.0, 1.0],
            text_current: [1.0, 1.0, 0.0, 1.0],
            text_header: [0.0, 1.0, 1.0, 1.0],
            syntax_keyword: [1.0, 0.3, 0.5, 1.0],
            syntax_type: [0.3, 0.8, 1.0, 1.0],
            syntax_string: [0.5, 1.0, 0.5, 1.0],
            syntax_comment: [0.5, 0.5, 0.5, 1.0],
            syntax_number: [1.0, 0.8, 0.4, 1.0],
            syntax_function: [0.8, 0.6, 1.0, 1.0],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            text: TextConfig::default(),
            gameplay: GameplayConfig::default(),
            debug: DebugConfig::default(),
            colors: ColorConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Result<Config>` - Loaded configuration or error
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        log::info!("Configuration loaded from: {:?}", path.as_ref());
        config.log_config_summary();

        Ok(config)
    }

    /// Load configuration from default location (config.toml)
    /// If file doesn't exist, returns default configuration
    pub fn load() -> Self {
        let config_path = "config.toml";

        match Self::from_file(config_path) {
            Ok(config) => {
                log::info!("Using configuration from config.toml");
                config
            }
            Err(e) => {
                log::warn!(
                    "Could not load config.toml: {}. Using default configuration.",
                    e
                );
                log::info!(
                    "Tip: Run 'Config::save_default(\"config.toml\")' to create a default config file"
                );
                Self::default()
            }
        }
    }

    /// Save configuration to a TOML file
    ///
    /// # Arguments
    /// * `path` - Path where to save the configuration file
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_string =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config to TOML")?;

        fs::write(path.as_ref(), toml_string)
            .with_context(|| format!("Failed to write config file: {:?}", path.as_ref()))?;

        log::info!("Configuration saved to: {:?}", path.as_ref());
        Ok(())
    }

    /// Save default configuration to a file
    /// Useful for generating a template config file
    pub fn save_default<P: AsRef<Path>>(path: P) -> Result<()> {
        let default_config = Config::default();
        default_config.save(path)
    }

    /// Log a summary of the current configuration
    pub fn log_config_summary(&self) {
        log::info!("=== CargoTap Configuration ===");
        log::info!(
            "Window: {}x{} @ {} FPS",
            self.window.width,
            self.window.height,
            self.window.target_fps
        );
        log::info!(
            "Font: {} (size: {})",
            self.text.font_path,
            self.text.font_size
        );
        log::info!(
            "Text position: ({}, {})",
            self.text.position_x,
            self.text.position_y
        );
        log::info!("Syntax highlighting: {}", self.text.syntax_highlighting);
        log::info!("Allow backspace: {}", self.gameplay.allow_backspace);
        log::info!("Strict mode: {}", self.gameplay.strict_mode);
        log::info!("Log level: {}", self.debug.log_level);
        log::info!("Vulkan validation: {}", self.debug.vulkan_validation);
        log::info!("Verbose input logging: {}", self.debug.verbose_input);

        if let Some(ref custom_code) = self.gameplay.custom_code_path {
            log::info!("Custom code file: {}", custom_code);
        } else {
            log::info!("Using built-in demo code");
        }

        log::info!("==============================");
    }

    /// Get log level as a log::Level enum
    pub fn get_log_level(&self) -> log::Level {
        match self.debug.log_level.to_lowercase().as_str() {
            "trace" => log::Level::Trace,
            "debug" => log::Level::Debug,
            "info" => log::Level::Info,
            "warn" => log::Level::Warn,
            "error" => log::Level::Error,
            _ => {
                log::warn!("Invalid log level '{}', using Info", self.debug.log_level);
                log::Level::Info
            }
        }
    }

    /// Validate configuration values and return warnings/errors
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check window dimensions
        if self.window.width < 640 || self.window.height < 480 {
            warnings.push("Window size is very small. Minimum recommended: 640x480".to_string());
        }

        // Check font size
        if self.text.font_size < 8.0 {
            warnings.push("Font size is very small and may be unreadable".to_string());
        } else if self.text.font_size > 200.0 {
            warnings.push("Font size is very large and may not fit on screen".to_string());
        }

        // Check font file exists
        if !Path::new(&self.text.font_path).exists() {
            warnings.push(format!("Font file not found: {}", self.text.font_path));
        }

        // Check custom code file if specified
        if let Some(ref custom_code_path) = self.gameplay.custom_code_path {
            if !Path::new(custom_code_path).exists() {
                warnings.push(format!("Custom code file not found: {}", custom_code_path));
            }
        }

        // Check color values are in valid range
        let color_fields = [
            ("background", self.colors.background),
            ("text_default", self.colors.text_default),
            ("text_correct", self.colors.text_correct),
            ("text_incorrect", self.colors.text_incorrect),
        ];

        for (name, color) in &color_fields {
            for (i, &value) in color.iter().enumerate() {
                if !(0.0..=1.0).contains(&value) {
                    warnings.push(format!(
                        "Color {}.{} is out of range [0.0, 1.0]: {}",
                        name, i, value
                    ));
                }
            }
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.window.title, "CargoTap - Typing Game");
        assert_eq!(config.text.font_size, 64.0);
        assert!(config.gameplay.allow_backspace);
        assert_eq!(config.debug.log_level, "info");
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        let warnings = config.validate();
        // Default config should have minimal warnings (possibly font file not found in test env)
        assert!(warnings.len() <= 1);
    }

    #[test]
    fn test_log_level_parsing() {
        let mut config = Config::default();

        config.debug.log_level = "debug".to_string();
        assert_eq!(config.get_log_level(), log::Level::Debug);

        config.debug.log_level = "warn".to_string();
        assert_eq!(config.get_log_level(), log::Level::Warn);

        config.debug.log_level = "invalid".to_string();
        assert_eq!(config.get_log_level(), log::Level::Info);
    }
}
