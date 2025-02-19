
/// Replaces words in `s` that start with `prefix` (case-insensitively) by `replacement`.
/// The function splits the string on whitespace and reassembles it with single spaces.
/// (Punctuation attached to words is considered part of the word.)
pub fn replace_word_prefix(s: &str, prefix: &str, replacement: &str) -> String {
    let prefix_lower = prefix.to_lowercase();
    s.split_whitespace()
        .map(|word| {
            if word.to_lowercase().starts_with(&prefix_lower) {
                replacement.to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Helper function: replaces all occurrences of a pattern in a case–insensitive manner.
/// This is a simple implementation that uses the regex crate.
/// Replaces all occurrences of `pat` in `s` with `rep` in a case–insensitive manner.
/// This implementation does not use regex but uses simple string search and slicing.
///
/// If `pat` is empty, the original string is returned unchanged.
pub fn replace_all_case_insensitive(s: &str, pat: &str, rep: &str) -> String {
    if pat.is_empty() {
        return s.to_string();
    }
    
    // Convert the source string and the pattern to lowercase for case-insensitive matching.
    let s_lower = s.to_lowercase();
    let pat_lower = pat.to_lowercase();
    
    let mut result = String::with_capacity(s.len());
    let mut start = 0;
    
    // Loop while we can find the pattern in the lowercase version.
    while let Some(pos) = s_lower[start..].find(&pat_lower) {
        let match_start = start + pos;
        // Append the part of the original string before the match.
        result.push_str(&s[start..match_start]);
        // Append the replacement.
        result.push_str(rep);
        // Advance the start index beyond the matched pattern.
        start = match_start + pat.len();
    }
    
    // Append the remaining part of the original string.
    result.push_str(&s[start..]);
    result
}

/// Searches for all occurrences of `needle` in `s` (which is assumed to be lowercase)
/// and returns `true` if there is at least one occurrence that is NOT followed
/// (anywhere later in the string) by `forbidden` (which should also be lowercase).
///
/// This mimics the negative lookahead in the regex for "Pentecosten(?!.*infra octavam)".
pub fn contains_without_following(s: &str, needle: &str, forbidden: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(pos) = s[start..].find(needle) {
        let pos = start + pos;
        let after = &s[pos + needle.len()..];
        if !after.contains(forbidden) {
            return true;
        }
        // Continue search after this occurrence.
        start = pos + 1;
    }
    false
}

/// --- taken from regex_syntax

/// Escapes all regular expression meta characters in `text`.
///
/// The string returned may be safely used as a literal in a regular
/// expression.
pub fn escape(text: &str) -> String {
    let mut quoted = String::new();
    escape_into(text, &mut quoted);
    quoted
}

/// Escapes all meta characters in `text` and writes the result into `buf`.
///
/// This will append escape characters into the given buffer. The characters
/// that are appended are safe to use as a literal in a regular expression.
pub fn escape_into(text: &str, buf: &mut String) {
    buf.reserve(text.len());
    for c in text.chars() {
        if is_meta_character(c) {
            buf.push('\\');
        }
        buf.push(c);
    }
}

pub fn is_meta_character(c: char) -> bool {
    match c {
        '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{'
        | '}' | '^' | '$' | '#' | '&' | '-' | '~' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_without_following() {
        // Lowercase string used in the test.
        let s = "pentecosten celebration";
        // "infra octavam" does not appear after "pentecosten"
        assert!(contains_without_following(s, "pentecosten", "infra octavam"));

        // If "infra octavam" appears after "pentecosten", then no match.
        let s2 = "pentecosten something infra octavam later";
        assert!(!contains_without_following(s2, "pentecosten", "infra octavam"));

        // If "infra octavam" appears before, it does not affect the occurrence.
        let s3 = "infra octavam then pentecosten celebration";
        assert!(contains_without_following(s3, "pentecosten", "infra octavam"));
    }

    #[test]
    fn test_replace_all_case_insensitive_basic() {
        let s = "Hello world, HELLO WORLD, hello World!";
        // Replace "hello" (case-insensitive) with "hi".
        let result = replace_all_case_insensitive(s, "hello", "hi");
        assert_eq!(result, "hi world, hi WORLD, hi World!");
    }

    #[test]
    fn test_replace_all_case_insensitive_no_match() {
        let s = "Rust is awesome!";
        // Replace a pattern that does not exist.
        let result = replace_all_case_insensitive(s, "python", "Java");
        assert_eq!(result, s);
    }

    #[test]
    fn test_replace_all_case_insensitive_empty_pattern() {
        let s = "Nothing changes";
        // With an empty pattern, the original string should be returned.
        let result = replace_all_case_insensitive(s, "", "replacement");
        assert_eq!(result, s);
    }

}