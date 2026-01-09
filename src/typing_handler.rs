use log::info;

use crate::app::CargoTapApp;
use crate::char_utils;
use crate::input;

pub fn handle_typing_input(app: &mut CargoTapApp) {
    let current_position = app.code_state.get_cursor_position();
    let session_just_finished = app.session_state.update(current_position);

    if session_just_finished {
        log::info!("Session just finished!");
        app.save_session_statistics();
        return;
    }

    if app.session_state.is_finished() {
        handle_finished_session(app);
        return;
    }

    if let Some(action) = app.input_handler.get_last_action() {
        match action {
            input::InputAction::ScrollDown => handle_scroll_down(app),
            input::InputAction::ScrollUp => handle_scroll_up(app),
            input::InputAction::SkipCharacter => handle_skip_character(app),
            input::InputAction::TypeCharacter(typed_char) => {
                handle_type_character(app, *typed_char)
            }
            input::InputAction::Backspace => handle_backspace(app),
            input::InputAction::Enter => handle_enter(app),
            input::InputAction::Tab => handle_tab(app),
            input::InputAction::ShowStatistics => handle_show_statistics(app),
            input::InputAction::Quit | input::InputAction::Other => {}
        }

        app.input_handler.clear_last_action();
    }
}

fn handle_finished_session(app: &mut CargoTapApp) {
    if let Some(action) = app.input_handler.get_last_action() {
        match action {
            input::InputAction::TypeCharacter(' ') => {
                let current_pos = app.code_state.get_cursor_position();
                app.session_state
                    .start_new_session(current_pos, app.current_file_path.clone());
                info!("Starting new session from position {}", current_pos);
                app.input_handler.clear_last_action();
            }
            input::InputAction::ShowStatistics => {
                handle_show_statistics(app);
                app.input_handler.clear_last_action();
            }
            input::InputAction::ScrollDown => {
                handle_scroll_down(app);
                app.input_handler.clear_last_action();
            }
            input::InputAction::ScrollUp => {
                handle_scroll_up(app);
                app.input_handler.clear_last_action();
            }
            _ => {
                app.input_handler.clear_last_action();
            }
        }
    }
}

fn handle_scroll_down(app: &mut CargoTapApp) {
    let scroll_lines = app.config.gameplay.scroll_lines;
    let full_code = app.code_state.get_full_code();
    let total_lines = full_code.chars().filter(|&c| c == '\n').count();

    if app.scroll_offset + scroll_lines <= total_lines {
        app.scroll_offset += scroll_lines;
        info!(
            "⬇️ Scrolled view down {} line(s) (offset: {})",
            scroll_lines, app.scroll_offset
        );
    } else if app.scroll_offset < total_lines {
        let lines_scrolled = total_lines - app.scroll_offset;
        app.scroll_offset = total_lines;
        info!(
            "⬇️ Scrolled view down {} line(s) to end (offset: {})",
            lines_scrolled, app.scroll_offset
        );
    } else {
        info!("⬇️ Already at the end of the code");
    }

    if app.config.gameplay.show_statistics {
        info!(
            "Typing Progress: {:.1}% ({}/{}) | View Offset: {} lines",
            app.code_state.get_progress() * 100.0,
            app.code_state.printed_code.len(),
            app.code_state.get_total_length(),
            app.scroll_offset
        );
    }
}

fn handle_scroll_up(app: &mut CargoTapApp) {
    let scroll_lines = app.config.gameplay.scroll_lines;

    if app.scroll_offset >= scroll_lines {
        app.scroll_offset -= scroll_lines;
        info!(
            "⬆️ Scrolled view up {} line(s) (offset: {})",
            scroll_lines, app.scroll_offset
        );
    } else if app.scroll_offset > 0 {
        let lines_scrolled = app.scroll_offset;
        app.scroll_offset = 0;
        info!(
            "⬆️ Scrolled view up {} line(s) to beginning (offset: {})",
            lines_scrolled, app.scroll_offset
        );
    } else {
        info!("⬆️ Already at the beginning of the code");
    }

    if app.config.gameplay.show_statistics {
        info!(
            "Typing Progress: {:.1}% ({}/{}) | View Offset: {} lines",
            app.code_state.get_progress() * 100.0,
            app.code_state.printed_code.len(),
            app.code_state.get_total_length(),
            app.scroll_offset
        );
    }
}

fn handle_skip_character(app: &mut CargoTapApp) {
    if app.config.gameplay.enable_manual_skip {
        if let Some(expected_char) = app.code_state.peek_next_character() {
            if let Some(description) = char_utils::get_untypeable_description(expected_char) {
                info!("⏭️  Manually skipping {}", description);
            } else {
                info!("⏭️  Manually skipping character: '{}'", expected_char);
            }

            app.code_state.type_character();
            app.session_state.record_char_typed();
        } else {
            info!("⏭️  No character to skip");
        }
    } else if app.config.debug.log_code_state {
        info!("⛔ Manual skip is disabled in configuration");
    }
}

fn handle_type_character(app: &mut CargoTapApp, typed_char: char) {
    if !app.session_state.is_active() {
        let current_pos = app.code_state.get_cursor_position();
        app.session_state
            .start(current_pos, app.current_file_path.clone());
    }

    if app.config.gameplay.auto_skip_untypeable {
        while let Some(expected_char) = app.code_state.peek_next_character() {
            if !char_utils::is_typeable_on_us_keyboard(expected_char) {
                if let Some(description) = char_utils::get_untypeable_description(expected_char) {
                    info!("⏭️  Auto-skipping {}", description);
                }
                app.code_state.type_character();
                app.session_state.record_char_typed();
            } else {
                break;
            }
        }
    }

    if let Some(expected_char) = app.code_state.peek_next_character() {
        if typed_char == expected_char {
            let advanced_char = app.code_state.type_character();
            if let Some(ch) = advanced_char {
                app.session_state.record_char_typed();
                if app.config.debug.log_code_state {
                    info!("✓ Correctly typed: '{}'", ch);
                }
                if app.config.gameplay.show_statistics {
                    info!(
                        "Progress: {:.1}% ({}/{})",
                        app.code_state.get_progress() * 100.0,
                        app.code_state.printed_code.len(),
                        app.code_state.get_total_length()
                    );
                }

                if app.code_state.is_complete() {
                    info!("🎉 Code typing completed!");
                } else if let Some(next_char) = app.code_state.peek_next_character() {
                    if !app.config.gameplay.auto_skip_untypeable
                        && !char_utils::is_typeable_on_us_keyboard(next_char)
                    {
                        if let Some(description) = char_utils::get_untypeable_description(next_char)
                        {
                            info!("⚠️  Next character is {}", description);
                            if app.config.gameplay.enable_manual_skip {
                                info!("💡 Press Ctrl+S (or Cmd+S) to skip it");
                            }
                        }
                    } else if app.config.gameplay.show_next_char_hint {
                        info!("Next character: '{}'", next_char);
                    }
                }
            }
        } else if app.config.debug.log_code_state {
            info!(
                "❌ Incorrect character! Expected '{}', got '{}'",
                expected_char, typed_char
            );
        }
    }
}

fn handle_backspace(app: &mut CargoTapApp) {
    if !app.config.gameplay.allow_backspace {
        if app.config.debug.log_code_state {
            info!("⛔ Backspace is disabled in configuration");
        }
        return;
    }

    if let Some(ch) = app.code_state.backspace() {
        app.session_state.record_backspace();
        if app.config.debug.log_code_state {
            info!("⬅️ Backspace: moved '{}' back to current code", ch);
        }
        if app.config.gameplay.show_statistics {
            info!(
                "Progress: {:.1}% ({}/{})",
                app.code_state.get_progress() * 100.0,
                app.code_state.printed_code.len(),
                app.code_state.get_total_length()
            );
        }
        if app.config.gameplay.show_next_char_hint {
            if let Some(next_char) = app.code_state.peek_next_character() {
                info!("Next character: '{}'", next_char);
            }
        }
    }
}

fn handle_enter(app: &mut CargoTapApp) {
    if !app.session_state.is_active() {
        let current_pos = app.code_state.get_cursor_position();
        app.session_state
            .start(current_pos, app.current_file_path.clone());
    }

    if let Some(expected_char) = app.code_state.peek_next_character() {
        if expected_char == '\n' {
            let advanced_char = app.code_state.type_character();
            if advanced_char.is_some() {
                app.session_state.record_char_typed();

                if app.config.debug.log_code_state {
                    info!("✓ Correctly typed newline");
                }
                if app.config.gameplay.show_statistics {
                    info!(
                        "Progress: {:.1}% ({}/{})",
                        app.code_state.get_progress() * 100.0,
                        app.code_state.printed_code.len(),
                        app.code_state.get_total_length()
                    );
                }
            }
        } else if app.config.debug.log_code_state {
            info!("❌ Incorrect! Expected '{}', got newline", expected_char);
        }
    }
}

fn handle_tab(app: &mut CargoTapApp) {
    let consumed = app.code_state.consume_whitespace();

    if consumed > 0 {
        app.session_state.record_char_typed();

        if app.config.debug.log_code_state {
            info!("⇥ Tab: consumed {} whitespace character(s)", consumed);
        }
        if app.config.gameplay.show_statistics {
            info!(
                "Progress: {:.1}% ({}/{})",
                app.code_state.get_progress() * 100.0,
                app.code_state.printed_code.len(),
                app.code_state.get_total_length()
            );
        }
        if app.config.gameplay.show_next_char_hint {
            if let Some(next_char) = app.code_state.peek_next_character() {
                info!("Next character: '{}'", next_char);
            }
        }
    } else if app.config.debug.log_code_state {
        info!("⇥ Tab: no whitespace to consume");
    }
}

fn handle_show_statistics(app: &mut CargoTapApp) {
    app.show_statistics = !app.show_statistics;
    if app.show_statistics {
        info!("📊 Showing statistics screen");
    } else {
        info!("📊 Hiding statistics screen");
    }
}
