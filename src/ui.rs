use crate::app::CargoTapApp;
use crate::text::{TextSurface, TextSurfaceWriter};
use crate::ui_blocks::{
    CodeDisplayBlock, FileInfoBlock, FooterBlock, FpsBlock, HeaderBlock, ProgressBlock,
    RainbowEffectsBlock, SeparatorBlock, SessionStateBlock, UiBlock,
};
use std::fs;
use std::path::Path;

pub fn create_colored_text(app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
    let mut writer = TextSurfaceWriter::new(surface);

    if app.file_selection_mode {
        create_file_selection_screen(app, &mut writer);
        return;
    }

    if app.show_statistics {
        create_statistics_screen(app, &mut writer);
        return;
    }

    HeaderBlock.render(app, &mut writer);
    FileInfoBlock.render(app, &mut writer);
    ProgressBlock.render(app, &mut writer);
    FpsBlock.render(app, &mut writer);
    SeparatorBlock { width: 50 }.render(app, &mut writer);
    SessionStateBlock.render(app, &mut writer);
    CodeDisplayBlock.render(app, &mut writer);

    if app.config.text.rainbow_effects {
        RainbowEffectsBlock.render(app, &mut writer);
    }

    FooterBlock.render(app, &mut writer);
}

fn create_statistics_screen(app: &mut CargoTapApp, writer: &mut TextSurfaceWriter) {
    writer.push_str(
        "╔═══════════════════════════════════════════════╗\n",
        [0.0, 1.0, 1.0, 1.0],
    );
    writer.push_str(
        "║          SESSION STATISTICS REPORT            ║\n",
        [0.0, 1.0, 1.0, 1.0],
    );
    writer.push_str(
        "╚═══════════════════════════════════════════════╝\n\n",
        [0.0, 1.0, 1.0, 1.0],
    );

    if app.session_history.count() == 0 {
        writer.push_str("No sessions recorded yet.\n", [0.7, 0.7, 0.7, 1.0]);
        writer.push_str(
            "Start typing to track your progress!\n\n",
            [0.7, 0.7, 0.7, 1.0],
        );
    } else {
        let summary = app.session_history.get_summary();
        let recent_summary = app.session_history.get_recent_summary(5);
        let (improved, improvement) = app.session_history.analyze_improvement(5);

        writer.push_str(
            &format!("📊 ALL-TIME STATS ({} sessions)\n", summary.total_sessions),
            [1.0, 1.0, 0.0, 1.0],
        );
        writer.push_str(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            [0.5, 0.8, 1.0, 1.0],
        );
        writer.push_str(
            &format!("  Total Characters: {}\n", summary.total_chars),
            app.config.colors.text_default,
        );
        writer.push_str(
            &format!("  Total Time: {:.1} minutes\n", summary.total_time / 60.0),
            app.config.colors.text_default,
        );
        writer.push_str(
            &format!(
                "  Avg Speed: {:.0} CPM / {:.0} WPM\n",
                summary.avg_cpm, summary.avg_wpm
            ),
            [0.0, 1.0, 0.0, 1.0],
        );
        writer.push_str(
            &format!("  Avg Accuracy: {:.1}%\n", summary.avg_accuracy),
            [0.0, 1.0, 0.0, 1.0],
        );
        writer.push_str(
            &format!("  Total Errors: {}\n\n", summary.total_errors),
            app.config.colors.text_default,
        );

        writer.push_str("🏆 BEST PERFORMANCES\n", [1.0, 0.84, 0.0, 1.0]);
        writer.push_str(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            [0.5, 0.8, 1.0, 1.0],
        );
        writer.push_str(
            &format!(
                "  Best Speed: {:.0} CPM / {:.0} WPM\n",
                summary.best_cpm, summary.best_wpm
            ),
            [1.0, 0.5, 0.0, 1.0],
        );
        writer.push_str(
            &format!("  Best Accuracy: {:.1}%\n\n", summary.best_accuracy),
            [1.0, 0.5, 0.0, 1.0],
        );

        if recent_summary.total_sessions > 0 {
            writer.push_str(
                &format!(
                    "📈 RECENT PERFORMANCE (last {} sessions)\n",
                    recent_summary.total_sessions
                ),
                [0.5, 1.0, 0.5, 1.0],
            );
            writer.push_str(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
                [0.5, 0.8, 1.0, 1.0],
            );
            writer.push_str(
                &format!(
                    "  Avg Speed: {:.0} CPM / {:.0} WPM\n",
                    recent_summary.avg_cpm, recent_summary.avg_wpm
                ),
                [0.0, 1.0, 0.0, 1.0],
            );
            writer.push_str(
                &format!("  Avg Accuracy: {:.1}%\n", recent_summary.avg_accuracy),
                [0.0, 1.0, 0.0, 1.0],
            );

            if improved {
                writer.push_str(
                    &format!("  📊 Improvement: +{:.1}% 🎉\n", improvement),
                    [0.0, 1.0, 0.5, 1.0],
                );
            } else if improvement < 0.0 {
                writer.push_str(
                    &format!("  📊 Change: {:.1}%\n", improvement),
                    [1.0, 0.5, 0.0, 1.0],
                );
            }
            writer.push_str("\n", app.config.colors.text_default);
        }

        writer.push_str("📝 RECENT SESSIONS\n", [0.7, 0.7, 1.0, 1.0]);
        writer.push_str(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            [0.5, 0.8, 1.0, 1.0],
        );
        for (i, session) in app
            .session_history
            .get_recent_sessions(5)
            .iter()
            .enumerate()
        {
            writer.push_str(
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

    writer.push_str(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        [0.5, 0.8, 1.0, 1.0],
    );
    writer.push_str(
        "Press ESC to return | Press Ctrl+T / Cmd+T to view stats\n",
        [0.7, 0.7, 0.7, 1.0],
    );
}

fn create_file_selection_screen(app: &mut CargoTapApp, writer: &mut TextSurfaceWriter) {
    writer.push_str(
        "╔═══════════════════════════════════════════════╗\n",
        [0.0, 1.0, 1.0, 1.0],
    );
    writer.push_str(
        "║              FILE SELECTION MODE              ║\n",
        [0.0, 1.0, 1.0, 1.0],
    );
    writer.push_str(
        "╚═══════════════════════════════════════════════╝\n\n",
        [0.0, 1.0, 1.0, 1.0],
    );

    writer.push_str("Enter file path to load:\n\n", [1.0, 1.0, 1.0, 1.0]);

    writer.push_str("📝 ", [1.0, 0.84, 0.0, 1.0]);
    writer.push_str(&app.file_input_buffer, [0.0, 1.0, 0.0, 1.0]);
    writer.push_str("█", [0.0, 1.0, 0.0, 1.0]);

    writer.push_str("\n\n", app.config.colors.text_default);
    writer.push_str(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        [0.5, 0.8, 1.0, 1.0],
    );

    let dir_path = get_directory_from_path(&app.file_input_buffer);

    if let Ok(entries) = fs::read_dir(&dir_path) {
        writer.push_str(
            &format!("Contents of directory: {}\n", dir_path),
            [0.7, 0.7, 0.7, 1.0],
        );
        writer.push_str(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            [0.5, 0.8, 1.0, 1.0],
        );

        let mut dirs: Vec<_> = Vec::new();
        let mut files: Vec<_> = Vec::new();

        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                dirs.push(entry);
            } else if entry.path().is_file() {
                files.push(entry);
            }
        }

        dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for entry in dirs.iter().take(10) {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            writer.push_str("  📁 ", [0.5, 0.7, 1.0, 1.0]);
            writer.push_str(&file_name_str, [0.5, 0.7, 1.0, 1.0]);
            writer.push_str("/", [0.5, 0.7, 1.0, 1.0]);

            let padding = 40_usize.saturating_sub(file_name_str.len() + 1);
            writer.push_str(&" ".repeat(padding), [0.7, 0.7, 0.7, 1.0]);
            writer.push_str("<DIR>", [0.5, 0.7, 1.0, 1.0]);
            writer.push_str("\n", [0.7, 0.7, 0.7, 1.0]);
        }

        if dirs.len() > 10 {
            writer.push_str(
                &format!("  ... and {} more directories\n", dirs.len() - 10),
                [0.6, 0.6, 0.6, 1.0],
            );
        }

        for entry in files.iter().take(20) {
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            let size_str = format_file_size(file_size);

            let full_path = path.to_string_lossy().to_string();
            let has_progress = app.progress_storage.get_progress(&full_path).is_some();

            if has_progress {
                writer.push_str("  ★ ", [1.0, 0.84, 0.0, 1.0]);
                writer.push_str(&file_name_str, [1.0, 1.0, 0.0, 1.0]);
            } else {
                writer.push_str("    ", [0.7, 0.7, 0.7, 1.0]);
                writer.push_str(&file_name_str, [0.9, 0.9, 0.9, 1.0]);
            }

            let padding = 40_usize.saturating_sub(file_name_str.len());
            writer.push_str(&" ".repeat(padding), [0.7, 0.7, 0.7, 1.0]);
            writer.push_str(&size_str, [0.5, 0.8, 1.0, 1.0]);
            writer.push_str("\n", [0.7, 0.7, 0.7, 1.0]);
        }

        if files.len() > 20 {
            writer.push_str(
                &format!("  ... and {} more files\n", files.len() - 20),
                [0.6, 0.6, 0.6, 1.0],
            );
        }

        writer.push_str("\n", app.config.colors.text_default);
    }

    writer.push_str(
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        [0.5, 0.8, 1.0, 1.0],
    );

    writer.push_str("Current file: ", [0.7, 0.7, 0.7, 1.0]);
    writer.push_str(&app.current_file_path, [0.5, 1.0, 1.0, 1.0]);
    writer.push_str("\n\n", app.config.colors.text_default);

    writer.push_str("Instructions:\n", [1.0, 1.0, 0.0, 1.0]);
    writer.push_str(
        "  • Edit the file path below (pre-filled with current path)\n",
        [0.7, 0.7, 0.7, 1.0],
    );
    writer.push_str("  • Press ENTER to load the file\n", [0.7, 0.7, 0.7, 1.0]);
    writer.push_str("  • Press ESC to cancel\n", [0.7, 0.7, 0.7, 1.0]);
    writer.push_str(
        "  • Use BACKSPACE to delete characters\n",
        [0.7, 0.7, 0.7, 1.0],
    );
    writer.push_str(
        "  • Files with ★ have saved progress\n\n",
        [0.7, 0.7, 0.7, 1.0],
    );
}

fn get_directory_from_path(path: &str) -> String {
    let path_obj = Path::new(path);

    if path_obj.is_dir() {
        return path.to_string();
    }

    if let Some(parent) = path_obj.parent() {
        if parent.as_os_str().is_empty() {
            return ".".to_string();
        }
        return parent.to_string_lossy().to_string();
    }

    ".".to_string()
}

fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::ColoredText;

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
