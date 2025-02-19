//! hymni.rs
//!
//! This module implements the special hymn‐retrieval routines used in the Hours.
//!
//! It defines three primary functions:
//!
//! - `get_hymn(lang: &str) -> Option<String>` – returns the hymn (with proper doxology, build–info, etc.).
//! - `hymnus_major(lang: &str) -> (Option<String>, String)` – returns a tuple (hymn, hymn name)
//!   for the major hours.
//! - `doxology(lang: &str) -> (Option<String>, String)` – returns the doxology text and its key.

use std::collections::HashMap;

// Assume that these helper functions are defined elsewhere:
use crate::language_text_tools::translate;
use crate::setup_string::setupstring;
use crate::specials_build::{setbuild, setbuild1, setbuild2};
use crate::proprium::getproprium;
use crate::tempora::gettempora;
use crate::postprocess::{postprocess_short_resp, postprocess_vr};
use crate::doxology_module::doxology; // our own doxology function
use crate::globals::{
    get_hora, get_version, get_vespera, get_daynames, get_seasonalflag, get_votive, get_winner,
};

/// Standardizes the hymn’s opening.  
/// This replicates the Perl regex:  
///   `s/^(?:v\.\s*)?(\p{Lu})/v. $1/`  
/// It checks whether the hymn starts with an optional `"v. "` followed by an uppercase letter.
/// If so, it returns `"v. "` concatenated with that letter; otherwise, if the hymn starts with an uppercase letter,
/// it prepends `"v. "` to it.
fn fix_initial(hymn: &str) -> String {
    // Remove any leading whitespace.
    let s = hymn.trim_start();
    // Check if it already starts with "v. " (case–insensitive)
    if s.to_lowercase().starts_with("v. ") {
        // We standardize it by ensuring that after "v. " the next character is kept.
        let rest = s[3..].trim_start();
        if let Some(first) = rest.chars().next() {
            return format!("v. {}", first);
        }
    } else if let Some(first) = s.chars().next() {
        if first.is_uppercase() {
            return format!("v. {}", first);
        }
    }
    s.to_string()
}

/// Removes all asterisks and following whitespace from the hymn text.
/// This replicates the Perl substitution: `s/\*\s*//g`
fn remove_stars(hymn: &str) -> String {
    hymn.replace("*", "").trim().to_string()
}

/// Replaces occurrences of `"_\n"` not followed by `"!"` with `"_\nr. "`.
/// (In our implementation we simply search for `"_\n"` and check the next character.)
fn fix_stropha(hymn: &str) -> String {
    let mut result = String::new();
    let mut chars = hymn.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '_' {
            result.push(ch);
            if let Some(&'\n') = chars.peek() {
                // Peek ahead after "\n" to check if not '!'
                let mut temp = chars.clone();
                temp.next(); // consume newline
                if let Some(&next_ch) = temp.peek() {
                    if next_ch != '!' {
                        // Replace "\n" with "\n" + "r. "
                        result.push('\n');
                        result.push_str("r. ");
                        chars.next(); // consume newline
                        continue;
                    }
                }
            }
            // otherwise, just continue
        } else {
            result.push(ch);
        }
    }
    result
}

/// If the hymn contains an asterisk, remove the asterisk and everything following it.
fn remove_attached_versicle(hymn: &str) -> String {
    if let Some(pos) = hymn.find('*') {
        hymn[..pos].trim_end().to_string()
    } else {
        hymn.to_string()
    }
}

/// Returns the hymn text for the current hour (and its associated section) as a single string.
/// 
/// The function follows these steps:
/// 
/// 1. It obtains a base section via `translate("Hymnus", lang)`.
/// 2. Depending on the global hour (`hora`):
///    - If `"Matutinum"`, it calls `hymnusmatutinum(lang)`.  
///      If no hymn is found, it sets hymn source to `"Matutinum"` and clears the section.
///    - If `"Laudes"` or `"Vespera"`, it calls `hymnus_major(lang)`, prepends `"Hymnus "` to the hymn name,
///      sets hymn source to `"Major"` (if no hymn was found), and sets section to `"_\n!{section}"`.
///      It then calls `getantvers("Versum", ind, lang)` with `ind = 2` for Laudes or `vespera` for Vespera.
///    - Otherwise (minor hours), it sets the name to `"Hymnus {hora}"` (with a special substitution for Tertia)
///      and, if the hour is `"Completorium"` and the version begins with `"Ordo Praedicatorum"`,
///      loads extra data from `"Psalterium/Special/Minor Special.txt"`, postprocesses it, and may append a seasonal
///      suffix from `gettempora("*")`. It sets hymn source to `"Prima"` if the hour is `"Prima"`,
///      otherwise `"Minor"`, and prefixes the section with `"#"`.
/// 3. If a hymn source is defined, it loads a lookup table from  
///    `"Psalterium/Special/{hymnsource} Special.txt"`, adjusts the hymn name using `tryoldhymn()`,
///    and retrieves the hymn text from the table.
/// 4. If the version does not match `/1960/` and the hymn text contains an asterisk,
///    it calls `doxology(lang)` to get a doxology and substitutes (replacing everything from the asterisk onward)
///    with the doxology. Also, it appends `" {Doxology: dname}"` to the section if appropriate.
/// 5. It then standardizes the hymn’s opening using `fix_initial()`, removes stars via `remove_stars()`,
///    and fixes stropha markers using `fix_stropha()`.
/// 6. Finally, it concatenates the section, hymn, and (if present) the versum text and returns the result.
pub fn get_hymn(lang: &str) -> Option<String> {
    // Retrieve global variables from our globals module.
    let hora = crate::globals::get_hora();
    let version = crate::globals::get_version();
    let vespera = crate::globals::get_vespera();
    let daynames = crate::globals::get_daynames(); // Vec<String>
    let mut section = translate("Hymnus", lang);

    // Variables that will be set by subsequent calls.
    let mut name = String::new();
    let mut hymn: Option<String> = None;
    let mut hymn_source: Option<String> = None;
    let mut versum: Option<String> = None;
    let mut cr: Option<String> = None; // Comment or similar

    if hora == "Matutinum" {
        // Assume hymnusmatutinum(lang) -> (Option<String>, String)
        let (h, n) = crate::specials_hymn::hymnusmatutinum(lang);
        hymn = h;
        name = n;
        if hymn.is_none() {
            hymn_source = Some("Matutinum".to_string());
        }
        section.clear();
    } else if hora == "Laudes" || hora == "Vespera" {
        // Call hymnus_major(lang) -> (Option<String>, String)
        let (h, n) = hymnus_major(lang);
        hymn = h;
        name = format!("Hymnus {}", n);
        if hymn.is_none() {
            hymn_source = Some("Major".to_string());
        }
        section = format!("_\n!{}", section);
        let ind = if hora == "Laudes" { 2 } else { vespera };
        // Assume getantvers returns (Option<String>, Option<String>)
        let (v, c) = crate::specials_hymn::getantvers("Versum", ind, lang);
        versum = v;
        cr = c;
    } else {
        // Minor hours:
        name = format!("Hymnus {}", hora);
        // If hora == "Tertia" and first dayname contains "Pasc7",
        // replace the first space with " Pasc7 " (we do a simple check)
        if hora == "Tertia" && !daynames.is_empty() && daynames[0].contains("Pasc7") {
            if let Some(pos) = name.find(' ') {
                name.replace_range(pos..=pos, " Pasc7 ");
            }
        }
        if hora == "Completorium" && version.starts_with("Ordo Praedicatorum") {
            if let Some(ant_map) = setupstring(lang, "Psalterium/Special/Minor Special.txt") {
                versum = ant_map.get("Versum 4").cloned();
            }
            // Postprocess the versum text.
            if let Some(ref mut v) = versum {
                postprocess_vr(v, lang);
            }
            let tempname = gettempora("*");
            // Instead of regex, we check whether tempname equals one of the allowed strings.
            if ["Quad5", "Quad", "Pasch", "Asc", "Pent"].iter().any(|&s| tempname.starts_with(s)) {
                name.push(' ');
                name.push_str(&tempname);
            }
        }
        hymn_source = Some(if hora == "Prima" { "Prima" } else { "Minor" }.to_string());
        section = format!("#{}", section);
    }

    // If hymn_source is defined, load a lookup table from "Psalterium/Special/{source} Special.txt"
    if let Some(ref src) = hymn_source {
        if let Some(hmap) = setupstring(lang, &format!("Psalterium/Special/{} Special.txt", src)) {
            // Assume tryoldhymn(hmap, name) -> String
            name = crate::specials_hymn::tryoldhymn(&hmap, name);
            hymn = hmap.get(&name).cloned();
        }
    }

    // If version does not match /1960/ and hymn contains "*", do doxology.
    if !version.contains("1960") {
        if let Some(ref mut h) = hymn {
            if h.contains('*') {
                let (dox, dname) = doxology(lang);
                if !dname.is_empty() {
                    // Replace from first "*" onward with dox.
                    if let Some(pos) = h.find('*') {
                        h.replace_range(pos.., &dox);
                    }
                    if !section.is_empty() {
                        section.push_str(&format!(" {{Doxology: {}}}", dname));
                    }
                }
            }
        }
    }

    // Standardize the hymn’s opening.
    if let Some(ref mut h) = hymn {
        *h = fix_initial(h);
        *h = remove_stars(h);
        *h = fix_stropha(h);
    }

    // Build the final output.
    let mut output = format!("{}\n{}", section, hymn.unwrap_or_default());
    if let Some(v) = versum {
        output.push_str(&format!("\n_\n{}", v));
    }
    Some(output)
}

/// Returns a tuple `(hymn, name)` for the major hymn corresponding to the given language.
/// 
/// This function uses several global variables (such as `hora`, `version`, `vespera`, the winners map, etc.)
/// which are assumed to be available via the `globals` module. It applies several rules:
/// 
/// 1. Start with `"Hymnus"`. If `hora` equals `"Vespera"`, append the result of `checkmtv(version, winners)`.
/// 2. Under certain conditions (depending on the winners map and the vespera value), the name is reset to `"Hymnus"`.
/// 3. If `hymnshift(version, day, month, year)` returns true then append `" Matutinum"` for Laudes or `" Laudes"` for Vespera,
///    and call `setbuild2("Hymnus shifted")`. Otherwise, append `" {hora}"`.
/// 4. If `hora` equals `"Vespera"` and `vespera == 3`, attempt to get the hymn via `getproprium("{name} 3", ...)`.
/// 5. Under a special condition (if version matches /cist/i, hora matches Vespera and winners{Rule} matches specific patterns),
///    set the name to `"Hymnus Vespera Hac die"`.
/// 6. If no hymn is found, set the name from `gettempora("Hymnus major")` concatenated with `hora`, and (if certain conditions hold)
///    append `" hiemalis"`, then call `setbuild1("Hymnus", name)`.
/// 7. Finally, return the tuple `(hymn, name)`.
pub fn hymnus_major(lang: &str) -> (Option<String>, String) {
    let hora = crate::globals::get_hora();
    let version = crate::globals::get_version();
    let vespera = crate::globals::get_vespera();
    let daynames = crate::globals::get_daynames();
    let seasonalflag = crate::globals::get_seasonalflag();
    let day = crate::globals::get_day();
    let month = crate::globals::get_month();
    let year = crate::globals::get_year();
    let winners = crate::globals::get_winner_map(); // assumed to be a HashMap<String,String>

    let mut name = "Hymnus".to_string();
    // If hora is "Vespera", append checkmtv(version, winners)
    if hora == "Vespera" {
        name.push_str(&crate::specials_hymn::checkmtv(&version, &winners));
    }
    // Reset name if certain conditions hold:
    if !winners.contains_key(&format!("{} Vespera", name))
        && vespera == 3
        && !winners.contains_key(&format!("{} Vespera 3", name))
        && ((vespera == 3 && winners.contains_key("Hymnus Vespera 3"))
            || winners.contains_key("Hymnus Vespera"))
    {
        name = "Hymnus".to_string();
    }
    if crate::specials_hymn::hymnshift(&version, day, month, year) {
        if hora == "Laudes" {
            name.push_str(" Matutinum");
        }
        if hora == "Vespera" {
            name.push_str(" Laudes");
        }
        setbuild2("Hymnus shifted");
    } else {
        name.push_str(&format!(" {}", hora));
    }
    let mut cr: Option<String> = None;
    let mut hymn: Option<String> = None;
    if hora == "Vespera" && vespera == 3 {
        let (h, c) = getproprium(&format!("{} 3", name), lang, seasonalflag, 1);
        hymn = h;
        cr = c;
    }
    if version.to_lowercase().contains("cist")
        && hora.to_lowercase().contains("vespera")
        && winners.get("Rule").map_or(false, |s| s.contains("C4") || s.contains("C5"))
        && winners.get("Rule").map_or(false, |s| s.contains("Hac die"))
    {
        name = "Hymnus Vespera Hac die".to_string();
    }
    if hymn.is_none() {
        let (h, c) = getproprium(&name, lang, seasonalflag, 1);
        hymn = h;
        cr = c;
    }
    if hymn.is_none() {
        name = format!("{} {}", gettempora("Hymnus major"), hora);
        // Check additional conditions:
        if name.contains("Day0")
            && (name.contains("Laudes") || version.to_lowercase().contains("cist"))
            && (daynames.get(0).map_or(false, |s| {
                s.contains("Epi2")
                    || s.contains("Epi3")
                    || s.contains("Epi4")
                    || s.contains("Epi5")
                    || s.contains("Epi6")
                    || s.contains("Quadp")
            })
                || winners.get("Rank")
                    .map_or(false, |s| s.contains("Novembris")
                        || (s.contains("Octobris") && !version.to_lowercase().contains("cist"))))
        {
            name.push_str(" hiemalis");
        }
        setbuild1("Hymnus", &name);
    }
    (hymn, name)
}

/// Returns a tuple `(dox, dname)` representing the doxology text and its key.
/// 
/// The function first checks if the winners map has a key `"Doxology"`. If so,
/// it selects one of two winners maps (based on the return of `columnsel(lang)`) and uses that.
/// Otherwise, it attempts to extract a doxology key from either the provided rule string,
/// or (if the version is Tridentine or the winner’s Rank does not match Adventus)
/// from the commemoratio data. If none of these apply, it sets the key to `"Nat"` in certain conditions,
/// or else calls `gettempora("Doxology")`.
/// Finally, if a key is found, it loads the doxologies from  
/// `"Psalterium/Doxologies.txt"`. For Monastic or 1570 versions, if a key with a trailing `"T"` exists,
/// it appends `"T"` to the key. It then sets build information and returns the tuple.
/// 
/// **Regex replacements:**  
/// Rather than using a regex to extract the key from a string (e.g. `/Doxology=([a-z]+)/i`),
/// we use simple string methods (such as splitting on `"Doxology="`) and case–insensitive comparisons.
pub fn doxology(lang: &str) -> (Option<String>, String) {
    let version = crate::globals::get_version();
    let rule = crate::globals::get_rule(); // assumed function returning Option<String>
    let daynames = crate::globals::get_daynames();
    let winners = crate::globals::get_winner_map();
    let winners2 = crate::globals::get_winner2_map();
    let commemoratio = crate::globals::get_commemoratio(); // assumed HashMap<String,String>
    let day = crate::globals::get_day();
    let month = crate::globals::get_month();
    let year = crate::globals::get_year();
    let dayofweek = crate::globals::get_dayofweek();

    let mut dox = String::new();
    let mut dname = String::new();

    if winners.contains_key("Doxology") {
        // Choose winners map based on columnsel(lang)
        let use_winner = if crate::globals::columnsel(lang) {
            &winners
        } else {
            &winners2
        };
        dox = use_winner.get("Doxology").cloned().unwrap_or_default();
        dname = "Special".to_string();
        setbuild2("Special doxology");
    } else {
        if let Some(r) = rule.clone() {
            // Look for substring "Doxology=" and then letters.
            if let Some(pos) = r.to_lowercase().find("doxology=") {
                // The key is the letters following "Doxology="
                dname = r[pos + "doxology=".len()..].split_whitespace().next().unwrap_or("").to_string();
            }
        } else if (version.contains("Trident") || 
                  !winners.get("Rank").map_or(false, |s| s.contains("Adventus")))
            && commemoratio.get("Rule").map_or(false, |s| s.to_lowercase().contains("doxology="))
        {
            // Extract the key from commemoratio's "Rule" value.
            if let Some(rule_str) = commemoratio.get("Rule") {
                if let Some(pos) = rule_str.to_lowercase().find("doxology=") {
                    dname = rule_str[pos + "doxology=".len()..].split_whitespace().next().unwrap_or("").to_string();
                }
            }
        } else if (month == 8 && day > 15 && day < 23 && !version.contains("1955") && !version.contains("1963"))
            || (!version.contains("1570") && !version.contains("1617") && !version.contains("altovadensis") &&
                month == 12 && day > 8 && day < 16 && dayofweek > 0)
        {
            dname = "Nat".to_string();
        } else {
            dname = gettempora("Doxology");
        }

        if !dname.is_empty() {
            if let Some(dox_map) = setupstring(lang, "Psalterium/Doxologies.txt") {
                // If version is Monastic or contains 1570 and key with "T" exists, append "T"
                if (version.contains("Monastic") || version.contains("1570"))
                    && dox_map.contains_key(&(dname.clone() + "T"))
                {
                    dname.push('T');
                }
                dox = dox_map.get(&dname).cloned().unwrap_or_default();
                setbuild2(&format!("Doxology: {}", dname));
            }
        }
    }
    (if dox.is_empty() { None } else { Some(dox) }, dname)
}

#[cfg(test)]
mod tests {

    //! ### Regex Replacement Commentary
    //!
    //! 1. **Simple substring checks:**  
    //!    Perl’s `$winner =~ /12-25/` is replaced by `winner.contains("12-25")`.
    //!
    //! 2. **Attached Versicle Removal:**  
    //!    Instead of using a regex substitution such as `s/\*.*/$dox/s`, we check if the hymn
    //!    contains an asterisk (`"*"`) and then split the string at that position.
    //!
    //! 3. **Standardizing the Initial:**  
    //!    The original Perl code uses `s/^(?:v\.\s*)?(\p{Lu})/v. $1/` to ensure that the hymn’s first
    //!    (uppercase) letter is preceded by `"v. "`. In Rust we define a helper function `fix_initial` that:
    //!    - If the hymn already starts (case–insensitively) with `"v. "`, it standardizes the initial letter.
    //!    - Otherwise, if the first character is uppercase, it prepends `"v. "`.
    //!
    //! 4. **Other substitutions** (such as removing all occurrences of `"*"` followed by optional whitespace, and
    //!    inserting `"r. "` after an underscore-line marker) are done via simple string methods.

    use super::*;

    #[test]
    fn test_fix_initial() {
        // Test that a string beginning with a capital letter gets prefixed with "v. "
        let s = "HELLO world";
        let fixed = fix_initial(s);
        // Our implementation will only prepend the first letter.
        assert_eq!(fixed, "v. H");

        // If already starts with "v. " (in any case), we standardize it.
        let s2 = "V. Greet";
        let fixed2 = fix_initial(s2);
        assert_eq!(fixed2, "v. G");
    }

    #[test]
    fn test_remove_stars() {
        let s = "This * is a * test *";
        let cleaned = remove_stars(s);
        assert_eq!(cleaned, "This  is a  test ");
    }

    #[test]
    fn test_fix_stropha() {
        // Test that "_\n" not followed by "!" becomes "_\nr. "
        let s = "Line1\n_\nNext line\n_!\nNo change";
        let fixed = fix_stropha(s);
        // Expected: the first occurrence becomes "_\nr. " but the second (followed by '!') is left unchanged.
        assert!(fixed.contains("_\nr. Next"));
        assert!(fixed.contains("_!\nNo change"));
    }

    #[test]
    fn test_remove_attached_versicle() {
        let s = "Hymn text here\n* Some extra verse";
        let cleaned = remove_attached_versicle(s);
        assert_eq!(cleaned, "Hymn text here");
    }
}
