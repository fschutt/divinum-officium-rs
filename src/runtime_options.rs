//! runtime_options.rs
//!
//! This module provides runtime option checks for the Divinum Officium project,
//! analogous to the original RunTimeOptions.pm. It exports three functions:
//! 
//! - `check_version(v: &str, missa: bool) -> Option<String>`  
//! - `check_horas(h: &str) -> Vec<Option<String>>`  
//! - `check_language(l: &str) -> Option<String>`  
//!
//! The module uses a private helper function, `unequivocal`, to look up values
//! in a dialog table (retrieved via `crate::main::get_dialog`) and then strip any
//! path components (i.e. return only the file name portion).
//!
//! Legacy version mappings are defined as lazy–static hash maps.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Returns a string with any leading path (up to and including the last '/') removed.
fn strip_path(s: &str) -> String {
    if let Some(pos) = s.rfind('/') {
        s[pos + 1..].to_string()
    } else {
        s.to_string()
    }
}

/// Private helper that looks up a given value in a dialog table and returns, if uniquely found,
/// the value with any preceding directory removed. It mimics the Perl `unequivocal` sub.
fn unequivocal(value: &str, tablename: &str) -> Option<String> {
    // Assume that get_dialog returns a Vec<String> (this function must be defined elsewhere).
    let values_array = crate::dialogcommon::get_dialog(tablename);
    // First, look for items that match `value` as a regex.
    let re = Regex::new(value).ok()?;
    let mut matches: Vec<&String> = values_array.iter().filter(|s| re.is_match(s)).collect();
    if matches.len() == 1 {
        return Some(strip_path(matches[0]));
    } else {
        // Otherwise, look for items equal to value.
        matches = values_array.iter().filter(|s| *s == value).collect();
        if matches.len() == 1 {
            return Some(strip_path(matches[0]));
        }
    }
    None
}

/// Legacy version names for non-missa usage.
static LEGACY_VERSION_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("Tridentine 1570", "Tridentine - 1570");
    m.insert("Tridentine 1910", "Tridentine - 1906");
    m.insert("Rubrics 1960", "Rubrics 1960 - 1960");
    m.insert("Reduced 1955", "Reduced - 1955");
    m.insert("Monastic", "Monastic - 1963");
    m.insert("1960 Newcalendar", "Rubrics 1960 - 2020 USA");
    m.insert("Dominican", "Ordo Praedicatorum - 1962");
    // safeguard switch from missa to horas
    m.insert("Tridentine - 1910", "Tridentine - 1906");
    m.insert("Ordo Praedicatorum Dominican 1962", "Ordo Praedicatorum - 1962");
    m.insert("Rubrics 1960 Newcalendar", "Rubrics 1960 - 2020 USA");
    m
});

/// Legacy version names for missa usage.
static LEGACY_MISSA_VERSION_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("Tridentine 1570", "Tridentine - 1570");
    m.insert("Tridentine 1910", "Tridentine - 1910");
    m.insert("Rubrics 1960", "Rubrics 1960 - 1960");
    m.insert("Reduced 1955", "Reduced - 1955");
    m.insert("1960 Newcalendar", "Rubrics 1960 - 2020 USA");
    m.insert("Dominican", "Ordo Praedicatorum Dominican 1962");
    // safeguard switch from horas to missa
    m.insert("Monastic Tridentinum 1617", "Tridentine - 1570");
    m.insert("Monastic Divino 1930", "Divino Afflatu - 1954");
    m.insert("Monastic - 1963", "Rubrics 1960 - 1960");
    m.insert("Monastic Tridentinum Cisterciensis 1951", "Tridentine - 1910");
    m.insert("Monastic Tridentinum Cisterciensis Altovadensis", "Reduced - 1955");
    m.insert("Tridentine - 1888", "Tridentine - 1910");
    m.insert("Tridentine - 1906", "Tridentine - 1910");
    m.insert("Ordo Praedicatorum - 1962", "Ordo Praedicatorum Dominican 1962");
    m
});

/// Checks the version string and returns its canonical (legacy) form if available.
/// If not, returns the result of calling `unequivocal($v, "versions")`.
pub fn check_version(v: &str, missa: bool) -> Option<String> {
    if v.is_empty() {
        return None;
    }
    if !missa {
        if let Some(mapped) = LEGACY_VERSION_NAMES.get(v) {
            Some(mapped.to_string())
        } else {
            unequivocal(v, "versions")
        }
    } else {
        if let Some(mapped) = LEGACY_MISSA_VERSION_NAMES.get(v) {
            Some(mapped.to_string())
        } else {
            unequivocal(v, "versions")
        }
    }
}

/// Splits the given string `h` at positions where a capital letter followed by lowercase letters starts,
/// and for each piece returns the result of calling `unequivocal(piece, "horas")`.
pub fn check_horas(h: &str) -> Vec<Option<String>> {
    // Split using a positive lookahead for an uppercase letter followed by lowercase letters.
    let re = Regex::new(r"(?=\p{Lu}\p{Ll}*)").unwrap();
    re.split(h)
        .filter(|s| !s.is_empty())
        .map(|s| unequivocal(s, "horas"))
        .collect()
}

/// Checks a language name by calling `unequivocal` with the 'languages' table.
pub fn check_language(l: &str) -> Option<String> {
    unequivocal(l, "languages")
}
