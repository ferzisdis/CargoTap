use crate::app::CargoTapApp;
use crate::examples::colored_text_demo::ColoredTextDemo;
use crate::text::ColoredText;

fn add_line_numbers_to_colored_text(
    colored_text: &mut ColoredText,
    text: &str,
    line_number_color: [f32; 4],
    current_line_color: [f32; 4],
    separator_color: [f32; 4],
    scroll_offset: usize,
    current_line: Option<usize>,
    cursor_position: usize,
    caret_bg_color: [f32; 4],
) {
    let lines: Vec<&str> = text.split('\n').collect();
    let total_lines = lines.len();
    let start_line = scroll_offset + 1;
    let num_digits = (total_lines + scroll_offset).to_string().len().max(3);

    for (i, line) in lines.iter().enumerate() {
        let line_num = start_line + i;
        let line_num_str = format!("{:>width$}", line_num, width = num_digits);

        let is_current = current_line.map_or(false, |cl| cl == line_num);
        let num_color = if is_current {
            current_line_color
        } else {
            line_number_color
        };

        for ch in line_num_str.chars() {
            colored_text.push(ch, num_color);
        }

        colored_text.push(' ', separator_color);
        colored_text.push('│', separator_color);
        colored_text.push(' ', separator_color);

        let mut char_index = 0;
        for ch in line.chars() {
            if char_index == cursor_position {
                colored_text.push_with_background(ch, [1.0, 1.0, 1.0, 1.0], caret_bg_color);
            } else {
                colored_text.push(ch, [1.0, 1.0, 1.0, 1.0]);
            }
            char_index += ch.len_utf8();
        }

        if i < lines.len() - 1 {
            colored_text.push('\n', [1.0, 1.0, 1.0, 1.0]);
        }
    }
}

pub fn create_colored_text(app: &CargoTapApp) -> ColoredText {
    if app.file_selection_mode {
        return create_file_selection_screen(app);
    }

    if app.show_statistics {
        return create_statistics_screen(app);
    }

    let mut colored_text = ColoredText::new();

    colored_text.push_str("🦀 CargoTap ", [1.0, 0.5, 0.0, 1.0]);
    colored_text.push_str("Live Demo\n", app.config.colors.text_header);

    colored_text.push_str(
        &format!("📄 File: {} ", app.current_file_path),
        [0.5, 1.0, 1.0, 1.0],
    );

    let progress_percent = app.code_state.get_progress() * 100.0;
    let cursor_pos = app.code_state.get_cursor_position();
    let total_len = app.code_state.get_total_length();
    colored_text.push_str(
        &format!(
            "| Progress: {}/{} ({:.1}%)\n",
            cursor_pos, total_len, progress_percent
        ),
        [0.0, 1.0, 0.5, 1.0],
    );

    if app.config.debug.show_profiling_info {
        colored_text.push_str(
            &format!(
                "📊 FPS: {:.1} | Key: {:.3}ms | Text Update: {:.3}ms | UI Gen: {:.3}ms\n",
                app.current_fps,
                app.last_key_processing_time_ms,
                app.text_update_time_ms,
                app.ui_generation_time_ms
            ),
            [0.8, 0.8, 0.8, 1.0],
        );
    } else {
        colored_text.push_str(
            &format!(
                "📊 FPS: {:.1} | Key Processing: {:.3} ms\n",
                app.current_fps, app.last_key_processing_time_ms
            ),
            [0.8, 0.8, 0.8, 1.0],
        );
    }

    colored_text.push_str("─".repeat(50).as_str(), [0.5, 0.8, 1.0, 1.0]);
    colored_text.push('\n', app.config.colors.text_default);

    if app.session_state.is_finished() {
        if let Some(stats) = app.session_state.last_stats() {
            colored_text.push_str("⏰ SESSION COMPLETE! ", [1.0, 1.0, 0.0, 1.0]);
            colored_text.push('\n', app.config.colors.text_default);

            let summary = format!(
                "Time: {:.1}s | Chars: {} | Speed: {:.0} CPM / {:.0} WPM | Accuracy: {:.1}%\n",
                stats.time_elapsed_secs,
                stats.chars_typed,
                stats.chars_per_minute,
                stats.words_per_minute,
                stats.accuracy
            );
            colored_text.push_str(&summary, [0.0, 1.0, 0.0, 1.0]);
            colored_text.push_str("Press SPACE to start new session\n", [0.0, 1.0, 1.0, 1.0]);
            colored_text.push_str("─".repeat(30).as_str(), [0.5, 0.8, 1.0, 1.0]);
            colored_text.push('\n', app.config.colors.text_default);
        }
    } else if app.session_state.is_active() {
        let time_str = format!("⏱️  Time: {} ", app.session_state.format_time_remaining());
        colored_text.push_str(&time_str, [1.0, 1.0, 0.0, 1.0]);

        let current_pos = app.code_state.get_cursor_position();
        let stats = app.session_state.current_stats(current_pos);
        if stats.time_elapsed_secs > 0.0 {
            let speed_str = format!("| {:.0} CPM ", stats.chars_per_minute);
            colored_text.push_str(&speed_str, [0.0, 1.0, 0.0, 1.0]);
        }
        colored_text.push('\n', app.config.colors.text_default);
        colored_text.push_str("─".repeat(30).as_str(), [0.5, 0.8, 1.0, 1.0]);
        colored_text.push('\n', app.config.colors.text_default);
    } else {
        colored_text.push_str("Start typing to begin session...\n", [0.7, 0.7, 0.7, 1.0]);
        colored_text.push_str("─".repeat(30).as_str(), [0.5, 0.8, 1.0, 1.0]);
        colored_text.push('\n', app.config.colors.text_default);
    }

    let cursor_position = app.code_state.get_cursor_position();

    // Get the cached colored text for the full code
    let full_code_colored = app.code_state.get_full_code_colored();

    // Apply scroll offset to the colored text
    let display_colored = apply_scroll_offset_colored(&full_code_colored, app.scroll_offset);

    // Calculate cursor position in display text (after scroll offset)
    let cursor_position_in_display = if app.scroll_offset == 0 {
        cursor_position
    } else {
        // Count bytes skipped by scroll offset
        let mut bytes_skipped = 0;
        let mut lines_skipped = 0;
        for colored_char in &full_code_colored.chars {
            if lines_skipped >= app.scroll_offset {
                break;
            }
            bytes_skipped += colored_char.ch.len_utf8();
            if colored_char.ch == '\n' {
                lines_skipped += 1;
            }
        }
        cursor_position.saturating_sub(bytes_skipped)
    };

    if app.config.text.show_line_numbers {
        let line_number_color = [0.5, 0.5, 0.6, 1.0];
        let current_line_color = [1.0, 0.85, 0.2, 1.0]; // Bright yellow/gold
        let separator_color = [0.4, 0.4, 0.5, 1.0];

        let current_line = Some(app.code_state.get_cursor_line());
        let caret_bg_color = [0.0, 1.0, 0.0, 0.5]; // Semi-transparent green overlay for caret

        let mut display_with_caret = display_colored.clone();

        // Apply caret background to the character at cursor position
        if cursor_position_in_display < display_with_caret.chars.len() {
            display_with_caret.chars[cursor_position_in_display].background_color =
                Some(caret_bg_color);
        } else if cursor_position_in_display == display_with_caret.chars.len() {
            // Cursor at end of text - add a space with background
            display_with_caret.chars.push(crate::text::ColoredChar {
                ch: ' ',
                color: app.config.colors.text_default,
                background_color: Some(caret_bg_color),
            });
        }

        // Count lines in display text for line numbering
        let line_count = display_with_caret
            .chars
            .iter()
            .filter(|c| c.ch == '\n')
            .count()
            + 1;
        let start_line = app.scroll_offset + 1;
        let num_digits = (line_count + app.scroll_offset).to_string().len().max(3);

        let mut char_index = 0;
        let mut current_line_num = start_line;

        // Add first line number
        let line_num_str = format!("{:>width$}", current_line_num, width = num_digits);
        let is_current = current_line.map_or(false, |cl| cl == current_line_num);
        let num_color = if is_current {
            current_line_color
        } else {
            line_number_color
        };

        for ch in line_num_str.chars() {
            colored_text.push(ch, num_color);
        }
        colored_text.push(' ', separator_color);
        colored_text.push('│', separator_color);
        colored_text.push(' ', separator_color);

        // Add colored code with line numbers
        for colored_char in &display_with_caret.chars {
            if colored_char.background_color.is_some() {
                colored_text.push_with_background(
                    colored_char.ch,
                    colored_char.color,
                    colored_char.background_color.unwrap(),
                );
            } else {
                colored_text.push(colored_char.ch, colored_char.color);
            }

            if colored_char.ch == '\n' && char_index + 1 < display_with_caret.chars.len() {
                // Add line number for next line
                current_line_num += 1;
                let line_num_str = format!("{:>width$}", current_line_num, width = num_digits);
                let is_current = current_line.map_or(false, |cl| cl == current_line_num);
                let num_color = if is_current {
                    current_line_color
                } else {
                    line_number_color
                };

                for ch in line_num_str.chars() {
                    colored_text.push(ch, num_color);
                }
                colored_text.push(' ', separator_color);
                colored_text.push('│', separator_color);
                colored_text.push(' ', separator_color);
            }

            char_index += 1;
        }
    } else {
        let caret_bg_color = [0.0, 1.0, 0.0, 0.5]; // Semi-transparent green overlay for caret

        let mut display_with_caret = display_colored.clone();

        // Apply caret background to the character at cursor position
        if cursor_position_in_display < display_with_caret.chars.len() {
            display_with_caret.chars[cursor_position_in_display].background_color =
                Some(caret_bg_color);
        } else if cursor_position_in_display == display_with_caret.chars.len() {
            // Cursor at end of text - add a space with background
            display_with_caret.chars.push(crate::text::ColoredChar {
                ch: ' ',
                color: app.config.colors.text_default,
                background_color: Some(caret_bg_color),
            });
        }

        for colored_char in &display_with_caret.chars {
            if colored_char.background_color.is_some() {
                colored_text.push_with_background(
                    colored_char.ch,
                    colored_char.color,
                    colored_char.background_color.unwrap(),
                );
            } else {
                colored_text.push(colored_char.ch, colored_char.color);
            }
        }
    }

    if app.config.text.rainbow_effects {
        colored_text.push('\n', app.config.colors.text_default);
        colored_text.push_str("✨ Rainbow: ", app.config.colors.text_default);
        let rainbow = ColoredTextDemo::create_rainbow_text("Per-character colors work!");
        colored_text.chars.extend(rainbow.chars);
    }

    colored_text.push('\n', app.config.colors.text_default);
    colored_text.push_str(
        "Press Cmd+P to change file | Press Ctrl+T / Cmd+T to view statistics",
        [0.5, 0.5, 0.5, 1.0],
    );

    colored_text
}

pub fn create_statistics_screen(app: &CargoTapApp) -> ColoredText {
    let mut colored_text = ColoredText::new();

    colored_text.push_str(
        "╔═══════════════════════════════════════════════╗\n",
        [0.0, 1.0, 1.0, 1.0],
    );
    colored_text.push_str(
        "║          SESSION STATISTICS REPORT            ║\n",
        [0.0, 1.0, 1.0, 1.0],
    );
    colored_text.push_str(
        "╚═══════════════════════════════════════════════╝\n\n",
        [0.0, 1.0, 1.0, 1.0],
    );

    if app.session_history.count() == 0 {
        colored_text.push_str("No sessions recorded yet.\n", [0.7, 0.7, 0.7, 1.0]);
        colored_text.push_str(
            "Start typing to track your progress!\n\n",
            [0.7, 0.7, 0.7, 1.0],
        );
    } else {
        let summary = app.session_history.get_summary();
        let recent_summary = app.session_history.get_recent_summary(5);
        let (improved, improvement) = app.session_history.analyze_improvement(5);

        colored_text.push_str(
            &format!("📊 ALL-TIME STATS ({} sessions)\n", summary.total_sessions),
            [1.0, 1.0, 0.0, 1.0],
        );
        colored_text.push_str(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            [0.5, 0.8, 1.0, 1.0],
        );
        colored_text.push_str(
            &format!("  Total Characters: {}\n", summary.total_chars),
            app.config.colors.text_default,
        );
        colored_text.push_str(
            &format!("  Total Time: {:.1} minutes\n", summary.total_time / 60.0),
            app.config.colors.text_default,
        );
        colored_text.push_str(
            &format!(
                "  Avg Speed: {:.0} CPM / {:.0} WPM\n",
                summary.avg_cpm, summary.avg_wpm
            ),
            [0.0, 1.0, 0.0, 1.0],
        );
        colored_text.push_str(
            &format!("  Avg Accuracy: {:.1}%\n", summary.avg_accuracy),
            [0.0, 1.0, 0.0, 1.0],
        );
        colored_text.push_str(
            &format!("  Total Errors: {}\n\n", summary.total_errors),
            app.config.colors.text_default,
        );

        colored_text.push_str("🏆 BEST PERFORMANCES\n", [1.0, 0.84, 0.0, 1.0]);
        colored_text.push_str(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            [0.5, 0.8, 1.0, 1.0],
        );
        colored_text.push_str(
            &format!(
                "  Best Speed: {:.0} CPM / {:.0} WPM\n",
                summary.best_cpm, summary.best_wpm
            ),
            [1.0, 0.5, 0.0, 1.0],
        );
        colored_text.push_str(
            &format!("  Best Accuracy: {:.1}%\n\n", summary.best_accuracy),
            [1.0, 0.5, 0.0, 1.0],
        );

        if recent_summary.total_sessions > 0 {
            colored_text.push_str(
                &format!(
                    "📈 RECENT PERFORMANCE (last {} sessions)\n",
                    recent_summary.total_sessions
                ),
                [0.5, 1.0, 0.5, 1.0],
            );
            colored_text.push_str(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
                [0.5, 0.8, 1.0, 1.0],
            );
            colored_text.push_str(
                &format!(
                    "  Avg Speed: {:.0} CPM / {:.0} WPM\n",
                    recent_summary.avg_cpm, recent_summary.avg_wpm
                ),
                [0.0, 1.0, 0.0, 1.0],
            );
            colored_text.push_str(
                &format!("  Avg Accuracy: {:.1}%\n", recent_summary.avg_accuracy),
                [0.0, 1.0, 0.0, 1.0],
            );

            if improved {
                colored_text.push_str(
                    &format!("  📊 Improvement: +{:.1}% 🎉\n", improvement),
                    [0.0, 1.0, 0.5, 1.0],
                );
            } else if improvement < 0.0 {
                colored_text.push_str(
                    &format!("  📊 Change: {:.1}%\n", improvement),
                    [1.0, 0.5, 0.0, 1.0],
                );
            }
            colored_text.push_str("\n", app.config.colors.text_default);
        }

        colored_text.push_str("📝 RECENT SESSIONS\n", [0.7, 0.7, 1.0, 1.0]);
        colored_text.push_str(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            [0.5, 0.8, 1.0, 1.0],
        );
        for (i, session) in app
            .session_history
            .get_recent_sessions(5)
            .iter()
            .enumerate()
        {
            colored_text.push_str(
                &format!(
                    "  {}. {:.0} CPM / {:.0} WPM | {:.1}% acc | {} chars\n",
                    i + 1,
                    session.chars_per_minute,
                    session.words_per_minute,
                    session.accuracy,
                    session.chars_typed
                ),
                app.config.colors.text_default,
            );
        }
    }

    colored_text.push_str(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        [0.5, 0.8, 1.0, 1.0],
    );
    colored_text.push_str(
        "Press ESC to return | Press Ctrl+T / Cmd+T to view stats\n",
        [0.7, 0.7, 0.7, 1.0],
    );

    colored_text
}

pub fn create_file_selection_screen(app: &CargoTapApp) -> ColoredText {
    let mut colored_text = ColoredText::new();

    colored_text.push_str(
        "╔═══════════════════════════════════════════════╗\n",
        [0.0, 1.0, 1.0, 1.0],
    );
    colored_text.push_str(
        "║              FILE SELECTION MODE              ║\n",
        [0.0, 1.0, 1.0, 1.0],
    );
    colored_text.push_str(
        "╚═══════════════════════════════════════════════╝\n\n",
        [0.0, 1.0, 1.0, 1.0],
    );

    colored_text.push_str("Enter file path to load:\n\n", [1.0, 1.0, 1.0, 1.0]);

    colored_text.push_str("📝 ", [1.0, 0.84, 0.0, 1.0]);
    colored_text.push_str(&app.file_input_buffer, [0.0, 1.0, 0.0, 1.0]);
    colored_text.push_str("█", [0.0, 1.0, 0.0, 1.0]);

    colored_text.push_str("\n\n", app.config.colors.text_default);
    colored_text.push_str(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        [0.5, 0.8, 1.0, 1.0],
    );

    colored_text.push_str("Current file: ", [0.7, 0.7, 0.7, 1.0]);
    colored_text.push_str(&app.current_file_path, [0.5, 1.0, 1.0, 1.0]);
    colored_text.push_str("\n\n", app.config.colors.text_default);

    colored_text.push_str("Instructions:\n", [1.0, 1.0, 0.0, 1.0]);
    colored_text.push_str(
        "  • Edit the file path below (pre-filled with current path)\n",
        [0.7, 0.7, 0.7, 1.0],
    );
    colored_text.push_str("  • Press ENTER to load the file\n", [0.7, 0.7, 0.7, 1.0]);
    colored_text.push_str("  • Press ESC to cancel\n", [0.7, 0.7, 0.7, 1.0]);
    colored_text.push_str(
        "  • Use BACKSPACE to delete characters\n\n",
        [0.7, 0.7, 0.7, 1.0],
    );

    colored_text.push_str("Examples:\n", [0.0, 1.0, 0.5, 1.0]);
    colored_text.push_str("  • src/main.rs\n", app.config.colors.text_default);
    colored_text.push_str(
        "  • /absolute/path/to/file.rs\n",
        app.config.colors.text_default,
    );
    colored_text.push_str(
        "  • ../other_project/code.rs\n",
        app.config.colors.text_default,
    );

    colored_text
}

fn apply_scroll_offset(text: &str, scroll_offset: usize) -> String {
    if scroll_offset == 0 {
        return text.to_string();
    }

    let mut lines_skipped = 0;
    let mut result = String::new();

    for ch in text.chars() {
        if lines_skipped >= scroll_offset {
            result.push(ch);
        } else if ch == '\n' {
            lines_skipped += 1;
        }
    }

    result
}

fn apply_scroll_offset_colored(colored_text: &ColoredText, scroll_offset: usize) -> ColoredText {
    if scroll_offset == 0 {
        return colored_text.clone();
    }

    let mut lines_skipped = 0;
    let mut result = ColoredText::new();

    for colored_char in &colored_text.chars {
        if lines_skipped >= scroll_offset {
            result.chars.push(crate::text::ColoredChar {
                ch: colored_char.ch,
                color: colored_char.color,
                background_color: colored_char.background_color,
            });
        } else if colored_char.ch == '\n' {
            lines_skipped += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_line_numbers_to_colored_text() {
        let mut colored_text = ColoredText::new();
        let test_code = "fn main() {\n    println!(\"Hello\");\n}";
        let line_num_color = [0.5, 0.5, 0.6, 1.0];
        let separator_color = [0.4, 0.4, 0.5, 1.0];

        add_line_numbers_to_colored_text(
            &mut colored_text,
            test_code,
            line_num_color,
            line_num_color,
            separator_color,
            0,
            None,
            0,
            [0.0, 1.0, 0.0, 1.0],
        );

        let result: String = colored_text.chars.iter().map(|c| c.ch).collect();

        // Should contain line numbers
        assert!(result.contains("  1 │"));
        assert!(result.contains("  2 │"));
        assert!(result.contains("  3 │"));

        // Should contain the original code
        assert!(result.contains("fn main() {"));
        assert!(result.contains("println!"));
    }

    #[test]
    fn test_add_line_numbers_with_scroll_offset() {
        let mut colored_text = ColoredText::new();
        let test_code = "line1\nline2\nline3";
        let line_num_color = [0.5, 0.5, 0.6, 1.0];
        let separator_color = [0.4, 0.4, 0.5, 1.0];

        add_line_numbers_to_colored_text(
            &mut colored_text,
            test_code,
            line_num_color,
            line_num_color,
            separator_color,
            10,
            None,
            0,
            [0.0, 1.0, 0.0, 1.0],
        );

        let result: String = colored_text.chars.iter().map(|c| c.ch).collect();

        // Line numbers should start from 11 (10 + 1)
        assert!(result.contains(" 11 │"));
        assert!(result.contains(" 12 │"));
        assert!(result.contains(" 13 │"));
    }

    #[test]
    fn test_caret_background_color() {
        use crate::code_state::CodeState;
        use crate::config::Config;

        let config = Config::load();
        let code = "hello world".to_string();
        let mut code_state = CodeState::new(code);

        // Type 5 characters so cursor is at position 5 (on 'o')
        for _ in 0..5 {
            code_state.type_character();
        }

        let full_code = code_state.get_full_code();
        let cursor_position = code_state.get_cursor_position();
        assert_eq!(cursor_position, 5);

        // Create colored text without syntax highlighting
        let mut colored_text = ColoredText::new();
        let caret_bg_color = [0.0, 1.0, 0.0, 1.0];

        let mut char_index = 0;
        for ch in full_code.chars() {
            if char_index == cursor_position {
                colored_text.push_with_background(ch, [1.0, 1.0, 1.0, 1.0], caret_bg_color);
            } else {
                colored_text.push(ch, [1.0, 1.0, 1.0, 1.0]);
            }
            char_index += ch.len_utf8();
        }

        // Verify the character at position 5 has a background color
        assert_eq!(colored_text.chars[5].ch, ' ');
        assert_eq!(colored_text.chars[5].background_color, Some(caret_bg_color));

        // Verify other characters don't have background
        assert_eq!(colored_text.chars[0].background_color, None);
        assert_eq!(colored_text.chars[4].background_color, None);
    }

    #[test]
    fn test_apply_scroll_offset() {
        let text = "line1\nline2\nline3\nline4\nline5";

        // No offset
        let result = apply_scroll_offset(text, 0);
        assert_eq!(result, text);

        // Skip 2 lines
        let result = apply_scroll_offset(text, 2);
        assert_eq!(result, "line3\nline4\nline5");

        // Skip all lines
        let result = apply_scroll_offset(text, 5);
        assert_eq!(result, "");
    }
}
