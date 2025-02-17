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
//! # Globals and Setup
//!
//! Internally, the module uses several global caches (wrapped in Mutexes) for:
//! - `_translate`: a map from language to translation mappings,
//! - `_prayers`, `_rubrics`, `_preces`: maps keyed by a composite “lang+version” string,
//! - `ALLELUIA_REGEX` and `OMIT_REGEX`: precompiled regular expressions,
//! - `FB_LANG`: the fallback language.
//!
//! The function `load_languages_data` must be called (typically once during setup)
//! to populate these globals using the “setupstring” function (assumed to be defined in
//! the setup module).
//!
//! The module also uses a global `VERSION` (here assumed to be set elsewhere) to pick
//! the correct version of the data.

use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::collections::HashMap;
use std::sync::Mutex;

// Assume that a function setupstring is provided in the setup module:
use crate::setup_string::{ResolveDirectives, SetupStringContext};

// Global mutable caches. Keys for _prayers, _rubrics, _preces are of the form "{lang}{version}".
// For _translate, the key is the language.
static TRANSLATE: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static PRAYERS: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static PRECES: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static RUBRICS: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Global regexes. These are set in load_languages_data.
static ALLELUIA_REGEX: Lazy<Mutex<Option<Regex>>> = Lazy::new(|| Mutex::new(None));
static OMIT_REGEX: Lazy<Mutex<Option<Regex>>> = Lazy::new(|| Mutex::new(None));

// Global fallback language.
static FB_LANG: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

// Global version string (to be set externally, analogous to $main::version).
static VERSION: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::from("1.00")));

/// Private helper: returns the “Alleluia” text for a given language.
/// It calls [`prayer("Alleluia", lang)`] and then removes a leading “v. ….” prefix.
fn alleluia(lang: &str) -> String {
    let text = prayer("Alleluia", lang);
    // Regex: match a line starting with "v. " then capture until the first period.
    let re = Regex::new(r"^v\. (.*?)\..*").unwrap();
    if let Some(caps) = re.captures(&text) {
        if let Some(m) = caps.get(1) {
            return m.as_str().to_string();
        }
    }
    text
}

/// Removes all occurrences of alleluia (and optional punctuation before it) from the given text.
/// This function mutates the provided string.
pub fn suppress_alleluia(text: &mut String) {
    if let Some(alleluia_re) = &*ALLELUIA_REGEX.lock().unwrap() {
        // Build a pattern that matches an optional comma/period, whitespace, then the alleluia text,
        // possibly followed by a closing parenthesis and trailing whitespace.
        let pattern = format!(r"(?i)[,\.]?\s*{}[\p{{P}}]?\s*$", alleluia_re.as_str());
        if let Ok(re) = Regex::new(&pattern) {
            *text = re.replace_all(text, "").to_string();
        }
    }
}

/// Processes inline alleluias in the given text.
/// If `paschalf` is true, unbrackets any alleluias (i.e. removes the surrounding parentheses);
/// otherwise, removes the bracketed alleluias altogether.
pub fn process_inline_alleluias(text: &mut String, paschalf: bool) {
    if let Some(alleluia_re) = &*ALLELUIA_REGEX.lock().unwrap() {
        if paschalf {
            // Replace bracketed alleluias with the content inside, surrounded by spaces.
            let pattern = format!(r"(?is)\(({0}.*?)\)", alleluia_re.as_str());
            if let Ok(re) = Regex::new(&pattern) {
                *text = re.replace_all(text, " $1 ").to_string();
            }
        } else {
            // Remove bracketed alleluias entirely.
            let pattern = format!(r"(?is)\({0}.*?\)", alleluia_re.as_str());
            if let Ok(re) = Regex::new(&pattern) {
                *text = re.replace_all(text, "").to_string();
            }
        }
    }
}

/// Ensures that the given text ends with a single alleluia (in the appropriate language).
/// If not, appends a comma, space, the lower-case alleluia, and a period.
pub fn ensure_single_alleluia(text: &mut String, lang: &str) {
    if let Some(alleluia_re) = &*ALLELUIA_REGEX.lock().unwrap() {
        // Build a regex that checks for alleluia at the end of the text.
        let pattern = format!(r"(?i){}[\p{{P}}]?\)?\s*$", alleluia_re.as_str());
        if let Ok(re) = Regex::new(&pattern) {
            if !re.is_match(text) {
                let addition = format!(", {}.", alleluia(lang).to_lowercase());
                text.push_str(&addition);
            }
        }
    }
}

/// Ensures that the given text ends with a double alleluia (i.e. two alleluias, with an asterisk
/// correctly positioned). If not, it first removes any asterisk pattern, then replaces the trailing
/// punctuation with a comma, an asterisk, the appropriate alleluia texts, and a period.
pub fn ensure_double_alleluia(text: &mut String, lang: &str) {
    // Get the Alleluia Duplex text and trim trailing whitespace.
    let mut alleluia_duplex = prayer("Alleluia Duplex", lang);
    alleluia_duplex = alleluia_duplex.trim_end().to_string();

    if let Some(alleluia_re) = &*ALLELUIA_REGEX.lock().unwrap() {
        // Build a regex that checks if text already ends in a double alleluia.
        let pattern = format!(
            r"(?i){}[,.]\s+{}[\p{{P}}]?\s*$",
            alleluia_re.as_str(),
            alleluia_re.as_str()
        );
        if let Ok(re) = Regex::new(&pattern) {
            if !re.is_match(text) {
                // First, remove any asterisk marker and make the following character lowercase.
                let re_star = Regex::new(r"\s*\*\s*(.)").unwrap();
                *text = re_star
                    .replace_all(text, |caps: &regex::Captures| {
                        format!(" {}", caps[1].to_lowercase())
                    })
                    .to_string();
                // Now, replace trailing punctuation/whitespace with the double alleluia.
                let re_end = Regex::new(r"(?i)[\p{P}\s]*$").unwrap();
                let addition =
                    format!(", * {}, {}.", alleluia(lang), alleluia(lang).to_lowercase());
                *text = re_end.replace(text, addition.as_str()).to_string();
            }
        }
    }
}

/// Returns a string of the form “Alleluja, * alleluja, alleluja.” for the given language.
pub fn alleluia_ant(lang: &str) -> String {
    let u = alleluia(lang);
    let l = u.to_lowercase();
    format!("{}, * {}, {}.", u, l, l)
}

/// Returns a clone of the omit regexp (if it has been set).
pub fn omit_regexp() -> Option<Regex> {
    (*OMIT_REGEX.lock().unwrap()).clone()
}

/// Translates a name according to the language data.
/// If the name begins with '$' or '&', that character is preserved as a prefix.
/// For Latin, the translation is taken from the Latin translation table (with trailing whitespace removed).
/// For other languages, the lookup is performed in order: the given language, then the fallback language,
/// then Latin; if no translation is found, returns the original name (with the prefix, if any).
pub fn translate(name: &str, lang: &str) -> String {
    let mut prefix = String::new();
    let mut name = name.to_string();
    if name.starts_with('$') || name.starts_with('&') {
        prefix = name[0..1].to_string();
        name = name[1..].to_string();
    }

    let result = if lang.to_lowercase().contains("latin") {
        // Look up translation in the Latin table.
        TRANSLATE
            .lock()
            .unwrap()
            .get("Latin")
            .and_then(|map| map.get(&name))
            .map(|s| s.trim_end().to_string())
            .unwrap_or(name.clone())
    } else {
        // Try the language table, then the fallback language, then Latin.
        let fb = FB_LANG.lock().unwrap();
        let fb_lang = fb.as_ref().map(String::as_str).unwrap_or("");
        TRANSLATE
            .lock()
            .unwrap()
            .get(lang)
            .and_then(|m| m.get(&name))
            .cloned()
            .or_else(|| {
                TRANSLATE
                    .lock()
                    .unwrap()
                    .get(fb_lang)
                    .and_then(|m| m.get(&name))
                    .cloned()
            })
            .or_else(|| {
                TRANSLATE
                    .lock()
                    .unwrap()
                    .get("Latin")
                    .and_then(|m| m.get(&name))
                    .cloned()
            })
            .unwrap_or(name.clone())
    };
    format!("{}{}", prefix, result.trim_end())
}

/// Returns the prayer text for the given name and language.
/// It uses the global `VERSION` (and the fallback language `FB_LANG`) to determine which data map to use.
/// If the prayer is not found, returns the name itself.
/// If the version contains “cist” (case–insensitive) and the name does not match certain patterns,
/// then all “+ ” markers are removed.
pub fn prayer(name: &str, lang: &str) -> String {
    let version = VERSION.lock().unwrap().clone();
    let key1 = format!("{}{}", lang, version);
    let fb_lang = FB_LANG.lock().unwrap().clone().unwrap_or_else(|| String::from("Latin"));
    let key2 = format!("{}{}", fb_lang, version);
    let key3 = format!("Latin{}", version);
    let candidate = {
        let prayers = PRAYERS.lock().unwrap();
        prayers
            .get(&key1)
            .and_then(|m| m.get(name))
            .or_else(|| prayers.get(&key2).and_then(|m| m.get(name)))
            .or_else(|| prayers.get(&key3).and_then(|m| m.get(name)))
            .cloned()
            .unwrap_or_else(|| name.to_string())
    };
    // If version contains "cist" (case-insensitive) and name is not one of the listed patterns,
    // then remove occurrences of "+" followed by a space.
    if version.to_lowercase().contains("cist")
        && !Regex::new(r"(?i)Pater Ave|Incipit|clara|bene.*Final")
            .unwrap()
            .is_match(name)
    {
        return candidate.replace("+ ", "");
    }
    candidate
}

/// Returns the rubric text for the given name and language.
/// Uses the global `VERSION` and fallback language similar to [`prayer`].
pub fn rubric(name: &str, lang: &str) -> String {
    let version = VERSION.lock().unwrap().clone();
    let key1 = format!("{}{}", lang, version);
    let fb_lang = FB_LANG.lock().unwrap().clone().unwrap_or_else(|| String::from("Latin"));
    let key2 = format!("{}{}", fb_lang, version);
    let key3 = format!("Latin{}", version);
    PRAYERS
        .lock()
        .unwrap() // Note: In the original, rubric data is stored in _rubrics.
        .get(&key1)
        .and_then(|m| m.get(name))
        .cloned()
        .or_else(|| RUBRICS.lock().unwrap().get(&key2).and_then(|m| m.get(name)).cloned())
        .or_else(|| RUBRICS.lock().unwrap().get(&key3).and_then(|m| m.get(name)).cloned())
        .unwrap_or_else(|| name.to_string())
}

/// Returns the preces (i.e. short prayers) text for the given name and language.
/// Follows the same fallback rules as [`prayer`].
pub fn prex(name: &str, lang: &str) -> String {
    let version = VERSION.lock().unwrap().clone();
    let key1 = format!("{}{}", lang, version);
    let fb_lang = FB_LANG.lock().unwrap().clone().unwrap_or_else(|| String::from("Latin"));
    let key2 = format!("{}{}", fb_lang, version);
    let key3 = format!("Latin{}", version);
    PRECES
        .lock()
        .unwrap()
        .get(&key1)
        .and_then(|m| m.get(name))
        .cloned()
        .or_else(|| PRECES.lock().unwrap().get(&key2).and_then(|m| m.get(name)).cloned())
        .or_else(|| PRECES.lock().unwrap().get(&key3).and_then(|m| m.get(name)).cloned())
        .unwrap_or_else(|| name.to_string())
}

/// Loads the language data from disk for the given languages, fallback language,
/// version, and a flag indicating whether this is for missa (which affects the directory used).
///
/// This function sets the global caches for prayers, rubrics, preces, and translations,
/// and computes the alleluia and omit regexes based on the loaded data.
pub fn load_languages_data(
    ctx: &mut SetupStringContext,
    lang1: &str,
    lang2: &str,
    langfb: &str,
    version: &str,
    missaf: bool,
) {
    // Create a unique list of languages: always include "Latin" first.
    let mut langs = vec!["Latin".to_string(), lang1.to_string(), lang2.to_string(), langfb.to_string()];
    // Remove duplicates while preserving order.
    let mut seen = HashMap::new();
    langs.retain(|l| seen.insert(l.clone(), true).is_none());

    // Save fallback language for use in other functions.
    *FB_LANG.lock().unwrap() = Some(langfb.to_string());

    // Determine the directory based on the missaf flag.
    let dir = if missaf { "Ordo" } else { "Psalterium/Common" };

    let res = ResolveDirectives::All;

    // For each language, load the appropriate data files.
    for lang in &langs {
        let key = format!("{}{}", lang, version);
        let prayers = ctx.setupstring(lang, &format!("{}/Prayers.txt", dir), res).unwrap_or_default();
        PRAYERS.lock().unwrap().insert(key.clone(), prayers);

        let rubrics = ctx.setupstring(lang, "Psalterium/Common/Rubricae.txt", res).unwrap_or_default();
        RUBRICS.lock().unwrap().insert(key.clone(), rubrics);

        let preces = ctx.setupstring(lang, "Psalterium/Special/Preces.txt", res).unwrap_or_default();
        PRECES.lock().unwrap().insert(key.clone(), preces);

        let translate_map = ctx.setupstring(lang, "Psalterium/Common/Translate.txt", res).unwrap_or_default();
        TRANSLATE.lock().unwrap().insert(lang.clone(), translate_map);
    }

    // Compute the alleluia regular expression from the alleluia translations.
    // For each language, get the lower-case version of alleluia(lang).
    let mut alleluias: Vec<String> = langs
        .iter()
        .map(|l| alleluia(l).to_lowercase())
        .collect();
    // Append an alternative spelling pattern.
    alleluias.push("allel[uú][ij]a".to_string());
    let alleluias_joined = alleluias.join("|");

    // Compile ALLELUIA_REGEX with case-insensitive flag.
    let alleluia_re = RegexBuilder::new(&format!(r"(?:{})", alleluias_joined))
        .case_insensitive(true)
        .build()
        .ok();
    *ALLELUIA_REGEX.lock().unwrap() = alleluia_re;

    // Compute the omit regexp.
    // For each language, load the Comment.txt file and split the "Preces" and "Suffragium" keys.
    let mut omits_vec = Vec::new();
    for lang in &langs {
        let comm = ctx.setupstring(lang, "Psalterium/Comment.txt", res).unwrap_or_default();
        if let Some(preces_val) = comm.get("Preces") {
            let lines: Vec<&str> = preces_val.split('\n').collect();
            if lines.len() > 1 {
                omits_vec.push(lines[1].to_string());
            }
        }
        if let Some(suff_val) = comm.get("Suffragium") {
            let lines: Vec<&str> = suff_val.split('\n').collect();
            if !lines.is_empty() {
                omits_vec.push(lines[0].to_string());
            }
        }
    }
    let omits = omits_vec.join("|");
    let omit_pattern = format!(r"\b(?:{})\b", omits);
    let omit_re = Regex::new(&omit_pattern).ok();
    *OMIT_REGEX.lock().unwrap() = omit_re;
}
