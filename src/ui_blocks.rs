use crate::app::CargoTapApp;
use crate::examples::colored_text_demo::ColoredTextDemo;
use crate::text::{ColoredChar, ColoredLine, ColoredText, TextSurface, WriteResult};

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
        surface.write_break();
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
        surface.write_break();
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
        surface.write_break();
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
        surface.write_break();
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
        surface.write_break();
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
                surface.write_break();

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
                surface.write_break();

                let mut line = ColoredLine::new();
                line.push_str("Press SPACE to start new session", [0.0, 1.0, 1.0, 1.0]);
                surface.write_line(&line);
                surface.write_break();

                let mut line = ColoredLine::new();
                line.push_str(&"─".repeat(30), [0.5, 0.8, 1.0, 1.0]);
                surface.write_line(&line);
                surface.write_break();
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
            surface.write_break();

            let mut line = ColoredLine::new();
            line.push_str(&"─".repeat(30), [0.5, 0.8, 1.0, 1.0]);
            surface.write_line(&line);
            surface.write_break();
        } else {
            let mut line = ColoredLine::new();
            line.push_str("Start typing to begin session...", [0.7, 0.7, 0.7, 1.0]);
            surface.write_line(&line);
            surface.write_break();

            let mut line = ColoredLine::new();
            line.push_str(&"─".repeat(30), [0.5, 0.8, 1.0, 1.0]);
            surface.write_line(&line);
            surface.write_break();
        }
    }
}

pub struct CodeDisplayBlock;

impl UiBlock for CodeDisplayBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        let line_number_color = [0.5, 0.5, 0.6, 1.0];
        let current_line_color = [1.0, 0.85, 0.2, 1.0];
        let separator_color = [0.4, 0.4, 0.5, 1.0];
        let caret_bg_color = [0.0, 1.0, 0.0, 0.5];

        let mut cursor_position = app.code_state.get_cursor_position() as i32;
        let mut scroll_offset = app.scroll_offset;
        let full_code_colored = app.code_state.get_full_code_colored();

        let num_digits = (full_code_colored.lines.len() + app.scroll_offset)
            .to_string()
            .len()
            .max(3);

        let mut current_line = ColoredLine::new();
        for (num, line) in full_code_colored.lines.iter().enumerate() {
            let line_len = line.chars.iter().map(|ch| ch.ch.len_utf8()).sum::<usize>();
            let break_len = '\n'.len_utf8();
            if scroll_offset > 0 {
                cursor_position -= (line_len + break_len) as i32;
                scroll_offset -= 1;
                continue;
            }

            let (code_line, is_current) =
                if cursor_position >= 0 && line_len + break_len > cursor_position as usize {
                    current_line = line.clone();
                    if line_len == cursor_position as usize {
                        current_line.chars.push(crate::text::ColoredChar {
                            ch: '↩',
                            color: app.config.colors.text_default,
                            background_color: None,
                        });
                    }

                    let mut index = cursor_position as usize;
                    for i in 0..current_line.chars.len() {
                        if index == 0 {
                            if let Some(ch_mut) = current_line.chars.get_mut(i) {
                                ch_mut.background_color = Some(caret_bg_color);
                            }
                            break;
                        }
                        if line.chars[i].ch.len_utf8() > index {
                            break;
                        }
                        index -= line.chars[i].ch.len_utf8();
                    }
                    (&current_line, true)
                } else {
                    (line, false)
                };

            let mut num_line = ColoredLine::new();
            let line_num_str = format!("{:>width$}", num + 1, width = num_digits);
            let num_color = if is_current {
                current_line_color
            } else {
                line_number_color
            };

            for ch in line_num_str.chars() {
                num_line.push(ch, num_color);
            }
            num_line.push(' ', separator_color);
            num_line.push('│', separator_color);
            num_line.push(' ', separator_color);

            surface.write_line(&num_line);
            surface.write_line_wordwrap(code_line);

            if matches!(surface.write_break(), WriteResult::Overflow { writed: _ }) {
                return;
            }
            cursor_position -= (line_len + break_len) as i32;
        }
    }
}

pub struct RainbowEffectsBlock;

impl UiBlock for RainbowEffectsBlock {
    fn render(&self, app: &mut CargoTapApp, surface: &mut dyn TextSurface) {
        surface.write_break();

        let mut line = ColoredLine::new();
        line.push_str("✨ Rainbow: ", app.config.colors.text_default);
        let rainbow = ColoredTextDemo::create_rainbow_text("Per-character colors work!");

        for rainbow_line in rainbow.lines {
            for colored_char in rainbow_line.chars {
                line.push(colored_char.ch, colored_char.color);
            }
        }
        surface.write_line(&line);
        surface.write_break();
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
        surface.write_break();
    }
}
