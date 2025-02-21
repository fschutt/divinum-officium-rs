//! This module corresponds to `Main.pm` from Divinum Officium,
//! providing two primary functions:
//!
//! 1. **`vernaculars(basedir)`**: Reads a `Linguae.txt` file from
//!    the given base directory and returns the lines as a list
//!    of available vernacular languages.  
//! 2. **`liturgical_color(input)`**: Given a string describing a
//!    liturgical day or feast (e.g. `"Passionis"`, `"Sancta Mariæ"`,
//!    `"Martyr"`, etc.), returns the recommended color
//!    (`"blue"`, `"red"`, `"black"`, etc.) according to the
//!    matching rule. If no specific rule matches, it defaults to
//!    `"black"`.
//!
//! # Usage
//!
//! ```ignore
//! use divinum_officium::{vernaculars, liturgical_color};
//!
//! fn example() -> std::io::Result<()> {
//!     // Read languages from some data folder
//!     let langs = vernaculars("/path/to/data")?;
//!     for lang in langs {
//!         println!("Language: {}", lang);
//!     }
//!
//!     // Determine color for a day named "Sancta Mariæ"
//!     let color = liturgical_color("Sancta Mariæ");
//!     println!("Liturgical color: {}", color);
//!     Ok(())
//! }
//! ```

use std::io;

pub mod date;
pub mod dialogcommon;
pub mod directorium;
pub mod fileio;
pub mod language_text_tools;
pub mod runtime_options;
pub mod scripting;
pub mod setup_string;
pub mod setup;
pub mod regex;
pub mod missa;
pub mod horas;

/// STARDAYS constant used in epactcycle.
const STARDAYS: [i32; 14] = [1, 31, 60, 90, 119, 149, 178, 208, 237, 267, 296, 326, 355, 385];

/// Month lengths (index 1..12); index 0 is unused.
const MONTH_LENGTH: [i32; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Names of months.
static MONTH_NAMES: [&str; 13] = [
    "",
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Wraps text in a font–styled SPAN. The font description is in "[size][ italic][ bold] color" format.
pub fn setfont(font: &str, text: &str) -> String {
    format!("<SPAN STYLE=\"font:{};\">{}</SPAN>", font, text)
}

/// Returns a list of available vernacular languages for the datafiles
/// rooted at `basedir`. This replicates the behavior of
/// `vernaculars($basedir)` in the Perl code, which reads
/// `Linguae.txt` and returns each line.
///
/// # Arguments
///
/// * `basedir` - Path to the directory containing `Linguae.txt`.
///
/// # Returns
///
/// * `Ok(Vec<String>)` - The lines from `Linguae.txt`.
/// * `Err(io::Error)` - If the file cannot be read.
///
/// # Examples
///
/// ```ignore
/// let langs = vernaculars("/path/to/data")?;
/// for lang in langs {
///     println!("Found language: {}", lang);
/// }
/// ```
pub fn vernaculars(basedir: &str) -> io::Result<Vec<String>> {
    let path = format!("{}/Linguae.txt", basedir);
    let lines = fileio::do_read(path)?;
    Ok(lines)
}

/// Returns the recommended liturgical color for a given text describing
/// the feast or celebration, using rules adapted from `liturgical_color()`
/// in the Perl code. If no rule matches, defaults to `"black"`.
///
/// Examples of input strings might be `"Beatae Mariae"`, `"Vigilia Pentecostes"`,
/// `"Defunctorum"`, `"Palmis"`, etc.
pub fn liturgical_color(s: &str) -> &'static str {
    if match_blue(s) {
        return "blue";
    }
    if match_red1(s) {
        return "red";
    }
    if match_grey(s) {
        return "grey";
    }
    if match_black1(s) {
        return "black";
    }
    if match_purple(s) {
        return "purple";
    }
    if match_black2(s) {
        return "black";
    }
    if match_green(s) {
        return "green";
    }
    if match_red2(s) {
        return "red";
    }
    "black"
}

/// Returns `true` if the blue rule matches:
/// (contains one of "Beatae Mari", "Beatæ Mari", "Sanctae Mari", or "Sanctæ Mari")
/// and does NOT contain "Vigil". (Both checks are case–sensitive.)
fn match_blue(s: &str) -> bool {
    (s.contains("Beatae Mari")
        || s.contains("Beatæ Mari")
        || s.contains("Sanctae Mari")
        || s.contains("Sanctæ Mari"))
        && !s.contains("Vigil")
}

/// Returns `true` if the first red rule matches:
/// (case–insensitive search for "vigilia pentecostes",
///  "quattuor temporum pentecostes", "decollatione", or "martyr")
fn match_red1(s: &str) -> bool {
    let ls = s.to_lowercase();
    ls.contains("vigilia pentecostes")
        || ls.contains("quattuor temporum pentecostes")
        || ls.contains("decollatione")
        || ls.contains("martyr")
}

/// Returns `true` if the grey rule matches:
/// (case–insensitive search for "defunctorum", "parasceve", or "morte")
fn match_grey(s: &str) -> bool {
    let ls = s.to_lowercase();
    ls.contains("defunctorum") || ls.contains("parasceve") || ls.contains("morte")
}

/// Returns `true` if the first black rule matches:
/// (the string starts with "In Vigilia Ascensionis" or "In Vigilia Epiphaniæ")
fn match_black1(s: &str) -> bool {
    s.starts_with("In Vigilia Ascensionis") || s.starts_with("In Vigilia Epiphaniæ")
}

/// Returns `true` if the purple rule matches:
/// (case–insensitive search for any of a set of substrings)
fn match_purple(s: &str) -> bool {
    let ls = s.to_lowercase();
    ls.contains("vigilia")
        || ls.contains("quattuor")
        || ls.contains("rogatio")
        || ls.contains("passion")
        || ls.contains("palmis")
        || ls.contains("gesim")
        || ls.contains("majoris hebdomadæ sanctæ")
        || ls.contains("majoris hebdomadæ")
        || ls.contains("hebdomadæ sanctæ")
        || ls.contains("hebdomadæ")
        || ls.contains("sabbato sancto")
        || ls.contains("dolorum")
        || ls.contains("ciner")
        || ls.contains("adventus")
}

/// Returns `true` if the second black rule matches:
/// (case–insensitive search for "conversione", "dedicatione", "cathedra", "oann",
///  "pasch", "confessor", "ascensio", or "cena")
fn match_black2(s: &str) -> bool {
    let ls = s.to_lowercase();
    ls.contains("conversione")
        || ls.contains("dedicatione")
        || ls.contains("cathedra")
        || ls.contains("oann")
        || ls.contains("pasch")
        || ls.contains("confessor")
        || ls.contains("ascensio")
        || ls.contains("cena")
}

/// Returns `true` if the green rule matches:
/// (case–insensitive search for either a "pentecosten" occurrence that is NOT followed by
///  "infra octavam", or the substrings "epiphaniam" or "post octavam")
fn match_green(s: &str) -> bool {
    let ls = s.to_lowercase();
    crate::regex::contains_without_following(&ls, "pentecosten", "infra octavam")
        || ls.contains("epiphaniam")
        || ls.contains("post octavam")
}

/// Returns `true` if the second red rule matches:
/// (case–insensitive search for "pentecostes", "evangel", "innocentium",
///  "sanguinis", "cruc", or "apostol")
fn match_red2(s: &str) -> bool {
    let ls = s.to_lowercase();
    ls.contains("pentecostes")
        || ls.contains("evangel")
        || ls.contains("innocentium")
        || ls.contains("sanguinis")
        || ls.contains("cruc")
        || ls.contains("apostol")
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_match_blue() {
        // Should be blue if one of the substrings is present and "Vigil" is not.
        assert!(match_blue("Beatae Mari in festo"));
        assert!(match_blue("Sanctæ Mari celebration"));
        // Should not match if "Vigil" appears anywhere.
        assert!(!match_blue("Sanctae Mari during Vigil"));
    }

    #[test]
    fn test_match_red1() {
        assert!(match_red1("Vigilia Pentecostes celebration"));
        assert!(match_red1("quattuor temporum pentecostes event"));
        assert!(match_red1("Decollatione of a saint"));
        assert!(match_red1("martyr story"));
        // Check case–insensitivity.
        assert!(match_red1("MARTYR remembrance"));
    }

    #[test]
    fn test_match_grey() {
        assert!(match_grey("Defunctorum mass"));
        assert!(match_grey("parasceve service"));
        assert!(match_grey("Morte lament"));
        // Case–insensitive check.
        assert!(match_grey("mOrTe reflection"));
    }

    #[test]
    fn test_match_black1() {
        assert!(match_black1("In Vigilia Ascensionis, something"));
        assert!(match_black1("In Vigilia Epiphaniæ at the start"));
        assert!(!match_black1("Something In Vigilia Ascensionis later"));
    }

    #[test]
    fn test_match_purple() {
        assert!(match_purple("Rogatio prayer"));
        assert!(match_purple("Majoris Hebdomadæ Sanctæ celebration"));
        assert!(match_purple("Adventus season"));
        // Even a match on a short substring like "passion" qualifies.
        assert!(match_purple("Passion play"));
    }

    #[test]
    fn test_match_black2() {
        assert!(match_black2("Conversione ceremony"));
        assert!(match_black2("dedicatione event"));
        assert!(match_black2("Cathedra honor"));
        assert!(match_black2("oann narrative"));
        // Case–insensitive.
        assert!(match_black2("ASCENSIO details"));
    }

    #[test]
    fn test_match_green() {
        // Test the "pentecosten" part (with negative lookahead).
        assert!(match_green("Pentecosten celebration")); // no "infra octavam"
        assert!(!match_green("Pentecosten event with infra octavam later"));

        // Test the other alternatives.
        assert!(match_green("Epiphaniam service"));
        assert!(match_green("After post octavam, something"));
    }

    #[test]
    fn test_match_red2() {
        assert!(match_red2("Pentecostes feast")); // note the final "s"
        assert!(match_red2("Evangel reading"));
        assert!(match_red2("Innocentium memory"));
        assert!(match_red2("Sanguinis ritual"));
        assert!(match_red2("Cruc service"));
        assert!(match_red2("Apostol message"));
    }


    #[test]
    fn test_liturgical_color() {
        // Test blue rule:
        assert_eq!(liturgical_color("Beatae Mari celebration"), "blue");
        // Test red1:
        assert_eq!(
            liturgical_color("Vigilia Pentecostes celebration"),
            "red"
        );
        // Test grey:
        assert_eq!(liturgical_color("Defunctorum mass"), "grey");
        // Test black1:
        assert_eq!(
            liturgical_color("In Vigilia Ascensionis something"),
            "black"
        );
        // Test purple:
        assert_eq!(liturgical_color("Rogatio prayer"), "purple");
        // Test black2:
        assert_eq!(liturgical_color("Conversione ceremony"), "black");
        // Test green:
        assert_eq!(liturgical_color("Pentecosten celebration"), "green");
        // Test red2:
        assert_eq!(liturgical_color("Pentecostes feast"), "red");
        // Test default (matches none):
        assert_eq!(liturgical_color("Some other text"), "black");
    }
}