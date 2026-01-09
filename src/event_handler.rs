use log::info;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

use crate::app::CargoTapApp;
use crate::input;
use crate::typing_handler;

impl ApplicationHandler for CargoTapApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.render_engine.resumed(event_loop);
        if let Err(e) = self.initialize_text_system() {
            log::error!("Failed to initialize text system: {}", e);
        }

        self.try_initialize_text_pipeline();
        self.update_text();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = &event {
            self.save_progress();
        }

        if let WindowEvent::ModifiersChanged(modifiers) = &event {
            info!("Modifiers changed: {:?}", modifiers.state());
            self.input_handler.update_modifiers(modifiers.state());
        }

        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = &event
        {
            self.input_handler.process_key_event(key_event.clone());

            if let Some(input::InputAction::Quit) = self.input_handler.get_last_action() {
                if self.show_statistics {
                    self.show_statistics = false;
                    log::info!("📊 Closed statistics screen");
                    self.input_handler.clear_last_action();
                    self.update_text();
                    return;
                }

                self.save_progress();
                event_loop.exit();
                return;
            }

            typing_handler::handle_typing_input(self);
            self.update_text();
        }

        self.render_engine
            .window_event(event_loop, _window_id, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.session_state.is_active() {
            let current_position = self.code_state.get_cursor_position();
            let session_just_finished = self.session_state.update(current_position);

            if session_just_finished {
                log::info!("Session just finished (timer expired)!");
                self.save_session_statistics();
            }

            self.update_text();
        }

        self.render_engine.about_to_wait(_event_loop);
    }
}
