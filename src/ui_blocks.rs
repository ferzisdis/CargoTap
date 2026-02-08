use crate::app::CargoTapApp;
use crate::examples::colored_text_demo::ColoredTextDemo;
use crate::text::{ColoredChar, ColoredLine, ColoredText, TextSurface};

pub trait UiBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface);
}

pub struct HeaderBlock;

impl UiBlock for HeaderBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        let mut line = ColoredLine::new();
        line.push_str("🦀 CargoTap ", [1.0, 0.5, 0.0, 1.0]);
        line.push_str("Live Demo", app.config.colors.text_header);
        surface.write_line(&line);
    }
}

pub struct FileInfoBlock;

impl UiBlock for FileInfoBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        let mut line = ColoredLine::new();
        line.push_str(
            &format!("📄 File: {} ", app.current_file_path),
            [0.5, 1.0, 1.0, 1.0],
        );
        surface.write_line(&line);
    }
}

pub struct ProgressBlock;

impl UiBlock for ProgressBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        let progress_percent = app.code_state.get_progress() * 100.0;
        let cursor_pos = app.code_state.get_cursor_position();
        let total_len = app.code_state.get_total_length();
        let mut line = ColoredLine::new();
        line.push_str(
            &format!(
                "| Progress: {}/{} ({:.1}%)",
                cursor_pos, total_len, progress_percent
            ),
            [0.0, 1.0, 0.5, 1.0],
        );
        surface.write_line(&line);
    }
}

pub struct FpsBlock;

impl UiBlock for FpsBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        let mut line = ColoredLine::new();
        if app.config.debug.show_profiling_info {
            line.push_str(
                &format!(
                    "📊 FPS: {:.1} | Key: {:.3}ms | Text Update: {:.3}ms | UI Gen: {:.3}ms",
                    app.current_fps,
                    app.last_key_processing_time_ms,
                    app.text_update_time_ms,
                    app.ui_generation_time_ms
                ),
                [0.8, 0.8, 0.8, 1.0],
            );
        } else {
            line.push_str(
                &format!(
                    "📊 FPS: {:.1} | Key Processing: {:.3} ms",
                    app.current_fps, app.last_key_processing_time_ms
                ),
                [0.8, 0.8, 0.8, 1.0],
            );
        }
        surface.write_line(&line);
    }
}

pub struct SeparatorBlock {
    pub width: usize,
}

impl UiBlock for SeparatorBlock {
    fn render(&self, _app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        let mut line = ColoredLine::new();
        line.push_str(&"─".repeat(self.width), [0.5, 0.8, 1.0, 1.0]);
        surface.write_line(&line);
    }
}

pub struct SessionStateBlock;

impl UiBlock for SessionStateBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        if app.session_state.is_finished() {
            if let Some(stats) = app.session_state.last_stats() {
                let mut line = ColoredLine::new();
                line.push_str("⏰ SESSION COMPLETE! ", [1.0, 1.0, 0.0, 1.0]);
                surface.write_line(&line);

                let summary = format!(
                    "Time: {:.1}s | Chars: {} | Speed: {:.0} CPM / {:.0} WPM | Accuracy: {:.1}%",
                    stats.time_elapsed_secs,
                    stats.chars_typed,
                    stats.chars_per_minute,
                    stats.words_per_minute,
                    stats.accuracy
                );
                let mut line = ColoredLine::new();
                line.push_str(&summary, [0.0, 1.0, 0.0, 1.0]);
                surface.write_line(&line);

                let mut line = ColoredLine::new();
                line.push_str("Press SPACE to start new session", [0.0, 1.0, 1.0, 1.0]);
                surface.write_line(&line);

                let mut line = ColoredLine::new();
                line.push_str(&"─".repeat(30), [0.5, 0.8, 1.0, 1.0]);
                surface.write_line(&line);
            }
        } else if app.session_state.is_active() {
            let time_str = format!("⏱️  Time: {} ", app.session_state.format_time_remaining());
            let mut line = ColoredLine::new();
            line.push_str(&time_str, [1.0, 1.0, 0.0, 1.0]);

            let current_pos = app.code_state.get_cursor_position();
            let stats = app.session_state.current_stats(current_pos);
            if stats.time_elapsed_secs > 0.0 {
                let speed_str = format!("| {:.0} CPM ", stats.chars_per_minute);
                line.push_str(&speed_str, [0.0, 1.0, 0.0, 1.0]);
            }
            surface.write_line(&line);

            let mut line = ColoredLine::new();
            line.push_str(&"─".repeat(30), [0.5, 0.8, 1.0, 1.0]);
            surface.write_line(&line);
        } else {
            let mut line = ColoredLine::new();
            line.push_str("Start typing to begin session...", [0.7, 0.7, 0.7, 1.0]);
            surface.write_line(&line);

            let mut line = ColoredLine::new();
            line.push_str(&"─".repeat(30), [0.5, 0.8, 1.0, 1.0]);
            surface.write_line(&line);
        }
    }
}

pub struct CodeDisplayBlock;

impl UiBlock for CodeDisplayBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        let cursor_position = app.code_state.get_cursor_position();
        let full_code_colored = app.code_state.get_full_code_colored();
        let display_colored = apply_scroll_offset_colored(&full_code_colored, app.scroll_offset);

        let cursor_position_in_display = if app.scroll_offset == 0 {
            cursor_position
        } else {
            let mut bytes_skipped = 0;
            for (line_idx, line) in full_code_colored.lines.iter().enumerate() {
                if line_idx >= app.scroll_offset {
                    break;
                }
                for colored_char in &line.chars {
                    bytes_skipped += colored_char.ch.len_utf8();
                }
                bytes_skipped += 1;
            }
            cursor_position.saturating_sub(bytes_skipped)
        };

        render_code_with_line_numbers(app, surface, &display_colored, cursor_position_in_display);
    }
}

fn apply_scroll_offset_colored(colored_text: &ColoredText, scroll_offset: usize) -> ColoredText {
    if scroll_offset == 0 {
        return colored_text.clone();
    }

    let mut result = ColoredText::new();
    result.lines.clear();

    for (line_idx, line) in colored_text.lines.iter().enumerate() {
        if line_idx >= scroll_offset {
            result.lines.push(line.clone());
        }
    }

    if result.lines.is_empty() {
        result.lines.push(crate::text::ColoredLine::new());
    }

    result
}

fn render_code_with_line_numbers(
    app: &mut CargoTapApp,
    surface: &mut dyn TextSurface,
    display_colored: &ColoredText,
    cursor_position_in_display: usize,
) {
    let line_number_color = [0.5, 0.5, 0.6, 1.0];
    let current_line_color = [1.0, 0.85, 0.2, 1.0];
    let separator_color = [0.4, 0.4, 0.5, 1.0];
    let caret_bg_color = [0.0, 1.0, 0.0, 0.5];

    let current_line = Some(app.code_state.get_cursor_line());
    let mut display_with_caret = display_colored.clone();

    if let Some(char_ref) = display_with_caret.get_char_mut(cursor_position_in_display) {
        char_ref.background_color = Some(caret_bg_color);
    } else {
        display_with_caret.push_colored_char(crate::text::ColoredChar {
            ch: ' ',
            color: app.config.colors.text_default,
            background_color: Some(caret_bg_color),
        });
    }

    let line_count = display_with_caret.lines.len();
    let start_line = app.scroll_offset + 1;
    let num_digits = (line_count + app.scroll_offset).to_string().len().max(3);
    let mut current_line_num = start_line;

    for (line_idx, code_line) in display_with_caret.lines.iter().enumerate() {
        let mut line = ColoredLine::new();

        let line_num_str = format!("{:>width$}", current_line_num, width = num_digits);
        let is_current = current_line.map_or(false, |cl| cl == current_line_num);
        let num_color = if is_current {
            current_line_color
        } else {
            line_number_color
        };

        for ch in line_num_str.chars() {
            line.push(ch, num_color);
        }
        line.push(' ', separator_color);
        line.push('│', separator_color);
        line.push(' ', separator_color);

        for colored_char in &code_line.chars {
            if let Some(bg_color) = colored_char.background_color {
                line.chars.push(ColoredChar {
                    ch: colored_char.ch,
                    color: colored_char.color,
                    background_color: Some(bg_color),
                });
            } else {
                line.push(colored_char.ch, colored_char.color);
            }
        }

        surface.write_line(&line);

        if line_idx + 1 < display_with_caret.lines.len() {
            current_line_num += 1;
        }
    }
}

pub struct RainbowEffectsBlock;

impl UiBlock for RainbowEffectsBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        surface.write_line(&ColoredLine::new());

        let mut line = ColoredLine::new();
        line.push_str("✨ Rainbow: ", app.config.colors.text_default);
        let rainbow = ColoredTextDemo::create_rainbow_text("Per-character colors work!");

        for rainbow_line in rainbow.lines {
            for colored_char in rainbow_line.chars {
                line.push(colored_char.ch, colored_char.color);
            }
        }
        surface.write_line(&line);
    }
}

pub struct FooterBlock;

impl UiBlock for FooterBlock {
    fn render(&self, _app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        surface.write_line(&ColoredLine::new());

        let mut line = ColoredLine::new();
        line.push_str(
            "Press Cmd+P to change file | Press Ctrl+T / Cmd+T to view statistics",
            [0.5, 0.5, 0.5, 1.0],
        );
        surface.write_line(&line);
    }
}
