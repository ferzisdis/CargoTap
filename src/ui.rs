use crate::app::CargoTapApp;
use crate::examples::colored_text_demo::ColoredTextDemo;
use crate::text::ColoredText;

fn find_current_line_number(text: &str) -> Option<usize> {
    let mut line_num = 1;
    for ch in text.chars() {
        if ch == '|' {
            return Some(line_num);
        }
        if ch == '\n' {
            line_num += 1;
        }
    }
    None
}

fn add_line_numbers_to_colored_text(
    colored_text: &mut ColoredText,
    text: &str,
    line_number_color: [f32; 4],
    current_line_color: [f32; 4],
    separator_color: [f32; 4],
    scroll_offset: usize,
    current_line: Option<usize>,
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

        for ch in line.chars() {
            colored_text.push(ch, [1.0, 1.0, 1.0, 1.0]);
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

    let full_code = app.code_state.get_full_code();
    let display_text = apply_scroll_offset(&full_code, app.scroll_offset);

    if app.config.text.show_line_numbers {
        let line_number_color = [0.5, 0.5, 0.6, 1.0];
        let current_line_color = [1.0, 0.85, 0.2, 1.0]; // Bright yellow/gold
        let separator_color = [0.4, 0.4, 0.5, 1.0];

        let current_line =
            find_current_line_number(&display_text).map(|line| line + app.scroll_offset);

        if app.config.text.syntax_highlighting {
            let syntax_highlighted = ColoredTextDemo::create_syntax_highlighted_rust(&display_text);
            let lines: Vec<&str> = display_text.split('\n').collect();
            let total_lines = lines.len();
            let start_line = app.scroll_offset + 1;
            let num_digits = (total_lines + app.scroll_offset).to_string().len().max(3);

            let mut char_index = 0;
            for (line_idx, _) in lines.iter().enumerate() {
                let line_num = start_line + line_idx;
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

                loop {
                    if char_index >= syntax_highlighted.chars.len() {
                        break;
                    }
                    let colored_char = &syntax_highlighted.chars[char_index];
                    colored_text.push(colored_char.ch, colored_char.color);
                    char_index += 1;

                    if colored_char.ch == '\n' {
                        break;
                    }
                }
            }
        } else {
            add_line_numbers_to_colored_text(
                &mut colored_text,
                &display_text,
                line_number_color,
                current_line_color,
                separator_color,
                app.scroll_offset,
                current_line,
            );
        }
    } else {
        if app.config.text.syntax_highlighting {
            let syntax_highlighted = ColoredTextDemo::create_syntax_highlighted_rust(&display_text);
            colored_text.chars.extend(syntax_highlighted.chars);
        } else {
            colored_text.push_str(&display_text, app.config.colors.text_default);
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
        );

        let result: String = colored_text.chars.iter().map(|c| c.ch).collect();

        // Line numbers should start from 11 (10 + 1)
        assert!(result.contains(" 11 │"));
        assert!(result.contains(" 12 │"));
        assert!(result.contains(" 13 │"));
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

    #[test]
    fn test_find_current_line_number() {
        let text = "line1\nline2\n|line3\nline4";
        assert_eq!(find_current_line_number(text), Some(3));

        let text2 = "|first line\nsecond line";
        assert_eq!(find_current_line_number(text2), Some(1));

        let text3 = "no cursor here";
        assert_eq!(find_current_line_number(text3), None);
    }
}
