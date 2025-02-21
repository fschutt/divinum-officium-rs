//! language_text_tools.rs
//!
//! This module provides text–processing routines for Divinum Officium’s
//! multilingual data (prayers, rubrics, preces, translations). It mirrors the
//! original LanguageTextTools.pm module. In particular, it offers functions to:
//!
//! - Remove or process inline “Alleluia” strings,
//! - Ensure that text ends with a single or double Alleluia (translated appropriately),
//! - Look up translations, prayers, rubrics, and preces using language–specific maps,
//! - And load the language data from disk.
//!
//! File–loading is performed via a setup provider (of type implementing
//! [`SetupStringProvider`]) which, in production, is the real SetupStringContext
//! (a struct) from the `setupstring` module.

use std::collections::HashMap;
use crate::setup_string::{FileSections, ResolveDirectives, SetupStringProvider};

/// Holds the language data previously stored in globals.
#[derive(Debug, Default)]
pub struct LanguageTextContext {
    pub translate: HashMap<String, FileSections>,
    pub prayers: HashMap<String, FileSections>,
    pub rubrics: HashMap<String, FileSections>,
    pub preces: HashMap<String, FileSections>,
    pub alleluia_variants: Vec<String>,
    pub omit_words: Vec<String>,
    pub fb_lang: String,
    pub version: String,
}

/// Initializes a new LanguageTextContext using data loaded from disk via the given
/// setup provider. (In production, pass a mutable reference to your SetupStringContext.)
pub fn initialize_language_text_context(
    setup_ctx: &mut dyn SetupStringProvider,
    lang1: &str,
    lang2: &str,
    langfb: &str,
    version: &str,
    missaf: bool,
) -> LanguageTextContext {
    // Always include Latin first, then the others.
    let mut langs = vec![
        "Latin".to_string(),
        lang1.to_string(),
        lang2.to_string(),
        langfb.to_string(),
    ];
    {
        let mut seen = HashMap::new();
        langs.retain(|l| seen.insert(l.clone(), ()) == None);
    }

    let mut translate = HashMap::new();
    let mut prayers = HashMap::new();
    let mut rubrics = HashMap::new();
    let mut preces = HashMap::new();

    let dir = if missaf { "Ordo" } else { "Psalterium/Common" };
    let res = ResolveDirectives::All;

    for lang in &langs {
        let key = format!("{}{}", lang, version);
        // Load the prayers, rubrics, preces and translation maps.
        let prayers_data = setup_ctx
            .setupstring(lang, &format!("{}/Prayers.txt", dir), res)
            .unwrap_or_default();
        prayers.insert(key.clone(), prayers_data);

        let rubrics_data = setup_ctx
            .setupstring(lang, "Psalterium/Common/Rubricae.txt", res)
            .unwrap_or_default();
        rubrics.insert(key.clone(), rubrics_data);

        let preces_data = setup_ctx
            .setupstring(lang, "Psalterium/Special/Preces.txt", res)
            .unwrap_or_default();
        preces.insert(key.clone(), preces_data);

        let translate_data = setup_ctx
            .setupstring(lang, "Psalterium/Common/Translate.txt", res)
            .unwrap_or_default();
        translate.insert(lang.clone(), translate_data);
    }

    // Build the alleluia variants vector.
    let mut alleluias: Vec<String> = langs
        .iter()
        .map(|l| alleluia_from_prayers(&prayers, l, langfb, version).to_lowercase())
        .collect();
    // Append an extra literal alternative.
    alleluias.push("alleluia".to_string());

    // Build omit words.
    let mut omits = Vec::new();
    for lang in &langs {
        if let Some(comm) = setup_ctx.setupstring(lang, "Psalterium/Comment.txt", res) {
            if let Some(preces_val) = comm.get("Preces") {
                let lines: Vec<&str> = preces_val.split('\n').collect();
                if lines.len() > 1 {
                    omits.push(lines[1].to_string());
                }
            }
            if let Some(suff_val) = comm.get("Suffragium") {
                let lines: Vec<&str> = suff_val.split('\n').collect();
                if !lines.is_empty() {
                    omits.push(lines[0].to_string());
                }
            }
        }
    }

    LanguageTextContext {
        translate,
        prayers,
        rubrics,
        preces,
        alleluia_variants: alleluias,
        omit_words: omits,
        fb_lang: langfb.to_string(),
        version: version.to_string(),
    }
}

/// Helper: compute the “Alleluia” string from the prayer “Alleluia” using fallback rules.
fn alleluia_from_prayers(
    prayers: &HashMap<String, HashMap<String, String>>,
    lang: &str,
    fb_lang: &str,
    version: &str,
) -> String {
    let key1 = format!("{}{}", lang, version);
    let key2 = format!("{}{}", fb_lang, version);
    let key3 = format!("Latin{}", version);
    let text = prayers
        .get(&key1)
        .and_then(|m| m.get("Alleluia"))
        .or_else(|| prayers.get(&key2).and_then(|m| m.get("Alleluia")))
        .or_else(|| prayers.get(&key3).and_then(|m| m.get("Alleluia")))
        .cloned()
        .unwrap_or_else(|| "Alleluia".to_string());
    if text.starts_with("v. ") {
        let after = &text[3..];
        if let Some(dot) = after.find('.') {
            return after[..dot].to_string();
        }
    }
    text
}

/// Returns the “Alleluia” text for the given language.
pub fn alleluia(ctx: &LanguageTextContext, lang: &str) -> String {
    alleluia_from_prayers(&ctx.prayers, lang, &ctx.fb_lang, &ctx.version)
}

/// Removes any trailing alleluia (and optional punctuation) from the given text.
pub fn suppress_alleluia(ctx: &LanguageTextContext, text: &mut String) {
    *text = remove_trailing_alleluia(text, &ctx.alleluia_variants);
}

/// Helper that removes a trailing alleluia variant (ignoring case and punctuation).
fn remove_trailing_alleluia(text: &str, variants: &[String]) -> String {
    let trimmed = text.trim_end();
    let lower = trimmed.to_lowercase();
    for allele in variants {
        let allele_lower = allele.to_lowercase();
        if lower.ends_with(&allele_lower) {
            if let Some(pos) = lower.rfind(&allele_lower) {
                let prefix = &trimmed[..pos];
                return prefix
                    .trim_end_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
                    .to_string();
            }
        }
    }
    text.to_string()
}

/// Processes inline alleluias in the given text.
/// If `paschalf` is true, unbrackets them; otherwise, removes them.
pub fn process_inline_alleluias(ctx: &LanguageTextContext, text: &mut String, paschalf: bool) {
    *text = process_inline_alleluias_helper(text, &ctx.alleluia_variants, paschalf);
}

/// Helper: looks for a parenthesized group whose content (trimmed, lowercased)
/// begins with an alleluia variant. If found, either unbrackets it or removes it.
fn process_inline_alleluias_helper(text: &str, variants: &[String], paschalf: bool) -> String {
    let mut result = String::new();
    let mut remaining = text;
    while let Some(start_idx) = remaining.find('(') {
        if let Some(end_idx) = remaining[start_idx..].find(')') {
            let end_idx = start_idx + end_idx;
            let before = &remaining[..start_idx];
            let inside = &remaining[start_idx + 1..end_idx];
            let after = &remaining[end_idx + 1..];
            let inside_lower = inside.trim().to_lowercase();
            let mut is_alleluia = false;
            for allele in variants {
                if inside_lower.starts_with(&allele.to_lowercase()) {
                    is_alleluia = true;
                    break;
                }
            }
            result.push_str(before);
            if is_alleluia && paschalf {
                result.push(' ');
                result.push_str(inside.trim());
                result.push(' ');
            }
            remaining = after;
        } else {
            break;
        }
    }
    result.push_str(remaining);
    result
}

/// Ensures that the given text ends with a single alleluia.
/// If not, appends “, alleluja.” (with the alleluia taken from the appropriate language).
pub fn ensure_single_alleluia(ctx: &LanguageTextContext, text: &mut String, lang: &str) {
    if !text_ends_with_alleluia(text, &ctx.alleluia_variants) {
        let addition = format!(", {}.", alleluia(ctx, lang).to_lowercase());
        text.push_str(&addition);
    }
}

/// Returns true if, after trimming, the text ends with one of the given alleluia variants.
fn text_ends_with_alleluia(text: &str, variants: &[String]) -> bool {
    let trimmed = text.trim_end().to_lowercase();
    for allele in variants {
        if trimmed.ends_with(&allele.to_lowercase()) {
            return true;
        }
    }
    false
}

/// Ensures that the text ends with a double alleluia (with an asterisk inserted).
///
/// If the text does not already end with two alleluias (ignoring punctuation),
/// first any asterisk marker is “moved” (i.e. removed and the following character lowercased),
/// then trailing punctuation is trimmed and the string
/// `", * {alleluia}, {alleluia_lower}."` is appended.
pub fn ensure_double_alleluia(ctx: &LanguageTextContext, text: &mut String, lang: &str) {
    // Compute the alleluia text (for both positions) and its lowercase.
    let allele = alleluia(ctx, lang);
    let allele_lower = allele.to_lowercase();
    // Check if the text already ends with two consecutive alleluias.
    if !text_ends_with_double_alleluia(text, &allele_lower) {
        // Remove any stray asterisk marker and lowercase its following character.
        *text = remove_asterisk_marker(text);
        // Trim any trailing punctuation or whitespace.
        let base = text
            .trim_end_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
            .to_string();
        // Append the double alleluia string.
        let addition = format!(", * {}, {}.", allele, allele_lower);
        *text = base + &addition;
    }
}

/// Returns true if the last two “words” (ignoring trailing punctuation)
/// in the text (after lowercasing) are equal to the given allele_lower.
fn text_ends_with_double_alleluia(text: &str, allele_lower: &str) -> bool {
    let text2 = text.trim_end().to_lowercase();
    let parts: Vec<&str> = text2.split_whitespace().collect();
    if parts.len() < 2 {
        return false;
    }
    let last = parts[parts.len() - 1].trim_matches(|c: char| c.is_ascii_punctuation());
    let second_last = parts[parts.len() - 2].trim_matches(|c: char| c.is_ascii_punctuation());
    (second_last == allele_lower) && (last == allele_lower)
}

/// Removes an asterisk marker (an asterisk optionally surrounded by whitespace)
/// and lowercases the character following it.
///
/// This mimics the Perl substitution:
///   s/\s*\*\s*(.)/ \l$1/
fn remove_asterisk_marker(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '*' {
            // Skip any whitespace immediately following the asterisk.
            while let Some(&nc) = chars.peek() {
                if nc.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            // If there is a next character, append a space followed by its lowercase.
            if let Some(nc) = chars.next() {
                result.push(' ');
                result.extend(nc.to_lowercase());
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Returns “Alleluja, * alleluja, alleluja.” for the given language.
pub fn alleluia_ant(ctx: &LanguageTextContext, lang: &str) -> String {
    let u = alleluia(ctx, lang);
    let l = u.to_lowercase();
    format!("{}, * {}, {}.", u, l, l)
}

/// Returns the translated text for the given name and language.
pub fn translate(ctx: &LanguageTextContext, name: &str, lang: &str) -> String {
    let mut prefix = String::new();
    let mut name = name.to_string();
    if name.starts_with('$') || name.starts_with('&') {
        prefix = name.chars().next().unwrap().to_string();
        name = name[1..].to_string();
    }
    let result = if lang.to_lowercase().contains("latin") {
        ctx.translate
            .get("Latin")
            .and_then(|m| m.get(&name))
            .map(|s| s.trim_end().to_string())
            .unwrap_or(name.clone())
    } else {
        ctx.translate
            .get(lang)
            .and_then(|m| m.get(&name))
            .or_else(|| ctx.translate.get(&ctx.fb_lang).and_then(|m| m.get(&name)))
            .or_else(|| ctx.translate.get("Latin").and_then(|m| m.get(&name)))
            .cloned()
            .unwrap_or(name.clone())
            .trim_end()
            .to_string()
    };
    format!("{}{}", prefix, result)
}

/// Returns the prayer text for the given name and language,
/// using fallback order: lang → fallback → Latin.
/// Also, if the version string contains “cist” (case–insensitive) and
/// the name is not exempt, removes occurrences of “+ ”.
pub fn prayer(ctx: &LanguageTextContext, name: &str, lang: &str) -> String {
    let version = &ctx.version;
    let key1 = format!("{}{}", lang, version);
    let key2 = format!("{}{}", ctx.fb_lang, version);
    let key3 = format!("Latin{}", version);
    let candidate = ctx
        .prayers
        .get(&key1)
        .and_then(|m| m.get(name))
        .or_else(|| ctx.prayers.get(&key2).and_then(|m| m.get(name)))
        .or_else(|| ctx.prayers.get(&key3).and_then(|m| m.get(name)))
        .cloned()
        .unwrap_or_else(|| name.to_string());
    if version.to_lowercase().contains("cist") && !name_contains_exempt(name) {
        candidate.replace("+ ", "")
    } else {
        candidate
    }
}

/// Helper: returns true if the name contains exempt patterns.
fn name_contains_exempt(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("pater ave")
        || lower.contains("incipit")
        || lower.contains("clara")
        || (lower.contains("bene") && lower.contains("final"))
}

/// Returns the rubric text for the given name and language.
pub fn rubric(ctx: &LanguageTextContext, name: &str, lang: &str) -> String {
    let version = &ctx.version;
    let key1 = format!("{}{}", lang, version);
    let key2 = format!("{}{}", ctx.fb_lang, version);
    let key3 = format!("Latin{}", version);
    ctx.rubrics
        .get(&key1)
        .and_then(|m| m.get(name))
        .cloned()
        .or_else(|| ctx.rubrics.get(&key2).and_then(|m| m.get(name)).cloned())
        .or_else(|| ctx.rubrics.get(&key3).and_then(|m| m.get(name)).cloned())
        .unwrap_or_else(|| name.to_string())
}

/// Returns the preces text for the given name and language.
pub fn prex(ctx: &LanguageTextContext, name: &str, lang: &str) -> String {
    let version = &ctx.version;
    let key1 = format!("{}{}", lang, version);
    let key2 = format!("{}{}", ctx.fb_lang, version);
    let key3 = format!("Latin{}", version);
    ctx.preces
        .get(&key1)
        .and_then(|m| m.get(name))
        .cloned()
        .or_else(|| ctx.preces.get(&key2).and_then(|m| m.get(name)).cloned())
        .or_else(|| ctx.preces.get(&key3).and_then(|m| m.get(name)).cloned())
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::collections::HashMap;
    use crate::setup_string::{ResolveDirectives, SetupStringProvider};

    /// A dummy setup provider that holds a mapping from (lang, file) to key/value data.
    struct DummySetupStringContext {
        data: HashMap<(String, String), HashMap<String, String>>,
    }

    impl DummySetupStringContext {
        fn new() -> Self {
            DummySetupStringContext {
                data: HashMap::new(),
            }
        }
        fn set_dummy(&mut self, lang: &str, file: &str, content: HashMap<String, String>) {
            self.data.insert((lang.to_string(), file.to_string()), content);
        }
    }

    impl SetupStringProvider for DummySetupStringContext {
        fn setupstring(&mut self, lang: &str, file: &str, _res: ResolveDirectives) -> Option<HashMap<String, String>> {
            self.data.get(&(lang.to_string(), file.to_string())).cloned()
        }
    }

    // --- Tests for helper “parsing” functions ---

    #[test]
    fn test_remove_trailing_alleluia() {
        let variants = vec!["alleluja".to_string()];
        let input = "This is a prayer, Alleluja.  ";
        let expected = "This is a prayer";
        assert_eq!(super::remove_trailing_alleluia(input, &variants), expected);
    }

    #[test]
    fn test_process_inline_alleluias_paschalf() {
        let variants = vec!["alleluja".to_string()];
        let input = "Some text (Alleluja extra) and more text";
        let output = super::process_inline_alleluias_helper(input, &variants, true);
        assert_eq!(output, "Some text  Alleluja extra  and more text");
    }

    #[test]
    fn test_text_ends_with_double_alleluia() {
        let allele = "Alleluja".to_string();
        assert!(super::text_ends_with_double_alleluia("Some text Alleluja, Alleluja.", &allele.to_lowercase()));
        assert!(!super::text_ends_with_double_alleluia("Some text Alleluja.", &allele.to_lowercase()));
    }

    #[test]
    fn test_remove_asterisk_marker() {
        let input = "Response * A";
        let expected = "Response  a";
        assert_eq!(super::remove_asterisk_marker(input), expected);
    }

    // --- Tests for public functions using the context ---

    #[test]
    fn test_alleluia_function() {
        let mut dummy = DummySetupStringContext::new();
        let mut prayers = HashMap::new();
        prayers.insert("Alleluia".to_string(), "v. Alleluja. More text".to_string());
        dummy.set_dummy("Latin", "Psalterium/Common/Prayers.txt", prayers);

        let ctx = initialize_language_text_context(&mut dummy, "English", "German", "Latin", "1.00", false);
        assert_eq!(alleluia(&ctx, "Latin"), "Alleluja".to_string());
    }

    #[test]
    fn test_suppress_alleluia_public() {
        let mut dummy = DummySetupStringContext::new();
        let mut prayers = HashMap::new();
        prayers.insert("Alleluia".to_string(), "v. Alleluja. Extra".to_string());
        dummy.set_dummy("Latin", "Psalterium/Common/Prayers.txt", prayers);

        let ctx = initialize_language_text_context(&mut dummy, "English", "German", "Latin", "1.00", false);
        let mut text = "This is a prayer, alleluja.".to_string();
        suppress_alleluia(&ctx, &mut text);
        assert_eq!(text, "This is a prayer");
    }

    #[test]
    fn test_ensure_single_alleluia_public() {
        let mut dummy = DummySetupStringContext::new();
        let mut prayers = HashMap::new();
        prayers.insert("Alleluia".to_string(), "v. Alleluja. Extra".to_string());
        dummy.set_dummy("Latin", "Psalterium/Common/Prayers.txt", prayers);

        let ctx = initialize_language_text_context(&mut dummy, "English", "German", "Latin", "1.00", false);
        let mut text = "This is a prayer".to_string();
        ensure_single_alleluia(&ctx, &mut text, "Latin");
        assert_eq!(text, "This is a prayer, alleluja.");
    }

    #[test]
    fn test_ensure_double_alleluia_public() {
        let mut dummy = DummySetupStringContext::new();
        let mut prayers = HashMap::new();
        prayers.insert("Alleluia".to_string(), "v. Alleluja. Extra".to_string());
        prayers.insert("Alleluia Duplex".to_string(), "Alleluja Duplex".to_string());
        dummy.set_dummy("Latin", "Psalterium/Common/Prayers.txt", prayers);

        let ctx = initialize_language_text_context(&mut dummy, "English", "German", "Latin", "1.00", false);
        let mut text = "This is a response * A".to_string();
        ensure_double_alleluia(&ctx, &mut text, "Latin");
        assert!(text.ends_with(", * Alleluja, alleluja."));
    }

    #[test]
    fn test_alleluia_ant_public() {
        let mut dummy = DummySetupStringContext::new();
        let mut prayers = HashMap::new();
        prayers.insert("Alleluia".to_string(), "v. Alleluja. Extra".to_string());
        dummy.set_dummy("Latin", "Psalterium/Common/Prayers.txt", prayers);

        let ctx = initialize_language_text_context(&mut dummy, "English", "German", "Latin", "1.00", false);
        let ant = alleluia_ant(&ctx, "Latin");
        assert_eq!(ant, "Alleluja, * alleluja, alleluja.");
    }

    #[test]
    fn test_translate_public() {
        let mut dummy = DummySetupStringContext::new();
        let mut trans = HashMap::new();
        trans.insert("Test".to_string(), "TestTranslation".to_string());
        dummy.set_dummy("English", "Psalterium/Common/Translate.txt", trans);

        let mut latin_trans = HashMap::new();
        latin_trans.insert("Test".to_string(), "LatinTest".to_string());
        dummy.set_dummy("Latin", "Psalterium/Common/Translate.txt", latin_trans);

        let ctx = initialize_language_text_context(&mut dummy, "English", "German", "Latin", "1.00", false);
        let tr = translate(&ctx, "Test", "English");
        assert_eq!(tr, "TestTranslation");
        let tr_latin = translate(&ctx, "Test", "Latin");
        assert_eq!(tr_latin, "LatinTest");
    }

    #[test]
    fn test_prayer_public_cist() {
        let mut dummy = DummySetupStringContext::new();
        let mut prayers = HashMap::new();
        prayers.insert("Test".to_string(), "Some + text".to_string());
        dummy.set_dummy("English", "Psalterium/Common/Prayers.txt", prayers);

        let ctx = initialize_language_text_context(&mut dummy, "English", "German", "Latin", "Cist1", false);
        let pr = prayer(&ctx, "Test", "English");
        assert_eq!(pr, "Some text");
    }
}
