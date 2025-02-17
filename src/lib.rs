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
pub fn liturgical_color(input: &str) -> &'static str {
    let text = input;
    // The original code sets `$_ = shift;`, then does a sequence of if-statements.
    // We replicate them directly here.
    if text.contains("Beat") || text.contains("Sanct") {
        // In Perl: `/(?:Beat|Sanct)(?:ae|æ) Mari/ && !/Vigil/`
        // We'll refine the partial:
        let re1 = regex::Regex::new(r"(?:Beat|Sanct)(?:ae|æ)\s+Mari").unwrap();
        let re2 = regex::Regex::new(r"Vigil").unwrap();
        if re1.is_match(text) && !re2.is_match(text) {
            return "blue";
        }
    }
    // `return 'red' if (/(?:Vigilia Pentecostes|Quattuor Temporum Pentecostes|Decollatione|Martyr)/i);`
    {
        let re = regex::Regex::new(r"(?i)(?:Vigilia Pentecostes|Quattuor Temporum Pentecostes|Decollatione|Martyr)").unwrap();
        if re.is_match(text) {
            return "red";
        }
    }
    // `return 'grey' if (/(?:Defunctorum|Parasceve|Morte)/i);`
    {
        let re = regex::Regex::new(r"(?i)(?:Defunctorum|Parasceve|Morte)").unwrap();
        if re.is_match(text) {
            return "grey";
        }
    }
    // `return 'black' if (/^In Vigilia Ascensionis|^In Vigilia Epiphaniæ/);`
    {
        let re = regex::Regex::new(r"^(In Vigilia Ascensionis|In Vigilia Epiphaniæ)").unwrap();
        if re.is_match(text) {
            return "black";
        }
    }
    // `return 'purple' if (...)`
    {
        let re = regex::Regex::new(r"(?i)(?:Vigilia|Quattuor|Rogatio|Passion|Palmis|gesim|(?:Majoris )?Hebdomadæ(?: Sanctæ)?|Sabbato Sancto|Dolorum|Ciner|Adventus)").unwrap();
        if re.is_match(text) {
            return "purple";
        }
    }
    // `return 'black' if (/(?:Conversione|Dedicatione|Cathedra|oann|Pasch|Confessor|Ascensio|Cena)/i);`
    {
        let re = regex::Regex::new(r"(?i)(?:Conversione|Dedicatione|Cathedra|oann|Pasch|Confessor|Ascensio|Cena)").unwrap();
        if re.is_match(text) {
            return "black";
        }
    }
    // `return 'green' if (/(?:Pentecosten(?!.*infra octavam)|Epiphaniam|post octavam)/i);`
    {
        let re = regex::Regex::new(r"(?i)(?:Pentecosten(?!.*infra octavam)|Epiphaniam|post octavam)").unwrap();
        if re.is_match(text) {
            return "green";
        }
    }
    // `return 'red' if (/(?:Pentecostes|Evangel|Innocentium|Sanguinis|Cruc|Apostol)/i);`
    {
        let re = regex::Regex::new(r"(?i)(?:Pentecostes|Evangel|Innocentium|Sanguinis|Cruc|Apostol)").unwrap();
        if re.is_match(text) {
            return "red";
        }
    }
    // Otherwise, default
    "black"
}
