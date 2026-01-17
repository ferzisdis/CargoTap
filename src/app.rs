use anyhow::Result;
use log::info;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::event_loop::EventLoop;

use crate::code_state;
use crate::config;
use crate::input;
use crate::progress_storage;
use crate::renderer;
use crate::session_history;
use crate::session_state;
use crate::text;

pub struct CargoTapApp {
    pub render_engine: renderer::VulkanRenderer,
    pub text_system: Option<Arc<Mutex<text::TextSystem>>>,
    pub input_handler: input::InputHandler,
    pub code_state: code_state::CodeState,
    pub config: config::Config,
    pub scroll_offset: usize,
    pub progress_storage: progress_storage::ProgressStorage,
    pub current_file_path: String,
    pub current_file_hash: String,
    pub session_state: session_state::SessionState,
    pub session_history: session_history::SessionHistory,
    pub show_statistics: bool,
    pub file_selection_mode: bool,
    pub file_input_buffer: String,
    pub frame_times: VecDeque<Instant>,
    pub last_frame_time: Instant,
    pub current_fps: f32,
    pub last_key_processing_time_ms: f64,
    pub text_update_time_ms: f64,
    pub ui_generation_time_ms: f64,
}

impl CargoTapApp {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let config = config::Config::load();

        let warnings = config.validate();
        for warning in warnings {
            log::warn!("Config validation: {}", warning);
        }

        let render_engine = renderer::VulkanRenderer::new(event_loop);
        let input_handler = input::InputHandler::new();

        let mut progress_storage = progress_storage::ProgressStorage::default();
        let _ = progress_storage.load();

        // Determine which file to load: last opened > config > demo
        let file_path = if let Some(last_opened) = progress_storage.get_last_opened_file() {
            log::info!("Restoring last opened file: {}", last_opened);
            last_opened.clone()
        } else if let Some(ref custom_path) = config.gameplay.custom_code_path {
            log::info!("Using config file: {}", custom_path);
            custom_path.clone()
        } else {
            log::info!("Using demo code");
            "demo_code.rs".to_string()
        };

        // Load the file content
        let demo_code = if file_path == "demo_code.rs" {
            include_str!("demo_code.rs").to_string()
        } else {
            match std::fs::read_to_string(&file_path) {
                Ok(code) => {
                    log::info!("Successfully loaded file from: {}", file_path);
                    code
                }
                Err(e) => {
                    log::error!("Failed to load file from {}: {}", file_path, e);
                    log::info!("Falling back to demo code");
                    include_str!("demo_code.rs").to_string()
                }
            }
        };

        let current_file_hash = progress_storage::compute_hash(&demo_code);

        let mut code_state = code_state::CodeState::new(demo_code);

        let mut scroll_offset = 0;
        if let Some(progress) = progress_storage.get_progress(&file_path) {
            if progress.content_hash == current_file_hash {
                log::info!(
                    "Restoring progress at position {} with scroll offset {}",
                    progress.position,
                    progress.scroll_offset
                );
                for _ in 0..progress.position {
                    if code_state.type_character().is_none() {
                        break;
                    }
                }
                scroll_offset = progress.scroll_offset;
            } else {
                log::info!("File changed, starting from beginning");
            }
        }

        let session_state =
            session_state::SessionState::new(config.gameplay.session_duration_minutes);

        let mut session_history = session_history::SessionHistory::default();
        if let Err(e) = session_history.load() {
            log::warn!("Failed to load session history: {}", e);
        } else {
            log::info!("Loaded {} previous sessions", session_history.count());
        }

        // Save the current file as last opened
        progress_storage.set_last_opened_file(file_path.clone());
        if let Err(e) = progress_storage.save() {
            log::error!("Failed to save last opened file: {}", e);
        }

        let now = Instant::now();
        Ok(Self {
            render_engine,
            text_system: None,
            input_handler,
            code_state,
            config,
            scroll_offset,
            progress_storage,
            current_file_path: file_path,
            current_file_hash,
            session_state,
            session_history,
            show_statistics: false,
            file_selection_mode: false,
            file_input_buffer: String::new(),
            frame_times: VecDeque::with_capacity(60),
            last_frame_time: now,
            current_fps: 0.0,
            last_key_processing_time_ms: 0.0,
            text_update_time_ms: 0.0,
            ui_generation_time_ms: 0.0,
        })
    }

    pub fn initialize_text_system(&mut self) -> Result<()> {
        if self.text_system.is_none() {
            let initial_settings = text::TextRenderSettings {
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

            info!("Initializing text system and rendering demo code");
            let display_text = self.code_state.get_full_code();
            text_system.rasterize_text_to_console(&display_text)?;

            info!("Initial code state:");
            info!("  Total characters: {}", self.code_state.get_total_length());
            info!("  Progress: {:.1}%", self.code_state.get_progress() * 100.0);
            if let Some(next_char) = self.code_state.peek_next_character() {
                info!("  Next character to type: '{}'", next_char);
            }

            info!("Text system supports configurable colors and positioning");

            info!("Creating text rendering pipeline");
            text_system.create_text_pipeline()?;

            let text_system_arc = Arc::new(Mutex::new(text_system));
            self.render_engine.set_text_system(text_system_arc.clone());
            self.text_system = Some(text_system_arc);
            self.update_text();
        }
        Ok(())
    }

    pub fn try_initialize_text_pipeline(&mut self) {
        let mut atlas_created = false;
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
                            atlas_created = true;
                        }
                    }
                }
            }
        }

        if atlas_created {
            self.update_text();
        }
    }

    pub fn update_text(&mut self) {
        let start = Instant::now();

        // Update the syntax highlighting cache in code_state
        self.code_state
            .set_syntax_highlighting(self.config.text.syntax_highlighting);
        self.code_state.update_colored_cache();

        let ui_start = Instant::now();
        let colored_text = crate::ui::create_colored_text(self);
        self.ui_generation_time_ms = ui_start.elapsed().as_secs_f64() * 1000.0;

        if let Some(ref text_system) = self.text_system {
            if let Ok(mut text_system) = text_system.lock() {
                if let Err(e) = text_system.update_text_with_settings(&colored_text) {
                    log::error!("Failed to update main text: {}", e);
                }
            }
        }

        self.text_update_time_ms = start.elapsed().as_secs_f64() * 1000.0;
    }

    pub fn save_session_statistics(&mut self) -> bool {
        if let Some(stats) = self.session_state.last_stats() {
            self.session_history.add_session(stats.clone());
            if let Err(e) = self.session_history.save() {
                log::error!("Failed to save session history: {}", e);
                return false;
            } else {
                log::info!(
                    "✅ Session saved to history (total: {})",
                    self.session_history.count()
                );
                return true;
            }
        }
        false
    }

    pub fn save_progress(&mut self) {
        let position = self.code_state.get_cursor_position();
        self.progress_storage.save_progress_with_scroll_offset(
            self.current_file_path.clone(),
            self.current_file_hash.clone(),
            position,
            self.scroll_offset,
        );
        if let Err(e) = self.progress_storage.save() {
            log::error!("Failed to save progress: {}", e);
        } else {
            log::info!(
                "Progress saved at position {} with scroll offset {}",
                position,
                self.scroll_offset
            );
        }
    }

    pub fn load_file(&mut self, file_path: String) -> Result<()> {
        let code = match std::fs::read_to_string(&file_path) {
            Ok(code) => {
                log::info!("Successfully loaded file: {}", file_path);
                code
            }
            Err(e) => {
                log::error!("Failed to load file {}: {}", file_path, e);
                return Err(anyhow::anyhow!("Failed to load file: {}", e));
            }
        };

        self.save_progress();

        let new_file_hash = progress_storage::compute_hash(&code);
        self.current_file_path = file_path.clone();
        self.current_file_hash = new_file_hash;

        self.progress_storage
            .set_last_opened_file(file_path.clone());
        if let Err(e) = self.progress_storage.save() {
            log::error!("Failed to save last opened file: {}", e);
        }

        self.code_state = code_state::CodeState::new(code);
        self.scroll_offset = 0;

        if let Some(progress) = self.progress_storage.get_progress(&file_path) {
            if progress.content_hash == self.current_file_hash {
                log::info!(
                    "Restoring progress at position {} with scroll offset {}",
                    progress.position,
                    progress.scroll_offset
                );
                for _ in 0..progress.position {
                    if self.code_state.type_character().is_none() {
                        break;
                    }
                }
                self.scroll_offset = progress.scroll_offset;
            } else {
                log::info!("File changed, starting from beginning");
            }
        }

        let current_pos = self.code_state.get_cursor_position();
        self.session_state.start_new_session(current_pos, file_path);

        Ok(())
    }

    pub fn update_frame_time(&mut self) {
        let now = Instant::now();
        self.frame_times.push_back(now);

        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }

        if self.frame_times.len() >= 2 {
            let oldest = self.frame_times.front().unwrap();
            let duration = now.duration_since(*oldest);
            let frames = self.frame_times.len() as f32;
            self.current_fps = frames / duration.as_secs_f32();
        }

        self.last_frame_time = now;
    }
}
