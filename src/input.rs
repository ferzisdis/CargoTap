use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct InputHandler {
    pub current_input: String,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            current_input: String::new(),
        }
    }

    pub fn process_key_event(&mut self, input: KeyEvent) {
        if let PhysicalKey::Code(key) = input.physical_key {
            if input.state == ElementState::Pressed {
                // Обработка специальных клавиш
                match key {
                    KeyCode::Backspace => {
                        self.current_input.pop();
                    }
                    KeyCode::Enter => {
                        self.current_input.push('\n');
                    }
                    // ...
                    _ => {
                        if let Some(c) = input.text {
                            self.current_input.push_str(&c);
                        }
                    }
                }
            }
        }
    }
}
