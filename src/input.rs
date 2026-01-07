use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

#[derive(Debug, Clone)]
pub enum InputAction {
    TypeCharacter(char),
    Backspace,
    Enter,
    Tab,
    ScrollDown,
    ScrollUp,
    SkipCharacter,
    ShowStatistics,
    Quit,
    Other,
}

pub struct InputHandler {
    pub current_input: String,
    pub last_action: Option<InputAction>,
    pub modifiers: ModifiersState,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            current_input: String::new(),
            last_action: None,
            modifiers: ModifiersState::empty(),
        }
    }

    pub fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    pub fn process_key_event(&mut self, input: KeyEvent) {
        self.last_action = None;

        if let PhysicalKey::Code(key) = input.physical_key {
            if input.state == ElementState::Pressed {
                // Check for Command+Q to quit (may be intercepted by macOS)
                if key == KeyCode::KeyQ && self.modifiers.super_key() {
                    log::info!("Command+Q detected!");
                    self.last_action = Some(InputAction::Quit);
                    return;
                }

                // Check for Command+W to quit (better macOS support)
                if key == KeyCode::KeyW && self.modifiers.super_key() {
                    log::info!("Command+W detected - quitting!");
                    self.last_action = Some(InputAction::Quit);
                    return;
                }

                // Also check for Escape as an alternative quit method OR to close statistics
                if key == KeyCode::Escape {
                    // First check if we're showing stats - if so, just close them
                    // Otherwise, quit the application
                    // Note: This will be handled in the main application logic
                    self.last_action = Some(InputAction::Quit);
                    return;
                }

                // Check for Command+J (or Ctrl+J on other platforms) for scrolling
                let is_cmd_or_ctrl = self.modifiers.super_key() || self.modifiers.control_key();

                // Check for 'T' key with modifiers to show statistics
                if key == KeyCode::KeyT && is_cmd_or_ctrl {
                    self.last_action = Some(InputAction::ShowStatistics);
                    return;
                }

                if key == KeyCode::KeyJ && is_cmd_or_ctrl {
                    self.last_action = Some(InputAction::ScrollDown);
                    return;
                }

                // Check for Command+K (or Ctrl+K on other platforms) for scrolling up
                if key == KeyCode::KeyK && is_cmd_or_ctrl {
                    self.last_action = Some(InputAction::ScrollUp);
                    return;
                }

                // Check for Command+S (or Ctrl+S) for skipping current character
                if key == KeyCode::KeyS && is_cmd_or_ctrl {
                    self.last_action = Some(InputAction::SkipCharacter);
                    return;
                }

                // Обработка специальных клавиш
                match key {
                    KeyCode::Backspace => {
                        self.current_input.pop();
                        self.last_action = Some(InputAction::Backspace);
                    }
                    KeyCode::Enter => {
                        self.current_input.push('\n');
                        self.last_action = Some(InputAction::Enter);
                    }
                    KeyCode::Tab => {
                        self.last_action = Some(InputAction::Tab);
                    }
                    _ => {
                        if let Some(text) = &input.text {
                            if let Some(ch) = text.chars().next() {
                                self.current_input.push(ch);
                                self.last_action = Some(InputAction::TypeCharacter(ch));
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_last_action(&self) -> Option<&InputAction> {
        self.last_action.as_ref()
    }

    pub fn clear_last_action(&mut self) {
        self.last_action = None;
    }
}
