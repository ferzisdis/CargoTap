use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

#[derive(Debug, Clone)]
pub enum InputAction {
    TypeCharacter(char),
    Backspace,
    Enter,
    ScrollDown,
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
                // Check for Command+J (or Ctrl+J on other platforms) for scrolling
                let is_cmd_or_ctrl = self.modifiers.super_key() || self.modifiers.control_key();

                if key == KeyCode::KeyJ && is_cmd_or_ctrl {
                    self.last_action = Some(InputAction::ScrollDown);
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
