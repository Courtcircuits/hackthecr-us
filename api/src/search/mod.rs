pub mod service;
pub mod router;
pub mod handlers;

use unicode_normalization::UnicodeNormalization;

pub fn clean_word(keyword: String) -> String {
    keyword
        .to_lowercase()
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect::<String>()
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect()
}
