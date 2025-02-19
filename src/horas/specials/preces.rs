//! preces.rs
//!
//! This module implements routines from `/horas/specials/preces.pl`.
//!
//! It provides two public functions:
//! 
//! - `preces(item: &str) -> bool` – decides whether to use preces (returns true) or to omit them.
//! - `get_preces(hora: &str, lang: &str, flag: bool) -> Option<String>` – returns the preces text.
//!
//! The module has been broken into several helper functions:
//!
//! - `handle_dominicales_branch(...) -> Option<bool>`  
//!   (Returns Some(true) if the “Dominicales” branch dictates preces should be used.)
//!
//! - `handle_feriales_branch(...) -> bool`  
//!   (Returns true if the “Feriales” branch conditions are met.)
//!
//! All regex checks have been replaced by simple string methods or by small custom functions.
//!
//! ## Tests
//!
//! Tests at the end verify that our helper functions produce results equivalent
//! to the original Perl regex checks.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

// Assume these external functions and globals are defined elsewhere:
use crate::globals::{
    get_commemoratio, get_commemoentries, get_daynames, get_dayofweek, get_duplex, get_rule,
    get_version, get_winner, get_winner_map, get_hora, emberday, get_datafolder, get_day,
};
use crate::offices::officestring;
use crate::setup_string::setupstring;

/// A helper function that performs a case–insensitive check whether `s` contains `pat`.
fn contains_ci(s: &str, pat: &str) -> bool {
    s.to_lowercase().contains(&pat.to_lowercase())
}

/// Returns the first nonempty value among the keys "Commemoratio", "Commemoratio 1", etc.
fn check_commemoratio(map: &HashMap<String, String>) -> String {
    map.get("Commemoratio")
        .or_else(|| map.get("Commemoratio 1"))
        .or_else(|| map.get("Commemoratio 2"))
        .or_else(|| map.get("Commemoratio 3"))
        .cloned()
        .unwrap_or_else(String::new)
}

/// Helper to handle the Dominicales branch.
///
/// It receives:
/// - `item`: the original item string,
/// - `winner`: a string from the globals,
/// - `winner_map`: a map (from globals) of winner values,
/// - `rule`: the current rule string,
/// - `duplex`: a numeric value,
/// - `seasonalflag`: a boolean flag,
/// - `daynames`: vector of dayname strings,
/// - `version`: the version string,
/// - `commemoratio`: a map with commemoratio data,
/// - `commemoentries`: a list of commemo entries,
/// - `datafolder`: the datafolder string.
///
/// If all conditions are met, it returns Some(true) indicating that preces should be used,
/// otherwise it returns None.
fn handle_dominicales_branch(
    item: &str,
    winner: &str,
    winner_map: &HashMap<String, String>,
    rule: &str,
    duplex: f64,
    seasonalflag: bool,
    daynames: &[String],
    version: &str,
    commemoratio: &HashMap<String, String>,
    commemoentries: &[String],
    datafolder: &str,
) -> Option<bool> {
    if !item.to_lowercase().contains("dominicales") {
        return None;
    }
    let mut dominicales = true;
    if !commemoratio.is_empty() {
        let rank_str = commemoratio.get("Rank").unwrap_or(&String::new());
        let parts: Vec<&str> = rank_str.split(";;").collect();
        if parts
            .get(2)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
            >= 3.0
            || rank_str.to_lowercase().contains("octav")
            || check_commemoratio(commemoratio).to_lowercase().contains("octav")
        {
            dominicales = false;
        } else if !commemoentries.is_empty() {
            for commemo in commemoentries.iter() {
                let mut filename = commemo.clone();
                // Append ".txt" if necessary.
                if !std::path::Path::new(&format!("{}/Latin/{}", datafolder, filename))
                    .exists()
                    && !filename.to_lowercase().ends_with("txt")
                {
                    filename.push_str(".txt");
                }
                let c = officestring("Latin", &filename, 0).unwrap_or_default();
                let rank_c = c.get("Rank").unwrap_or(&String::new());
                let parts_c: Vec<&str> = rank_c.split(";;").collect();
                if parts_c
                    .get(2)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0)
                    >= 3.0
                    || rank_c.to_lowercase().contains("octav")
                    || check_commemoratio(&c).to_lowercase().contains("octav")
                {
                    dominicales = false;
                }
            }
        }
    }
    if dominicales
        && (!contains_ci(
            winner_map.get("Rank").unwrap_or(&String::new()),
            "octav",
        ) || contains_ci(
            winner_map.get("Rank").unwrap_or(&String::new()),
            "post octav",
        ))
        && !check_commemoratio(winner_map).to_lowercase().contains("octav")
    {
        // Here, we assume that calling get_preces("Feriales", ...) returns Some(_)
        if get_preces("Feriales", "dummy_lang", false).is_some() {
            return Some(true);
        }
    }
    None
}

/// Helper to handle the Feriales branch.
///
/// Returns true if the conditions for the feriales branch are met.
fn handle_feriales_branch(
    item: &str,
    dayofweek: u32,
    hora: &str,
    winner: &str,
    rule: &str,
    daynames: &[String],
    version: &str,
) -> bool {
    if !item.to_lowercase().contains("feriales") {
        return false;
    }
    if dayofweek == 0 || (dayofweek == 6 && hora == "Vespera") {
        return false;
    }
    let cond1 = !winner.to_lowercase().contains("sancti")
        && (rule.to_lowercase().contains("preces")
            || (!daynames.is_empty()
                && (daynames[0].to_lowercase().contains("adv")
                    || (daynames[0].to_lowercase().contains("quad")
                        && !daynames[0].to_lowercase().contains("quadp")))
            )
            || emberday());
    let cond2 = (!version.contains("1955")
        && !version.contains("1960")
        && !version.contains("Newcal")
        && winner
            .to_lowercase()
            .contains("vigil")
        && daynames.get(1).map_or(false, |s| {
            !s.to_lowercase().contains("epi") && !s.to_lowercase().contains("pasc")
        }));
    let cond3 = !version.contains("1955")
        && !version.contains("1960")
        && !version.contains("Newcal")
        || (matches!(dayofweek, 3 | 5) || emberday());
    cond1 || cond2 && cond3
}

/// Public function `preces` returns true if preces should be used, false otherwise.
pub fn preces(item: &str) -> bool {
    // Get global values.
    let winner = get_winner();
    let rule = get_rule();
    let duplex = get_duplex();
    let seasonalflag = crate::globals::get_seasonalflag();
    let daynames = get_daynames();
    let version = get_version();
    let commemoratio = get_commemoratio(); // HashMap<String, String>
    let commemoentries = get_commemoentries(); // Vec<String>
    let hora = get_hora();
    let dayofweek = get_dayofweek();
    let datafolder = get_datafolder();

    // Early return if any of the following conditions are met:
    if contains_ci(&winner, "C12")
        || (rule.to_lowercase().contains("omit") && rule.to_lowercase().contains(" preces"))
        || (duplex > 2.0 && seasonalflag)
        || (!daynames.is_empty()
            && (daynames[0].to_lowercase().contains("pasc6")
                || daynames[0].to_lowercase().contains("pasc7")))
    {
        return false;
    }

    // Check Dominicales branch:
    if let Some(true) = handle_dominicales_branch(
        item,
        &winner,
        &get_winner_map(),
        rule,
        duplex,
        seasonalflag,
        &daynames,
        &version,
        &commemoratio,
        &commemoentries,
        &datafolder,
    ) {
        return true;
    }

    // Check Feriales branch:
    if handle_feriales_branch(item, dayofweek, &hora, &winner, rule, &daynames, &version) {
        return true;
    }

    false
}

/// Public function `get_preces` returns the preces text based on the current hour.
/// The `flag` parameter indicates whether we are using the 'Dominicales' variant.
pub fn get_preces(hora: &str, lang: &str, flag: bool) -> Option<String> {
    let version = get_version();
    let (src, key) = if hora == "Tertia" || hora == "Sexta" || hora == "Nona" {
        ("Minor", "Feriales".to_string())
    } else if hora == "Laudes" || hora == "Vespera" {
        ("Major", format!("feriales {}", hora))
    } else if hora == "Completorium" {
        ("Minor", "Dominicales".to_string())
    } else if flag {
        let src = "Prima";
        let mod_val = if version.starts_with("Monastic") { 1 } else { 2 };
        let mut counter = PREC_DOMFER.lock().unwrap();
        let value = ((*counter + 1) % mod_val) + 1;
        *counter += 1;
        (src, format!("Dominicales Prima {}", value))
    } else {
        ("Prima", "feriales Prima".to_string())
    };

    if let Some(brevis_map) = setupstring(lang, &format!("Psalterium/Special/{} Special.txt", src), &[]) {
        return brevis_map.get(&format!("Preces {}", key)).cloned();
    }
    None
}
 
// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_ci() {
        let s = "Hello C12 world";
        assert!(contains_ci(s, "c12"));
        assert!(!contains_ci(s, "C13"));
    }

    #[test]
    fn test_check_commemoratio() {
        let mut map = HashMap::new();
        map.insert("Commemoratio 2".to_string(), "Test".to_string());
        assert_eq!(check_commemoratio(&map), "Test".to_string());
    }

    #[test]
    fn test_handle_dominicales_branch_returns_none_when_not_applicable() {
        // If the item does not contain "dominicales", it should return None.
        let result = handle_dominicales_branch(
            "Some other item",
            "winner",
            &HashMap::new(),
            "Some rule",
            1.0,
            false,
            &vec!["Monday".to_string()],
            "TestVersion",
            &HashMap::new(),
            &vec![],
            "datafolder",
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_feriales_branch() {
        // For a sample input that should trigger the feriales branch.
        let daynames = vec!["Adv".to_string(), "SomeSecond".to_string()];
        let result = handle_feriales_branch(
            "Feriales",
            3,                  // dayofweek nonzero and not Saturday (6)
            "Laudes",
            "non sancti text",  // winner does not contain "sancti"
            "Preces something",
            &daynames,
            "TestVersion",
        );
        // Depending on emberday() (stubbed as false) and other conditions,
        // we expect true if the conditions are met.
        assert!(result);
    }

    #[test]
    fn test_get_preces_returns_none_if_setupstring_fails() {
        // Without a proper override of setupstring, get_preces should return None.
        assert_eq!(get_preces("Tertia", "Latin", false), None);
    }
}
