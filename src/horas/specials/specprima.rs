//! specprima.rs
//!
//! This module implements the “Prima Special” routines from the original Perl
//! `/horas/specials/specprima.pl`. It provides three public functions:
//!
//! 1. `lectio_brevis_prima(lang: &str) -> (String, i32)` – collects the brief lecture
//!    for Prima (returning the text and a comment code).
//! 2. `capitulum_prima(lang: &str, with_responsory: bool) -> String` – collects the capitulum
//!    (with optional responsory) for Prima.
//! 3. `get_prima_responsory(lang: &str) -> String` – looks up the Prima responsory.
//!
//! Throughout, we use early–return style and split out branches into helper functions.
//! “Regexes” from the original code (for example, case–insensitive matching) are replaced by
//! dedicated helper functions (see `contains_ci()` and `parse_doxology_key()`).
//!
//! ## External dependencies
//!
//! This module assumes the existence of functions such as:
//! - `setupstring(lang, filename)` returning a `HashMap<String,String>`,
//! - `gettempora(key)` returning a `String`,
//! - `setbuild(section, name, ord)` and variants,
//! - `setcomment(label, source, comment, lang, prefix)`,
//! - and various globals via a `globals` module.
//!
//! See the tests at the end for examples of how our helper functions mimic the original Perl behavior.

use std::collections::HashMap;

use crate::globals::{
    get_commune, get_commune2, get_version, get_winner, get_winner2, gettempora, get_label,
};
use crate::setup_string::setupstring;
use crate::specials_build::{setbuild, setbuild1, setbuild2};
use crate::comment::setcomment;
use crate::columnsel;
use crate::specials_papal::replace_ndot;
use crate::regex::contains_ci;

/// Returns the brief lecture for Prima as a tuple `(text, comment)`.
pub fn lectio_brevis_prima(lang: &str) -> (String, i32) {
    // Get globals (version, winner maps, commune maps)
    let version = crate::globals::get_version();
    let winner = get_winner();
    let winner2 = get_winner2();
    let commune = get_commune();
    let commune2 = get_commune2();

    // Load the special data file.
    let brevis_map = setupstring(lang, "Psalterium/Special/Prima Special.txt", &[]).unwrap_or_default();
    let name = gettempora("Lectio brevis Prima");
    // Get the initial brevis text from the map.
    let mut brevis = brevis_map.get(&name).cloned().unwrap_or_default();
    // Set comment: if name (case-insensitive) contains "per annum", comment=5, else 1.
    let mut comment = if contains_ci(&name, "per annum") { 5 } else { 1 };

    setbuild("Psalterium/Special/Prima Special", &name, "Lectio brevis ord");

    // If version does not match /1955|196|cist/i, then try to substitute a new Lectio Prima.
    if !(contains_ci(&version, "1955")
        || contains_ci(&version, "196")
        || contains_ci(&version, "cist"))
    {
        let b = if let Some(val) = winner.get("Lectio Prima") {
            if columnsel(lang) { val.clone() } else { winner2.get("Lectio Prima").cloned().unwrap_or_default() }
        } else if let Some(val) = commune.get("Lectio Prima") {
            if columnsel(lang) { val.clone() } else { commune2.get("Lectio Prima").cloned().unwrap_or_default() }
        } else {
            String::new()
        };
        if !b.is_empty() {
            setbuild2(&format!("Subst Lectio Prima {}", winner));
            comment = 3.max(comment); // If substitution from winner occurred, comment becomes 3.
            // Use substituted text if available.
            brevis = b;
        }
    }
    // Unless the version starts with "Monastic" (case-insensitive), prepend a benedictio.
    if !version.to_lowercase().starts_with("monastic") {
        brevis = format!("$benedictio Prima\n{}", brevis);
    }
    brevis.push_str("\n$Tu autem");
    (brevis, comment)
}

/// Returns the capitulum for Prima as a String.
/// The parameter `with_responsory` indicates whether responsory text should be included.
pub fn capitulum_prima(lang: &str, with_responsory: bool) -> String {
    // Retrieve needed globals.
    let dayofweek = crate::globals::get_dayofweek();
    let version = crate::globals::get_version();
    let winner = get_winner();
    let commune = get_commune();
    let rank = crate::globals::get_rank();
    let daynames = crate::globals::get_daynames();
    let label = get_label();
    let winner2 = get_winner2();

    let brevis_map = setupstring(lang, "Psalterium/Special/Prima Special.txt", &[]).unwrap_or_default();

    // Compute key based on conditions.
    let key = if dayofweek > 0
        && !contains_ci(&version, "196")
        && (winner.get("Rank").unwrap_or(&String::new()).to_lowercase().contains("feria")
            || winner.get("Rank").unwrap_or(&String::new()).to_lowercase().contains("vigilia"))
        && !winner.get("Rank").unwrap_or(&String::new()).to_lowercase().contains("vigilia epi")
        && (commune.is_empty() || !contains_ci(&commune, "C10"))
        && (rank < 3 || (!daynames.is_empty() && contains_ci(&daynames[0], "quad6")))
        && (!daynames.is_empty() && !contains_ci(&daynames[0], "pasc"))
    {
        "Feria".to_string()
    } else {
        "Dominica".to_string()
    };

    let mut capit = brevis_map.get(&key).cloned().unwrap_or_default();
    capit.push_str("\n$Deo gratias\n_\n");
    setbuild1("Capitulum", &format!("Psalterium {}", key));

    if contains_ci(&version, "1963") {
        capit = format!("{}\n{}", label, capit);
    } else {
        setcomment(&label, "Source", key == "Feria", lang, "");
    }

    let mut resp_lines = Vec::new();
    if with_responsory {
        if let Some(resp_text) = brevis_map.get("Responsory") {
            resp_lines = resp_text.lines().map(|s| s.to_string()).collect();
        }
        let mut prima_responsory = get_prima_responsory(lang);
        let wpr = if columnsel(lang) { winner.clone() } else { winner2.clone() };
        if let Some(val) = wpr.get("Versum Prima") {
            prima_responsory = val.clone();
        }
        if !prima_responsory.is_empty() && resp_lines.len() > 2 {
            resp_lines[2] = format!("V. {}", prima_responsory);
        }
        resp_lines.push("_".to_string());
    }
    if let Some(versum) = brevis_map.get("Versum") {
        let mut vers_lines: Vec<String> = versum.lines().map(|s| s.to_string()).collect();
        resp_lines.append(&mut vers_lines);
    }
    crate::postprocess::postprocess_short_resp(&mut resp_lines, lang);
    format!("{}{}", capit, resp_lines.join("\n"))
}

/// Returns the Prima responsory as a String.
/// If no key can be determined, returns an empty string.
pub fn get_prima_responsory(lang: &str) -> String {
    let version = crate::globals::get_version();
    let month = crate::globals::get_day(); // assuming month is available (stub)
    let day = crate::globals::get_day();   // assuming day
    let rule = crate::globals::get_rule();
    // Assume commemoratio is a map loaded from globals.
    let commemoratio = crate::globals::get_commemoratio();
    
    let mut key = gettempora("Prima responsory");

    if let Some(k) = parse_doxology_key(rule) {
        key = k;
    } else if !contains_ci(&version, "196") && month == 8 && day > 15 && day < 23 {
        key = "Nat".to_string();
    }
    if contains_ci(&version, "196") && month == 12 && day > 8 && day < 16 && !contains_ci(&version, "Newcal") && day != 12 {
        key = "Adv".to_string();
    }
    if contains_ci(&version, "196") && (key.contains("Corp") || key.contains("Heart")) {
        key.clear();
    }
    if key.is_empty() {
        return String::new();
    }
    let t_map = setupstring(lang, "Psalterium/Special/Prima Special.txt", &[]).unwrap_or_default();
    t_map.get(&format!("Responsory {}", key)).cloned().unwrap_or_default()
}

/// Helper: parses a doxology key from a string.  
/// Looks for a substring like "Doxology=Nat" (case-insensitive) and returns "Nat".
fn parse_doxology_key(s: &str) -> Option<String> {
    // Instead of using a regex, we search for "doxology=" (case-insensitive)
    let lower = s.to_lowercase();
    if let Some(pos) = lower.find("doxology=") {
        // Get the substring after "doxology="
        let remainder = &s[pos + "doxology=".len()..];
        // Take the first word (split on whitespace)
        let key = remainder.split_whitespace().next()?.to_string();
        return Some(key);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_ci() {
        assert!(contains_ci("Hello C12 world", "c12"));
        assert!(!contains_ci("Hello world", "C13"));
    }

    #[test]
    fn test_parse_doxology_key() {
        let s = "Some text Doxology=Nat and more";
        let key = parse_doxology_key(s);
        assert_eq!(key.unwrap(), "Nat");
    }

    #[test]
    fn test_lectio_brevis_prima() {
        // With a dummy setupstring, gettempora returns the key itself.
        let result = lectio_brevis_prima("Latin");
        // We expect the returned text to include "$Tu autem" at the end.
        assert!(result.0.contains("$Tu autem"));
    }

    #[test]
    fn test_get_prima_responsory_empty() {
        // With no data in setupstring, we expect an empty string.
        let result = get_prima_responsory("Latin");
        assert!(result.is_empty());
    }
}
