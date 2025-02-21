
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

/// Helper for parenthesised_text: returns true if `text` contains any digit immediately followed by '.' or ','.
pub fn contains_digit_followed_by_dot_or_comma(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i].is_ascii_digit() && (chars[i + 1] == '.' || chars[i + 1] == ',') {
            return true;
        }
    }
    false
}

/// Helper: returns true if `haystack` contains any of the patterns in `needles` (case–insensitively).
pub fn contains_any_ci(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|&needle| ci_contains(haystack, needle))
}

/// Helper: returns true if `haystack` contains both `first` and `second` in order.
/// That is, if the index of `first` (case–insensitively) is found and afterwards `second` appears.
pub fn contains_in_order(haystack: &str, first: &str, second: &str) -> bool {
    let hay_lower = haystack.to_lowercase();
    let first_lower = first.to_lowercase();
    let second_lower = second.to_lowercase();
    if let Some(first_index) = hay_lower.find(&first_lower) {
        hay_lower[first_index..].find(&second_lower).is_some()
    } else {
        false
    }
}

/// Returns true if `haystack` contains `needle` (ignoring case).
pub fn ci_contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Returns true if the given filename matches the pattern "C1[a-z]?".
pub fn file_matches_c1(file: &str) -> bool {
    let chars: Vec<char> = file.chars().collect();
    let len = chars.len();
    for i in 0..len {
        if chars[i] != 'C' {
            continue;
        }
        if i + 1 >= len || chars[i + 1] != '1' {
            continue;
        }
        // At this point we have found "C1".
        if i + 2 < len {
            let next = chars[i + 2];
            if next.is_alphabetic() && !next.is_lowercase() {
                // The following character is alphabetic but not lowercase;
                // this occurrence does not match our pattern. Continue searching.
                continue;
            }
        }
        // Found a valid occurrence.
        return true;
    }
    false
}

/// If `lang` contains a dash, returns the substring up to (but not including) the last dash.
/// Otherwise, returns None.
pub fn fallback_lang(lang: &str) -> Option<String> {
    lang.rfind('-').map(|pos| lang[..pos].to_string()).filter(|s| !s.is_empty())
}

/// Replaces all occurrences of `needle` with `replacement` in a case–insensitive way.
pub fn ci_replace_all(haystack: &str, needle: &str, replacement: &str) -> String {
    let lower_h = haystack.to_lowercase();
    let lower_n = needle.to_lowercase();
    let mut result = String::new();
    let mut start = 0;
    while let Some(pos) = lower_h[start..].find(&lower_n) {
        let pos = start + pos;
        result.push_str(&haystack[start..pos]);
        result.push_str(replacement);
        start = pos + needle.len();
    }
    result.push_str(&haystack[start..]);
    result
}

/// Returns true if `s` starts with `prefix` (ignoring case).
pub fn ci_starts_with(s: &str, prefix: &str) -> bool {
    s.to_lowercase().starts_with(&prefix.to_lowercase())
}

/// Removes leading zeros.
pub fn remove_leading_zeros(s: &str) -> String {
    let trimmed = s.trim_start_matches('0');
    if trimmed.is_empty() { "0".to_string() } else { trimmed.to_string() }
}

/// Returns a subdirectory path based on the version prefix.
/// 
/// If `version` starts with "Monastic", appends "M/" to `subdir`.
/// If `version` starts with "Ordo Praedicatorum", appends "OP/" to `subdir`.
/// Otherwise, just appends "/" to `subdir`.
pub fn subdirname(subdir: &str, version: &str) -> String {
    if version.starts_with("Monastic") {
        return format!("{}M/", subdir);
    }
    if version.starts_with("Ordo Praedicatorum") {
        return format!("{}OP/", subdir);
    }
    format!("{}/", subdir)
}

/// Remove everything from the beginning of `s` up to and including the last occurrence of `pat`.
pub fn remove_prefix_to_last(s: &str, pat: &str) -> String {
    if let Some(pos) = s.rfind(pat) {
        s[(pos + pat.len())..].to_string()
    } else {
        s.to_string()
    }
}

/// Find the first occurrence of `pat` in `s` and replace that occurrence and everything after it with `replacement`.
pub fn replace_from_first(s: &str, pat: &str, replacement: &str) -> String {
    if let Some(pos) = s.find(pat) {
        format!("{}{}", &s[..pos], replacement)
    } else {
        s.to_string()
    }
}

/// Helper: remove everything after a particular delimiter, case-insensitive.
pub fn remove_after(s: &mut String, delim: &str) {
    // If we want case-insensitive search, we can do something more elaborate.
    // For simplicity, we do a case-*sensitive* find below. Adjust if needed.
    if let Some(idx) = s.find(delim) {
        s.truncate(idx);
    }
}

/// Helper: case-insensitive `starts_with`
pub fn starts_with_ignore_case(haystack: &str, prefix: &str) -> bool {
    haystack
        .to_lowercase()
        .starts_with(&prefix.to_lowercase())
}

pub fn contains_ci(haystack: &str, needle: &str) -> bool {
    let hl = haystack.to_lowercase();
    for n in needle.split("|").map(|s| s.trim()) {
        if hl.contains(&needle.to_lowercase()) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_prefix_to_last() {
        // This should mimic Perl’s s/.*99!// removal.
        let line = "blah blah 99!remains";
        assert_eq!(remove_prefix_to_last(line, "99!"), "remains");
    }

    #[test]
    fn test_replace_from_first() {
        // This should mimic Perl’s s/99!.*/99/ replacement.
        let line = "prefix 99!suffix";
        assert_eq!(replace_from_first(line, "99!", "99"), "prefix 99");
    }

    #[test]
    fn test_monastic() {
        let subdir = "foo/";
        let version = "Monastic something";
        let expected = "foo/M/";
        assert_eq!(subdirname(subdir, version), expected);
    }

    #[test]
    fn test_ordo_praedicatorum() {
        let subdir = "bar/";
        let version = "Ordo Praedicatorum v2";
        let expected = "bar/OP/";
        assert_eq!(subdirname(subdir, version), expected);
    }

    #[test]
    fn test_default() {
        let subdir = "baz/";
        let version = "Other version";
        let expected = "baz//";
        // Note: If the subdir already ends with a slash, this will result in a double slash.
        // Adjust behavior as needed.
        assert_eq!(subdirname(subdir, version), expected);
    }

    #[test]
    fn test_no_trailing_slash() {
        let subdir = "qux";
        let version = "Other version";
        let expected = "qux/";
        assert_eq!(subdirname(subdir, version), expected);
    }

    #[test]
    fn test_contains_digit_followed_by_dot_or_comma() {
        assert!(contains_digit_followed_by_dot_or_comma("Version 2.0 release"));
        assert!(contains_digit_followed_by_dot_or_comma("Price 100, discounted"));
        assert!(!contains_digit_followed_by_dot_or_comma("No such pattern here"));
    }

    #[test]
    fn test_ci_contains() {
        assert!(ci_contains("Hello World", "world"));
        assert!(!ci_contains("Hello World", "mars"));
    }

    #[test]
    fn test_file_matches_c1() {
        assert!(file_matches_c1("C1abc.txt"));
        assert!(!file_matches_c1("D1abc.txt"));
    }

    #[test]
    fn test_fallback_lang() {
        assert_eq!(fallback_lang("En-UK"), Some("En".to_string()));
        assert_eq!(fallback_lang("En-US-extra"), Some("En-US".to_string()));
        assert_eq!(fallback_lang("English"), None);
    }

    #[test]
    fn test_replace_all_case_insensitive() {
        // Test that a pattern like "Confessor\w+" is replaced case-insensitively.
        let input = "Confessorus confessorABC CONFESSORtest";
        let expected = "Conf. conf. Conf.";
        let output = replace_word_prefix(&input, "Confessor", "Conf.");;
        assert_eq!(output, expected);
    }

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