//!   =============================================================================
//! Text Cleaner Module
//!   =============================================================================
//! Cleans text for speech synthesis by removing:
//! - Emojis
//! - Code blocks
//! - Markdown formatting
//! - URLs
//! - Extra whitespace
//!   =============================================================================

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Emoji pattern covering most common emoji ranges
    static ref EMOJI_PATTERN: Regex = Regex::new(
        r"[\x{1F600}-\x{1F64F}\x{1F300}-\x{1F5FF}\x{1F680}-\x{1F6FF}\x{1F1E0}-\x{1F1FF}\x{2702}-\x{27B0}\x{24C2}-\x{1F251}\x{1F900}-\x{1F9FF}\x{1FA00}-\x{1FA6F}\x{1FA70}-\x{1FAFF}\x{2600}-\x{26FF}\x{2300}-\x{23FF}]+"
    ).expect("Invalid emoji regex");

    /// Code block pattern (```...```)
    static ref CODE_BLOCK_PATTERN: Regex = Regex::new(r"```[\s\S]*?```")
        .expect("Invalid code block regex");

    /// Inline code pattern (`...`)
    static ref INLINE_CODE_PATTERN: Regex = Regex::new(r"`[^`]+`")
        .expect("Invalid inline code regex");

    /// URL pattern
    static ref URL_PATTERN: Regex = Regex::new(r"https?://\S+")
        .expect("Invalid URL regex");

    /// Bold markdown pattern (**text**)
    static ref BOLD_ASTERISK_PATTERN: Regex = Regex::new(r"\*\*([^*]+)\*\*")
        .expect("Invalid bold asterisk regex");

    /// Italic markdown pattern (*text*)
    static ref ITALIC_ASTERISK_PATTERN: Regex = Regex::new(r"\*([^*]+)\*")
        .expect("Invalid italic asterisk regex");

    /// Bold markdown pattern (__text__)
    static ref BOLD_UNDERSCORE_PATTERN: Regex = Regex::new(r"__([^_]+)__")
        .expect("Invalid bold underscore regex");

    /// Italic markdown pattern (_text_)
    static ref ITALIC_UNDERSCORE_PATTERN: Regex = Regex::new(r"_([^_]+)_")
        .expect("Invalid italic underscore regex");

    /// Header markdown pattern (# text)
    static ref HEADER_PATTERN: Regex = Regex::new(r"(?m)^#{1,6}\s+")
        .expect("Invalid header regex");

    /// List marker pattern (- item or * item)
    static ref LIST_MARKER_PATTERN: Regex = Regex::new(r"(?m)^\s*[-*+]\s+")
        .expect("Invalid list marker regex");

    /// Numbered list pattern (1. item)
    static ref NUMBERED_LIST_PATTERN: Regex = Regex::new(r"(?m)^\s*\d+\.\s+")
        .expect("Invalid numbered list regex");

    /// Multiple whitespace pattern
    static ref WHITESPACE_PATTERN: Regex = Regex::new(r"\s+")
        .expect("Invalid whitespace regex");
}

/// Clean text for speech synthesis
/// 
/// Removes:
/// - Emojis
/// - Code blocks (```...```)
/// - Inline code (`...`)
/// - URLs
/// - Markdown formatting (bold, italic, headers, lists)
/// - Extra whitespace
/// 
/// # Arguments
/// * `text` - Input text to clean
/// 
/// # Returns
/// Cleaned text suitable for TTS
pub fn clean_text_for_speech(text: &str) -> String {
    use std::borrow::Cow;
    
    // Usar Cow para evitar allocaciones cuando no hay cambios
    // Solo convierte a String owned cuando hay un reemplazo real
    let result: Cow<str> = CODE_BLOCK_PATTERN.replace_all(text, " código omitido ");
    let result: Cow<str> = INLINE_CODE_PATTERN.replace_all(&result, "");
    let result: Cow<str> = EMOJI_PATTERN.replace_all(&result, "");
    let result: Cow<str> = URL_PATTERN.replace_all(&result, "");
    let result: Cow<str> = BOLD_ASTERISK_PATTERN.replace_all(&result, "$1");
    let result: Cow<str> = ITALIC_ASTERISK_PATTERN.replace_all(&result, "$1");
    let result: Cow<str> = BOLD_UNDERSCORE_PATTERN.replace_all(&result, "$1");
    let result: Cow<str> = ITALIC_UNDERSCORE_PATTERN.replace_all(&result, "$1");
    let result: Cow<str> = HEADER_PATTERN.replace_all(&result, "");
    let result: Cow<str> = LIST_MARKER_PATTERN.replace_all(&result, "");
    let result: Cow<str> = NUMBERED_LIST_PATTERN.replace_all(&result, "");
    let result: Cow<str> = WHITESPACE_PATTERN.replace_all(&result, " ");

    // Solo al final convertimos a String y trimmeamos
    result.trim().to_string()
}

/// Split text into sentences for streaming synthesis
/// 
/// # Arguments
/// * `text` - Input text to split
/// 
/// # Returns
/// Vector of sentences
pub fn split_into_sentences(text: &str) -> Vec<String> {
    // Split on sentence-ending punctuation followed by whitespace
    // Optimizado: usar una sola pasada con chars() en lugar de múltiples replace()
    let mut result = String::with_capacity(text.len() + text.len() / 20); // Pequeño overhead para marcadores
    let mut chars = text.chars().peekable();
    
    while let Some(c) = chars.next() {
        result.push(c);
        // Si es puntuación final y el siguiente es espacio o newline, insertar marcador
        if c == '.' || c == '!' || c == '?' {
            if let Some(&next) = chars.peek() {
                if next == ' ' || next == '\n' {
                    result.push_str("|SPLIT|");
                    chars.next(); // Consumir el espacio/newline
                }
            }
        }
    }
    
    let sentences: Vec<String> = result
        .split("|SPLIT|")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if sentences.is_empty() {
        return vec![text.to_string()];
    }

    // Merge only *obvious* fragments (e.g. abbreviations). The previous
    // implementation merged almost every short sentence, which breaks
    // streaming chunking and unit tests.
    fn is_abbrev(s: &str) -> bool {
        let t = s.trim();
        matches!(
            t,
            "Dr." | "Dra." | "Sr." | "Sra." | "Srta." | "Prof." | "Mr." | "Mrs." | "Ms." | "St." | "etc." | "e.g." | "i.e." | "p.ej." | "ej."
        )
    }

    let mut result: Vec<String> = Vec::new();
    for sentence in sentences {
        if let Some(last) = result.last() {
            if is_abbrev(last) {
                if let Some(last_mut) = result.last_mut() {
                    *last_mut = format!("{} {}", last_mut, sentence);
                    continue;
                }
            }
        }
        result.push(sentence);
    }

    if result.is_empty() {
        vec![text.to_string()]
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_code_blocks() {
        let text = "Hello ```rust\nfn main() {}\n``` world";
        let cleaned = clean_text_for_speech(text);
        assert!(cleaned.contains("código omitido"));
        assert!(!cleaned.contains("fn main"));
    }

    #[test]
    fn test_clean_inline_code() {
        let text = "Use `println!` macro";
        let cleaned = clean_text_for_speech(text);
        assert!(!cleaned.contains("`"));
        assert!(!cleaned.contains("println"));
    }

    #[test]
    fn test_clean_urls() {
        let text = "Visit https://example.com for more";
        let cleaned = clean_text_for_speech(text);
        assert!(!cleaned.contains("https"));
        assert!(!cleaned.contains("example.com"));
    }

    #[test]
    fn test_clean_markdown() {
        let text = "This is **bold** and *italic*";
        let cleaned = clean_text_for_speech(text);
        assert!(!cleaned.contains("*"));
        assert!(cleaned.contains("bold"));
        assert!(cleaned.contains("italic"));
    }

    #[test]
    fn test_split_sentences() {
        let text = "Hello world. How are you? I am fine!";
        let sentences = split_into_sentences(text);
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn test_whitespace_normalization() {
        let text = "Hello    world\n\ntest";
        let cleaned = clean_text_for_speech(text);
        assert_eq!(cleaned, "Hello world test");
    }
}
