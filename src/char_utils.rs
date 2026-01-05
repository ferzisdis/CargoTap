/// Utilities for handling characters that may be difficult or impossible to type
/// on a standard US keyboard

/// Determines if a character can be easily typed on a US keyboard
///
/// Returns `false` for:
/// - Emoji and other pictographic symbols
/// - Arabic, Hebrew, Chinese, Japanese, Korean scripts
/// - Cyrillic (Russian, etc.) unless specifically allowed
/// - Mathematical symbols beyond basic ASCII
/// - Box-drawing characters
/// - Most Unicode symbols beyond ASCII range
///
/// Returns `true` for:
/// - ASCII printable characters (space through ~)
/// - Common whitespace (space, tab, newline, carriage return)
/// - Extended Latin characters that might be on some keyboards
pub fn is_typeable_on_us_keyboard(ch: char) -> bool {
    match ch {
        // Basic ASCII printable characters (space through tilde)
        ' '..='~' => true,

        // Common whitespace characters
        '\t' | '\n' | '\r' => true,

        // Extended Latin characters (accented characters that may appear on international keyboards)
        // Being conservative here - only allowing common Latin extensions
        '\u{00A0}'..='\u{00FF}' => {
            // Latin-1 Supplement (includes accented characters like é, ñ, etc.)
            // These might be accessible via Alt codes or option key combinations
            true
        }

        // Everything else is considered untypeable on a standard US keyboard
        _ => false,
    }
}

/// Categorizes why a character is untypeable
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnTypeableReason {
    Emoji,
    Arabic,
    Hebrew,
    CJK, // Chinese, Japanese, Korean
    Cyrillic,
    MathSymbol,
    BoxDrawing,
    OtherSymbol,
}

/// Provides detailed information about why a character cannot be typed
pub fn get_untypeable_reason(ch: char) -> Option<UnTypeableReason> {
    if is_typeable_on_us_keyboard(ch) {
        return None;
    }

    let code_point = ch as u32;

    Some(match code_point {
        // Emoji ranges
        0x1F300..=0x1F9FF | // Miscellaneous Symbols and Pictographs, Emoticons, etc.
        0x1FA00..=0x1FAFF | // Extended pictographic
        0x2600..=0x26FF |   // Miscellaneous Symbols
        0x2700..=0x27BF     // Dingbats
        => UnTypeableReason::Emoji,

        // Arabic
        0x0600..=0x06FF | // Arabic
        0x0750..=0x077F | // Arabic Supplement
        0x08A0..=0x08FF   // Arabic Extended-A
        => UnTypeableReason::Arabic,

        // Hebrew
        0x0590..=0x05FF => UnTypeableReason::Hebrew,

        // CJK (Chinese, Japanese, Korean)
        0x4E00..=0x9FFF |   // CJK Unified Ideographs
        0x3040..=0x309F |   // Hiragana
        0x30A0..=0x30FF |   // Katakana
        0xAC00..=0xD7AF     // Hangul Syllables
        => UnTypeableReason::CJK,

        // Cyrillic
        0x0400..=0x04FF | // Cyrillic
        0x0500..=0x052F   // Cyrillic Supplement
        => UnTypeableReason::Cyrillic,

        // Mathematical symbols
        0x2200..=0x22FF => UnTypeableReason::MathSymbol,

        // Box drawing
        0x2500..=0x257F => UnTypeableReason::BoxDrawing,

        // Everything else
        _ => UnTypeableReason::OtherSymbol,
    })
}

/// Returns a human-readable description of why a character is untypeable
pub fn get_untypeable_description(ch: char) -> Option<String> {
    get_untypeable_reason(ch).map(|reason| {
        let category = match reason {
            UnTypeableReason::Emoji => "emoji",
            UnTypeableReason::Arabic => "Arabic script",
            UnTypeableReason::Hebrew => "Hebrew script",
            UnTypeableReason::CJK => "CJK character (Chinese/Japanese/Korean)",
            UnTypeableReason::Cyrillic => "Cyrillic script",
            UnTypeableReason::MathSymbol => "mathematical symbol",
            UnTypeableReason::BoxDrawing => "box-drawing character",
            UnTypeableReason::OtherSymbol => "special Unicode symbol",
        };

        format!("'{}' ({}) - not typeable on US keyboard", ch, category)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ascii_is_typeable() {
        assert!(is_typeable_on_us_keyboard('a'));
        assert!(is_typeable_on_us_keyboard('Z'));
        assert!(is_typeable_on_us_keyboard('0'));
        assert!(is_typeable_on_us_keyboard('9'));
        assert!(is_typeable_on_us_keyboard(' '));
        assert!(is_typeable_on_us_keyboard('!'));
        assert!(is_typeable_on_us_keyboard('{'));
        assert!(is_typeable_on_us_keyboard('}'));
        assert!(is_typeable_on_us_keyboard('\n'));
        assert!(is_typeable_on_us_keyboard('\t'));
    }

    #[test]
    fn test_emoji_not_typeable() {
        assert!(!is_typeable_on_us_keyboard('🦀'));
        assert!(!is_typeable_on_us_keyboard('😀'));
        assert!(!is_typeable_on_us_keyboard('🎉'));
        assert_eq!(get_untypeable_reason('🦀'), Some(UnTypeableReason::Emoji));
    }

    #[test]
    fn test_arabic_not_typeable() {
        assert!(!is_typeable_on_us_keyboard('ا')); // Arabic letter Alef
        assert!(!is_typeable_on_us_keyboard('ب')); // Arabic letter Beh
        assert_eq!(get_untypeable_reason('ا'), Some(UnTypeableReason::Arabic));
    }

    #[test]
    fn test_hebrew_not_typeable() {
        assert!(!is_typeable_on_us_keyboard('א')); // Hebrew letter Alef
        assert!(!is_typeable_on_us_keyboard('ב')); // Hebrew letter Bet
        assert_eq!(get_untypeable_reason('א'), Some(UnTypeableReason::Hebrew));
    }

    #[test]
    fn test_cyrillic_not_typeable() {
        assert!(!is_typeable_on_us_keyboard('Ж')); // Cyrillic letter Zhe
        assert!(!is_typeable_on_us_keyboard('я')); // Cyrillic letter Ya
        assert_eq!(get_untypeable_reason('Ж'), Some(UnTypeableReason::Cyrillic));
    }

    #[test]
    fn test_cjk_not_typeable() {
        assert!(!is_typeable_on_us_keyboard('中')); // Chinese
        assert!(!is_typeable_on_us_keyboard('日')); // Japanese Kanji
        assert!(!is_typeable_on_us_keyboard('あ')); // Japanese Hiragana
        assert!(!is_typeable_on_us_keyboard('한')); // Korean Hangul
        assert_eq!(get_untypeable_reason('中'), Some(UnTypeableReason::CJK));
    }

    #[test]
    fn test_box_drawing_not_typeable() {
        assert!(!is_typeable_on_us_keyboard('─'));
        assert!(!is_typeable_on_us_keyboard('═'));
        assert!(!is_typeable_on_us_keyboard('│'));
        assert_eq!(
            get_untypeable_reason('─'),
            Some(UnTypeableReason::BoxDrawing)
        );
    }

    #[test]
    fn test_math_symbols_not_typeable() {
        assert!(!is_typeable_on_us_keyboard('∀')); // For all
        assert!(!is_typeable_on_us_keyboard('∃')); // There exists
        assert!(!is_typeable_on_us_keyboard('∈')); // Element of
        assert_eq!(
            get_untypeable_reason('∀'),
            Some(UnTypeableReason::MathSymbol)
        );
    }

    #[test]
    fn test_extended_latin_is_typeable() {
        // Being lenient with accented characters
        assert!(is_typeable_on_us_keyboard('é'));
        assert!(is_typeable_on_us_keyboard('ñ'));
        assert!(is_typeable_on_us_keyboard('ü'));
    }

    #[test]
    fn test_description() {
        let desc = get_untypeable_description('🦀');
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("emoji"));

        let desc = get_untypeable_description('ا');
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("Arabic"));

        let desc = get_untypeable_description('a');
        assert!(desc.is_none());
    }
}
