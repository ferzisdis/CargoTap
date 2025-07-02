use anyhow::Result;
use log::info;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

mod code_state;
mod demo_code_state;
mod input;
mod renderer;
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
}

impl CargoTapApp {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let render_engine = renderer::VulkanRenderer::new(event_loop);
        let input_handler = input::InputHandler::new();

        // Загрузка демо-кода вместо GitHub API на старте
        let demo_code = include_str!("demo_code.rs").to_string();
        let code_state = code_state::CodeState::new(demo_code);

        Ok(Self {
            render_engine,
            text_system: None,
            input_handler,
            code_state,
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

        // Add a colorful header
        colored_text.push_str("🦀 CargoTap ", [1.0, 0.5, 0.0, 1.0]); // Orange
        colored_text.push_str("Live Demo\n", [0.0, 1.0, 1.0, 1.0]); // Cyan
        colored_text.push_str("─".repeat(30).as_str(), [0.5, 0.8, 1.0, 1.0]); // Light blue
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);

        // Add syntax highlighted code
        let display_text = self.code_state.get_full_code();
        let syntax_highlighted = ColoredTextDemo::create_syntax_highlighted_rust(&display_text);
        colored_text.chars.extend(syntax_highlighted.chars);

        // Add footer with rainbow text
        colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);
        colored_text.push_str("✨ Rainbow: ", [1.0, 1.0, 1.0, 1.0]);
        let rainbow = ColoredTextDemo::create_rainbow_text("Per-character colors work!");
        colored_text.chars.extend(rainbow.chars);

        colored_text
    }

    fn initialize_text_system(&mut self) -> Result<()> {
        if self.text_system.is_none() {
            // Define initial text render settings
            let initial_settings = TextRenderSettings {
                color: [0.9, 0.9, 0.9, 1.0], // Light gray
                font_size: 64.0,
                position: [20.0, 50.0],
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
        // Handle keyboard input before passing to render engine
        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = &event
        {
            // Process the key event with input handler
            self.input_handler.process_key_event(key_event.clone());

            // Handle typing logic based on input
            self.handle_typing_input();

            // Update text display
            self.update_text();
        }

        self.render_engine
            .window_event(event_loop, _window_id, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.render_engine.about_to_wait(_event_loop);
    }
}

impl CargoTapApp {
    fn handle_typing_input(&mut self) {
        if let Some(action) = self.input_handler.get_last_action() {
            match action {
                input::InputAction::TypeCharacter(typed_char) => {
                    if let Some(expected_char) = self.code_state.peek_next_character() {
                        if *typed_char == expected_char {
                            // Correct character typed - advance the code
                            let advanced_char = self.code_state.type_character();
                            if let Some(ch) = advanced_char {
                                info!("✓ Correctly typed: '{}'", ch);
                                info!(
                                    "Progress: {:.1}% ({}/{})",
                                    self.code_state.get_progress() * 100.0,
                                    self.code_state.printed_code.len(),
                                    self.code_state.get_total_length()
                                );

                                if self.code_state.is_complete() {
                                    info!("🎉 Code typing completed!");
                                } else if let Some(next_char) =
                                    self.code_state.peek_next_character()
                                {
                                    info!("Next character: '{}'", next_char);
                                }
                            }
                        } else {
                            info!(
                                "❌ Incorrect character! Expected '{}', got '{}'",
                                expected_char, typed_char
                            );
                        }
                    }
                }
                input::InputAction::Backspace => {
                    // Move character back from printed to current
                    if let Some(ch) = self.code_state.backspace() {
                        info!("⬅️ Backspace: moved '{}' back to current code", ch);
                        info!(
                            "Progress: {:.1}% ({}/{})",
                            self.code_state.get_progress() * 100.0,
                            self.code_state.printed_code.len(),
                            self.code_state.get_total_length()
                        );
                        if let Some(next_char) = self.code_state.peek_next_character() {
                            info!("Next character: '{}'", next_char);
                        }
                    }
                }
                input::InputAction::Enter => {
                    // Handle enter key if it matches expected character
                    if let Some(expected_char) = self.code_state.peek_next_character() {
                        if expected_char == '\n' {
                            let advanced_char = self.code_state.type_character();
                            if let Some(_ch) = advanced_char {
                                info!("✓ Correctly typed newline");
                                info!(
                                    "Progress: {:.1}% ({}/{})",
                                    self.code_state.get_progress() * 100.0,
                                    self.code_state.printed_code.len(),
                                    self.code_state.get_total_length()
                                );
                            }
                        } else {
                            info!("❌ Incorrect! Expected '{}', got newline", expected_char);
                        }
                    }
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
    simple_logger::init_with_level(log::Level::Info).unwrap();

    // Check for demo argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "demo" {
        demo_code_state::run_demo();
        return Ok(());
    }

    info!("Starting CargoTap application");
    info!("Loading demo code from demo_code.rs");
    info!("Tip: Run with 'cargo run demo' for command-line demo");

    // Display the demo code content
    let demo_code = include_str!("demo_code.rs");
    info!("Demo code content:\n{}", demo_code);

    let event_loop = EventLoop::new()?;
    let mut app = CargoTapApp::new(&event_loop)?;

    info!("Starting event loop");
    event_loop.run_app(&mut app)?;

    Ok(())
}
