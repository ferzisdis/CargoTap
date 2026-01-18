use crate::app::CargoTapApp;
use crate::text::ColoredText;
use crate::ui_blocks::{
    CodeDisplayBlock, FileInfoBlock, FooterBlock, FpsBlock, HeaderBlock, ProgressBlock,
    RainbowEffectsBlock, SeparatorBlock, SessionStateBlock, UiBlock,
};

pub fn create_colored_text(app: &mut CargoTapApp) -> ColoredText {
    if app.file_selection_mode {
        return create_file_selection_screen(app);
    }

    if app.show_statistics {
        return create_statistics_screen(app);
    }

    let mut colored_text = ColoredText::new();

    HeaderBlock.render(app, &mut colored_text);
    FileInfoBlock.render(app, &mut colored_text);
    ProgressBlock.render(app, &mut colored_text);
    FpsBlock.render(app, &mut colored_text);
    SeparatorBlock { width: 50 }.render(app, &mut colored_text);
    SessionStateBlock.render(app, &mut colored_text);
    CodeDisplayBlock.render(app, &mut colored_text);

    if app.config.text.rainbow_effects {
        RainbowEffectsBlock.render(app, &mut colored_text);
    }

    FooterBlock.render(app, &mut colored_text);

    colored_text
}

fn create_statistics_screen(app: &mut CargoTapApp) -> ColoredText {
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

fn create_file_selection_screen(app: &mut CargoTapApp) -> ColoredText {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caret_background_color() {
        use crate::code_state::CodeState;
        use crate::config::Config;

        let config = Config::load();
        let code = "hello world".to_string();
        let mut code_state = CodeState::new(code);
        let _config = config;

        for _ in 0..5 {
            code_state.type_character();
        }

        let full_code = code_state.get_full_code();
        let cursor_position = code_state.get_cursor_position();
        assert_eq!(cursor_position, 5);

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

        if let Some(char_at_5) = colored_text.get_char(5) {
            assert_eq!(char_at_5.ch, ' ');
            assert_eq!(char_at_5.background_color, Some(caret_bg_color));
        }

        if let Some(char_at_0) = colored_text.get_char(0) {
            assert_eq!(char_at_0.background_color, None);
        }
        if let Some(char_at_4) = colored_text.get_char(4) {
            assert_eq!(char_at_4.background_color, None);
        }
    }
}
