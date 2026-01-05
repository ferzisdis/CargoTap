use anyhow::Result;
use log::info;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

mod char_utils;
mod code_state;
mod config;
mod demo_code_state;
mod input;
mod progress_helper;
mod progress_storage;
mod renderer;
mod session_state;
mod text;

use text::{ColoredText, TextRenderSettings};

mod examples;
use examples::colored_text_demo::ColoredTextDemo;

// Главная структура приложения
pub struct CargoTapApp {
    render_engine: renderer::VulkanRenderer,
    text_system: Option<Arc<Mutex<text::TextSystem>>>,
    input_handler: input::InputHandler,
    code_state: code_state::CodeState,
    config: config::Config,
    scroll_offset: usize, // Number of lines scrolled down (view offset)
    progress_storage: progress_storage::ProgressStorage,
    current_file_path: String,
    current_file_hash: String,
    session_state: session_state::SessionState,
}

impl CargoTapApp {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        // Load configuration
        let config = config::Config::load();

        // Validate configuration and log warnings
        let warnings = config.validate();
        for warning in warnings {
            log::warn!("Config validation: {}", warning);
        }

        let render_engine = renderer::VulkanRenderer::new(event_loop);
        let input_handler = input::InputHandler::new();

        // Load code based on configuration
        let demo_code = if let Some(ref custom_path) = config.gameplay.custom_code_path {
            match std::fs::read_to_string(custom_path) {
                Ok(code) => {
                    log::info!("Loaded custom code from: {}", custom_path);
                    code
                }
                Err(e) => {
                    log::error!("Failed to load custom code from {}: {}", custom_path, e);
                    log::info!("Falling back to demo code");
                    include_str!("demo_code.rs").to_string()
                }
            }
        } else {
            include_str!("demo_code.rs").to_string()
        };

        // Load progress storage
        let mut progress_storage = progress_storage::ProgressStorage::default();
        let _ = progress_storage.load(); // Ignore errors on first run

        // Compute file hash
        let file_path = config
            .gameplay
            .custom_code_path
            .clone()
            .unwrap_or_else(|| "demo_code.rs".to_string());
        let current_file_hash = progress_storage::compute_hash(&demo_code);

        // Create code state and restore progress
        let mut code_state = code_state::CodeState::new(demo_code);

        // Restore saved position if available and file hasn't changed
        if let Some(progress) = progress_storage.get_progress(&file_path) {
            if progress.content_hash == current_file_hash {
                log::info!("Restoring progress at position {}", progress.position);
                for _ in 0..progress.position {
                    if code_state.type_character().is_none() {
                        break;
                    }
                }
            } else {
                log::info!("File changed, starting from beginning");
            }
        }

        // Initialize session state with configured duration
        let session_state =
            session_state::SessionState::new(config.gameplay.session_duration_minutes);

        Ok(Self {
            render_engine,
            text_system: None,
            input_handler,
            code_state,
            config,
            scroll_offset: 0,
            progress_storage,
            current_file_path: file_path,
            current_file_hash,
            session_state,
        })
    }

    fn update_text(&mut self) {
        // Demonstrate colored text functionality
        if let Some(ref text_system) = self.text_system {
            if let Ok(mut text_system) = text_system.lock() {
                let colored_text = self.create_colored_text();
                if let Err(e) = text_system.update_text_with_settings(&colored_text) {
                    log::error!("Failed to update main text: {}", e);
                }
            }
        }
    }

    fn create_colored_text(&self) -> ColoredText {
        // Create comprehensive demo with header and syntax highlighting
        let mut colored_text = ColoredText::new();

        // Add a colorful header (if enabled)
        colored_text.push_str("🦀 CargoTap ", [1.0, 0.5, 0.0, 1.0]); // Orange
        colored_text.push_str("Live Demo\n", self.config.colors.text_header); // Use config color
        colored_text.push_str("─".repeat(30).as_str(), [0.5, 0.8, 1.0, 1.0]); // Light blue
        colored_text.push('\n', self.config.colors.text_default);

        // Add session information
        if self.session_state.is_finished() {
            // Show session results
            if let Some(stats) = self.session_state.last_stats() {
                colored_text.push_str("⏰ SESSION COMPLETE! ", [1.0, 1.0, 0.0, 1.0]); // Yellow
                colored_text.push('\n', self.config.colors.text_default);

                let summary = format!(
                    "Time: {:.1}s | Chars: {} | Speed: {:.0} CPM / {:.0} WPM\n",
                    stats.time_elapsed_secs,
                    stats.chars_typed,
                    stats.chars_per_minute,
                    stats.words_per_minute
                );
                colored_text.push_str(&summary, [0.0, 1.0, 0.0, 1.0]); // Green
                colored_text.push_str("Press SPACE to start new session\n", [0.0, 1.0, 1.0, 1.0]); // Cyan
                colored_text.push_str("─".repeat(30).as_str(), [0.5, 0.8, 1.0, 1.0]);
                colored_text.push('\n', self.config.colors.text_default);
            }
        } else if self.session_state.is_active() {
            // Show timer
            let time_str = format!("⏱️  Time: {} ", self.session_state.format_time_remaining());
            colored_text.push_str(&time_str, [1.0, 1.0, 0.0, 1.0]); // Yellow

            // Show current stats
            let current_pos = self.code_state.get_cursor_position();
            let stats = self.session_state.current_stats(current_pos);
            if stats.time_elapsed_secs > 0.0 {
                let speed_str = format!("| {:.0} CPM ", stats.chars_per_minute);
                colored_text.push_str(&speed_str, [0.0, 1.0, 0.0, 1.0]); // Green
            }
            colored_text.push('\n', self.config.colors.text_default);
            colored_text.push_str("─".repeat(30).as_str(), [0.5, 0.8, 1.0, 1.0]);
            colored_text.push('\n', self.config.colors.text_default);
        } else {
            // Session not started yet
            colored_text.push_str("Start typing to begin session...\n", [0.7, 0.7, 0.7, 1.0]); // Gray
            colored_text.push_str("─".repeat(30).as_str(), [0.5, 0.8, 1.0, 1.0]);
            colored_text.push('\n', self.config.colors.text_default);
        }

        // Get the full code and apply scroll offset (view-based scrolling)
        let full_code = self.code_state.get_full_code();
        let display_text = self.apply_scroll_offset(&full_code);

        // Add syntax highlighted code (if enabled in config)
        if self.config.text.syntax_highlighting {
            let syntax_highlighted = ColoredTextDemo::create_syntax_highlighted_rust(&display_text);
            colored_text.chars.extend(syntax_highlighted.chars);
        } else {
            colored_text.push_str(&display_text, self.config.colors.text_default);
        }

        // Add footer with rainbow text (if enabled in config)
        if self.config.text.rainbow_effects {
            colored_text.push('\n', self.config.colors.text_default);
            colored_text.push_str("✨ Rainbow: ", self.config.colors.text_default);
            let rainbow = ColoredTextDemo::create_rainbow_text("Per-character colors work!");
            colored_text.chars.extend(rainbow.chars);
        }

        colored_text
    }

    /// Applies scroll offset to the text (skips first N lines for display)
    /// This is a VIEW operation - it doesn't change the code state
    fn apply_scroll_offset(&self, text: &str) -> String {
        if self.scroll_offset == 0 {
            return text.to_string();
        }

        let mut lines_skipped = 0;
        let mut result = String::new();

        for ch in text.chars() {
            if lines_skipped >= self.scroll_offset {
                result.push(ch);
            } else if ch == '\n' {
                lines_skipped += 1;
            }
        }

        result
    }

    fn initialize_text_system(&mut self) -> Result<()> {
        if self.text_system.is_none() {
            // Define initial text render settings from config
            let initial_settings = TextRenderSettings {
                color: self.config.colors.text_default,
                font_size: self.config.text.font_size,
                position: [self.config.text.position_x, self.config.text.position_y],
            };

            let mut text_system = text::TextSystem::new(
                self.render_engine.device.clone(),
                self.render_engine.queue.clone(),
                self.render_engine.memory_allocator.clone(),
                initial_settings,
            )?;

            // Demonstrate text rendering to console
            info!("Initializing text system and rendering demo code");
            let display_text = self.code_state.get_full_code();
            text_system.rasterize_text_to_console(&display_text)?;

            // Show initial code state
            info!("Initial code state:");
            info!("  Total characters: {}", self.code_state.get_total_length());
            info!("  Progress: {:.1}%", self.code_state.get_progress() * 100.0);
            if let Some(next_char) = self.code_state.peek_next_character() {
                info!("  Next character to type: '{}'", next_char);
            }

            info!("Text system supports configurable colors and positioning");

            // Initialize text pipeline
            info!("Creating text rendering pipeline");
            text_system.create_text_pipeline()?;

            let text_system_arc = Arc::new(Mutex::new(text_system));
            self.render_engine.set_text_system(text_system_arc.clone());
            self.text_system = Some(text_system_arc);
            self.update_text();
        }
        Ok(())
    }

    fn try_initialize_text_pipeline(&mut self) {
        if let Some(text_system_arc) = &self.text_system {
            if let Ok(mut text_system) = text_system_arc.lock() {
                if !text_system.is_pipeline_ready && self.render_engine.is_ready() {
                    if let Some(text_pipeline_layout) =
                        self.render_engine.get_text_pipeline_layout()
                    {
                        if let Err(e) = text_system.create_text_atlas(text_pipeline_layout) {
                            log::error!("Failed to create text atlas: {}", e);
                        } else {
                            info!("Text atlas created successfully");
                        }
                    }
                }
            }
        }
    }
}

impl ApplicationHandler for CargoTapApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.render_engine.resumed(event_loop);
        if let Err(e) = self.initialize_text_system() {
            log::error!("Failed to initialize text system: {}", e);
        }

        // Try to initialize text pipeline if it wasn't created earlier
        self.try_initialize_text_pipeline();
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Save progress before closing
        if let WindowEvent::CloseRequested = &event {
            let position = self.code_state.get_cursor_position();
            self.progress_storage.save_progress(
                self.current_file_path.clone(),
                self.current_file_hash.clone(),
                position,
            );
            if let Err(e) = self.progress_storage.save() {
                log::error!("Failed to save progress on exit: {}", e);
            } else {
                log::info!("Progress saved at position {}", position);
            }
        }

        // Track modifier keys (Command, Ctrl, etc.)
        if let WindowEvent::ModifiersChanged(modifiers) = &event {
            log::info!("Modifiers changed: {:?}", modifiers.state());
            self.input_handler.update_modifiers(modifiers.state());
        }

        // Handle keyboard input before passing to render engine
        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = &event
        {
            // Process the key event with input handler
            self.input_handler.process_key_event(key_event.clone());

            // Check if quit was requested (Command+Q)
            if let Some(input::InputAction::Quit) = self.input_handler.get_last_action() {
                // Save progress before quitting
                let position = self.code_state.get_cursor_position();
                self.progress_storage.save_progress(
                    self.current_file_path.clone(),
                    self.current_file_hash.clone(),
                    position,
                );
                if let Err(e) = self.progress_storage.save() {
                    log::error!("Failed to save progress on quit: {}", e);
                } else {
                    log::info!("Progress saved at position {} before quit", position);
                }
                // Exit the application
                event_loop.exit();
                return;
            }

            // Handle typing logic based on input
            self.handle_typing_input();

            // Update text display
            self.update_text();
        }

        self.render_engine
            .window_event(event_loop, _window_id, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Update session state and text every frame for smooth timer
        if self.session_state.is_active() {
            let current_position = self.code_state.get_cursor_position();
            self.session_state.update(current_position);
            self.update_text();
        }

        self.render_engine.about_to_wait(_event_loop);
    }
}

impl CargoTapApp {
    fn handle_typing_input(&mut self) {
        // Update session state and check if time expired
        let current_position = self.code_state.get_cursor_position();
        let session_just_finished = self.session_state.update(current_position);

        if session_just_finished {
            // Session just finished - stats are logged in session_state.update()
            // User needs to press Space to start a new session
            return;
        }

        // If session is finished, only allow Space to start new session
        if self.session_state.is_finished() {
            if let Some(action) = self.input_handler.get_last_action() {
                match action {
                    input::InputAction::TypeCharacter(' ') => {
                        let current_pos = self.code_state.get_cursor_position();
                        self.session_state.start_new_session(current_pos);
                        info!("Starting new session from position {}", current_pos);
                    }
                    _ => {
                        // Ignore all other input when session is finished
                    }
                }
                self.input_handler.clear_last_action();
            }
            return;
        }

        if let Some(action) = self.input_handler.get_last_action() {
            match action {
                input::InputAction::ScrollDown => {
                    // Scroll down by the configured number of lines (view change only)
                    let scroll_lines = self.config.gameplay.scroll_lines;

                    // Count total lines in the full code
                    let full_code = self.code_state.get_full_code();
                    let total_lines = full_code.chars().filter(|&c| c == '\n').count();

                    // Check if we can scroll more
                    if self.scroll_offset + scroll_lines <= total_lines {
                        self.scroll_offset += scroll_lines;
                        info!(
                            "⬇️ Scrolled view down {} line(s) (offset: {})",
                            scroll_lines, self.scroll_offset
                        );
                    } else if self.scroll_offset < total_lines {
                        // Scroll to the end
                        let lines_scrolled = total_lines - self.scroll_offset;
                        self.scroll_offset = total_lines;
                        info!(
                            "⬇️ Scrolled view down {} line(s) to end (offset: {})",
                            lines_scrolled, self.scroll_offset
                        );
                    } else {
                        info!("⬇️ Already at the end of the code");
                    }

                    // Note: Code state (printed_code) is unchanged - this is view only!
                    if self.config.gameplay.show_statistics {
                        info!(
                            "Typing Progress: {:.1}% ({}/{}) | View Offset: {} lines",
                            self.code_state.get_progress() * 100.0,
                            self.code_state.printed_code.len(),
                            self.code_state.get_total_length(),
                            self.scroll_offset
                        );
                    }
                }
                input::InputAction::SkipCharacter => {
                    // Manual skip triggered by Ctrl+S or Cmd+S
                    if self.config.gameplay.enable_manual_skip {
                        if let Some(expected_char) = self.code_state.peek_next_character() {
                            if let Some(description) =
                                char_utils::get_untypeable_description(expected_char)
                            {
                                info!("⏭️  Manually skipping {}", description);
                            } else {
                                info!("⏭️  Manually skipping character: '{}'", expected_char);
                            }

                            // Skip the character
                            self.code_state.type_character();
                            self.session_state.record_char_typed();
                        } else {
                            info!("⏭️  No character to skip");
                        }
                    } else {
                        if self.config.debug.log_code_state {
                            info!("⛔ Manual skip is disabled in configuration");
                        }
                    }
                }
                input::InputAction::TypeCharacter(typed_char) => {
                    // Start session on first character if not started
                    if !self.session_state.is_active() {
                        let current_pos = self.code_state.get_cursor_position();
                        self.session_state.start(current_pos);
                    }

                    // Check if auto-skip is enabled and current character is untypeable
                    if self.config.gameplay.auto_skip_untypeable {
                        while let Some(expected_char) = self.code_state.peek_next_character() {
                            if !char_utils::is_typeable_on_us_keyboard(expected_char) {
                                // Auto-skip this untypeable character
                                if let Some(description) =
                                    char_utils::get_untypeable_description(expected_char)
                                {
                                    info!("⏭️  Auto-skipping {}", description);
                                }
                                self.code_state.type_character();
                                self.session_state.record_char_typed();
                            } else {
                                break;
                            }
                        }
                    }

                    if let Some(expected_char) = self.code_state.peek_next_character() {
                        if *typed_char == expected_char {
                            // Correct character typed - advance the code
                            let advanced_char = self.code_state.type_character();
                            if let Some(ch) = advanced_char {
                                // Record character in session
                                self.session_state.record_char_typed();
                                if self.config.debug.log_code_state {
                                    info!("✓ Correctly typed: '{}'", ch);
                                }
                                if self.config.gameplay.show_statistics {
                                    info!(
                                        "Progress: {:.1}% ({}/{})",
                                        self.code_state.get_progress() * 100.0,
                                        self.code_state.printed_code.len(),
                                        self.code_state.get_total_length()
                                    );
                                }

                                if self.code_state.is_complete() {
                                    info!("🎉 Code typing completed!");
                                } else if let Some(next_char) =
                                    self.code_state.peek_next_character()
                                {
                                    // Warn if next character is untypeable and auto-skip is disabled
                                    if !self.config.gameplay.auto_skip_untypeable
                                        && !char_utils::is_typeable_on_us_keyboard(next_char)
                                    {
                                        if let Some(description) =
                                            char_utils::get_untypeable_description(next_char)
                                        {
                                            info!("⚠️  Next character is {}", description);
                                            if self.config.gameplay.enable_manual_skip {
                                                info!("💡 Press Ctrl+S (or Cmd+S) to skip it");
                                            }
                                        }
                                    } else if self.config.gameplay.show_next_char_hint {
                                        info!("Next character: '{}'", next_char);
                                    }
                                }
                            }
                        } else {
                            if self.config.debug.log_code_state {
                                info!(
                                    "❌ Incorrect character! Expected '{}', got '{}'",
                                    expected_char, typed_char
                                );
                            }
                        }
                    }
                }
                input::InputAction::Backspace => {
                    // Check if backspace is allowed
                    if !self.config.gameplay.allow_backspace {
                        if self.config.debug.log_code_state {
                            info!("⛔ Backspace is disabled in configuration");
                        }
                        self.input_handler.clear_last_action();
                        return;
                    }

                    // Move character back from printed to current
                    if let Some(ch) = self.code_state.backspace() {
                        // Record backspace in session
                        self.session_state.record_backspace();
                        if self.config.debug.log_code_state {
                            info!("⬅️ Backspace: moved '{}' back to current code", ch);
                        }
                        if self.config.gameplay.show_statistics {
                            info!(
                                "Progress: {:.1}% ({}/{})",
                                self.code_state.get_progress() * 100.0,
                                self.code_state.printed_code.len(),
                                self.code_state.get_total_length()
                            );
                        }
                        if self.config.gameplay.show_next_char_hint {
                            if let Some(next_char) = self.code_state.peek_next_character() {
                                info!("Next character: '{}'", next_char);
                            }
                        }
                    }
                }
                input::InputAction::Enter => {
                    // Start session on first character if not started
                    if !self.session_state.is_active() {
                        let current_pos = self.code_state.get_cursor_position();
                        self.session_state.start(current_pos);
                    }

                    // Handle enter key if it matches expected character
                    if let Some(expected_char) = self.code_state.peek_next_character() {
                        if expected_char == '\n' {
                            let advanced_char = self.code_state.type_character();
                            if let Some(_ch) = advanced_char {
                                // Record character in session
                                self.session_state.record_char_typed();

                                if self.config.debug.log_code_state {
                                    info!("✓ Correctly typed newline");
                                }
                                if self.config.gameplay.show_statistics {
                                    info!(
                                        "Progress: {:.1}% ({}/{})",
                                        self.code_state.get_progress() * 100.0,
                                        self.code_state.printed_code.len(),
                                        self.code_state.get_total_length()
                                    );
                                }
                            }
                        } else {
                            if self.config.debug.log_code_state {
                                info!("❌ Incorrect! Expected '{}', got newline", expected_char);
                            }
                        }
                    }
                }
                input::InputAction::Tab => {
                    // Tab key consumes all whitespace until next non-whitespace character
                    let consumed = self.code_state.consume_whitespace();

                    if consumed > 0 {
                        // Record Tab press as single character in session (regardless of whitespace consumed)
                        self.session_state.record_char_typed();

                        if self.config.debug.log_code_state {
                            info!("⇥ Tab: consumed {} whitespace character(s)", consumed);
                        }
                        if self.config.gameplay.show_statistics {
                            info!(
                                "Progress: {:.1}% ({}/{})",
                                self.code_state.get_progress() * 100.0,
                                self.code_state.printed_code.len(),
                                self.code_state.get_total_length()
                            );
                        }
                        if self.config.gameplay.show_next_char_hint {
                            if let Some(next_char) = self.code_state.peek_next_character() {
                                info!("Next character: '{}'", next_char);
                            }
                        }
                    } else {
                        if self.config.debug.log_code_state {
                            info!("⇥ Tab: no whitespace to consume");
                        }
                    }
                }
                input::InputAction::Quit => {
                    // Quit is handled in window_event before this function is called
                    // This case exists to satisfy the exhaustive match requirement
                }
                input::InputAction::Other => {
                    // Handle other keys if needed
                }
            }

            // Clear the action after processing
            self.input_handler.clear_last_action();
        }
    }
}

fn main() -> Result<()> {
    // Load configuration early to set log level
    let config = config::Config::load();

    // Initialize logger with configured level
    simple_logger::init_with_level(config.get_log_level()).unwrap();

    // Check for special commands
    let args: Vec<String> = std::env::args().collect();

    // Generate default config file if requested
    if args.len() > 1 && args[1] == "gen-config" {
        match config::Config::save_default("config.toml") {
            Ok(_) => {
                println!("✓ Default configuration saved to config.toml");
                println!("  Edit this file to customize CargoTap settings.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("✗ Failed to save config: {}", e);
                return Err(e);
            }
        }
    }

    // Run demo mode if requested
    if args.len() > 1 && args[1] == "demo" {
        demo_code_state::run_demo();
        return Ok(());
    }

    info!("Starting CargoTap application");
    info!("Tip: Run with 'cargo run demo' for command-line demo");
    info!("Tip: Run with 'cargo run gen-config' to generate config.toml");

    let event_loop = EventLoop::new()?;
    let mut app = CargoTapApp::new(&event_loop)?;

    info!("Starting event loop");
    event_loop.run_app(&mut app)?;
    info!("Finished event loop");
    Ok(())
}
