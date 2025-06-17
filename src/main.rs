use anyhow::Result;
use log::info;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

mod input;
mod renderer;
mod text;

use text::TextRenderSettings;

// Главная структура приложения
pub struct CargoTapApp {
    render_engine: renderer::VulkanRenderer,
    text_system: Option<Arc<Mutex<text::TextSystem>>>,
    input_handler: input::InputHandler,
    current_code: String,
}

impl CargoTapApp {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let render_engine = renderer::VulkanRenderer::new(event_loop);
        let input_handler = input::InputHandler::new();

        // Загрузка демо-кода вместо GitHub API на старте
        let current_code = include_str!("demo_code.rs").to_string();

        Ok(Self {
            render_engine,
            text_system: None,
            input_handler,
            current_code,
        })
    }

    // pub fn run(mut self) -> Result<()> {
    //     event_loop.run(move |event, _, control_flow| {
    //         *control_flow = ControlFlow::Poll;

    //         match event {
    //             Event::WindowEvent { event, .. } => match event {
    //                 WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
    //                 WindowEvent::Resized(_) => self.render_engine.recreate_swapchain = true,
    //                 WindowEvent::KeyboardInput { input, .. } => {
    //                     self.input_handler.process_key_event(input);
    //                     // Обработка ввода и обновление текста
    //                     self.update_text();
    //                 }
    //                 _ => (),
    //             },
    //             Event::MainEventsCleared => {
    //                 self.render();
    //             }
    //             _ => (),
    //         }
    //     });

    //     Ok(())
    // }

    fn update_text(&mut self) {
        // Demonstrate multiple text blocks with different colors and positions
        if let Some(ref text_system) = self.text_system {
            if let Ok(mut text_system) = text_system.lock() {
                if let Err(e) = text_system.update_text_with_settings(&self.current_code) {
                    log::error!("Failed to update main text: {}", e);
                }
            }
        }
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
            text_system.rasterize_text_to_console(&self.current_code)?;

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
        match event {
            WindowEvent::KeyboardInput { .. } => {
                // Handle input and update text
                self.update_text();
            }
            _ => {}
        }

        self.render_engine
            .window_event(event_loop, _window_id, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.render_engine.about_to_wait(_event_loop);
    }
}

fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    info!("Starting CargoTap application");
    info!("Loading demo code from demo_code.rs");

    // Display the demo code content
    let demo_code = include_str!("demo_code.rs");
    info!("Demo code content:\n{}", demo_code);

    let event_loop = EventLoop::new()?;
    let mut app = CargoTapApp::new(&event_loop)?;

    info!("Starting event loop");
    event_loop.run_app(&mut app)?;

    Ok(())
}
