//! kal.rs
//!
//! This module prepares Kalendarium output for Divinum Officium.
//! 
//! It defines functions for converting numbers to Roman numerals,
//! computing “romanday” strings, producing a dominica letter,
//! computing an epact cycle, and ultimately building table rows.

use chrono::{Datelike, Local, Timelike};
use std::collections::HashMap;
use crate::liturgical_color;
use crate::{STARDAYS, MONTH_LENGTH, MONTH_NAMES};

/// Converts a number (assumed between 1 and 29) into a Roman numeral string.
/// If the provided version does NOT contain "196", then a trailing 'i' is replaced by 'j'.
pub fn romannumber(mut d: i32, version: &str) -> String {
    let mut o = String::new();
    if d > 19 {
        o.push_str("x");
        d -= 10;
    }
    if d > 9 {
        o.push_str("x");
        d -= 10;
    }
    if d == 9 {
        o.push_str("ix");
    } else if d == 4 {
        o.push_str("iv");
    } else {
        if d > 4 {
            o.push_str("v");
            d -= 5;
        }
        o.push_str(&"i".repeat(d as usize));
    }
    if !version.contains("196") && o.ends_with('i') {
        o.pop();
        o.push('j');
    }
    o
}

/// Returns a “romanday” string for a date given in "mm-dd" format.
/// Uses the provided version when calling romannumber.
pub fn romanday(date: &str, version: &str) -> String {
    if date.len() < 5 {
        return String::new();
    }
    let m: i32 = date[0..2].parse().unwrap_or(0);
    let d: i32 = date[3..5].parse().unwrap_or(0);
    if d == 1 {
        return "{Kal.}".to_string();
    }
    let id = if m == 3 || m == 5 || m == 7 || m == 10 { 15 } else { 13 };
    if d == id {
        return "{Idib.}".to_string();
    }
    if d == MONTH_LENGTH[m as usize] || d == (id - 1) {
        return "{Prid.}".to_string();
    }
    if d > id {
        return romannumber(MONTH_LENGTH[m as usize] - d + 2, version);
    }
    let no = id - 8;
    if d == no {
        return "{Non.}".to_string();
    }
    if d == (no - 1) {
        return "{Prid.}".to_string();
    }
    if d > no {
        return romannumber(id - d + 1, version);
    }
    romannumber(no - d + 1, version)
}

/// Returns the dominica letter for the given counter (cycling through 0..6).
/// If the letter is 'A', it is replaced with "{A}".
pub fn domlet(domlet_counter: i32) -> String {
    let index = (domlet_counter.rem_euclid(7)) as usize;
    let letters = "Abcdefg";
    let letter = letters.chars().nth(index).unwrap_or(' ');
    if letter == 'A' {
        "{A}".to_string()
    } else {
        letter.to_string()
    }
}

/// Computes the epact cycle string for a given day-of-cycle `d`.
/// Mimics the original Perl code’s behavior. (For d == 365, returns "19 {xx}")
pub fn epactcycle(d: i32, version: &str) -> String {
    if d == 365 {
        return "19 {xx}".to_string();
    }
    let mut i = 0;
    while i < STARDAYS.len() {
        let cond = d > STARDAYS[i];
        i += 1;
        if !cond {
            break;
        }
    }
    let r_raw = STARDAYS[i - 1] - d;
    if r_raw == 0 {
        return "{*}".to_string();
    }
    let mut r_val = r_raw;
    let mut o = String::new();
    if i % 2 == 1 {
        r_val += 1;
        if r_val == 26 {
            o.push_str("25. ");
        } else if r_val == 25 {
            o.push_str("{xxv.} ");
        }
        if r_val < 26 {
            r_val -= 1;
        }
    } else {
        if r_val == 25 {
            o.push_str("25. ");
        }
    }
    format!("{}{{{}}}", o, romannumber(r_val, version))
}

/// Returns a Latin‐uppercase version of the input string.
/// It converts the entire string to uppercase and replaces any "æ" with "Æ".
pub fn latin_uppercase(s: &str) -> String {
    s.to_uppercase().replace("æ", "Æ")
}

/// Reads the rank entry from a Sancti file for the given entry and version.
/// Constructs a filename via `subdirname("Sancti", ver)` and then loads the file via `setupstring`.
/// Splits the "Rank" field on ";;" and returns a tuple (antiphon, rankname_font).
pub fn findkalentry(entry: &str, ver: &str) -> (String, String) {
    let filename = format!("{}{}.txt", subdirname("Sancti", ver), entry);
    let saint_map = match setupstring("Latin", &filename, &[]) {
        Some(map) => map,
        None => return (String::new(), String::new()),
    };
    let rank_field = saint_map.get("Rank").unwrap_or(&String::new());
    let parts: Vec<&str> = rank_field.split(";;").collect();
    if parts.is_empty() || parts[0].is_empty() {
        return (String::new(), String::new());
    }
    // Parse the rank value from the third part.
    let rank: f64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let mut rankname_str = rankname("Latin");
    if ver.contains("Monastic") || ver.contains("Ordo Praedicatorum") {
        rankname_str = rankname_str.replace("IV. classis", "Memoria");
    }
    let antiphon = setfont(
        liturgical_color(parts[0]),
        if rank > 4.0 && !contains_ci(parts[0], "octava") && !contains_ci(parts[0], "vigilia") {
            latin_uppercase(parts[0])
        } else {
            parts[0].to_string()
        },
    );
    let rankname_font = setfont("1 maroon", &format!(" {}", rankname_str));
    (antiphon, rankname_font)
}

/// Returns the kalendar entry for a given date (in "mm-dd-yyyy" format) and version string.
pub fn kalendar_entry(date: &str, ver: &str) -> String {
    if date.len() < 5 {
        return String::new();
    }
    let date_trimmed = &date[0..5];
    let kal_str = get_kalendar(ver, date_trimmed);
    let mut kal_entries: Vec<&str> = kal_str.split('~').collect();
    if kal_entries.is_empty() {
        return String::new();
    }
    let first = kal_entries.remove(0);
    let (antiphon, rankfont) = findkalentry(first, ver);
    let mut output = format!("{} {}", antiphon, rankfont);
    if (ver.contains("1955") || ver.contains("196"))
        && date_trimmed.starts_with("01-")
        && {
            let d: i32 = date_trimmed[3..5].parse().unwrap_or(0);
            (7..=12).contains(&d)
        }
    {
        return String::new();
    }
    for ke in kal_entries {
        let (d1, _d2) = findkalentry(ke, ver);
        output.push_str(&format!(" Com. {}", d1));
    }
    output
}

/// Returns a table row for the given date (in "mm-dd-yyyy" format) and current day number (`cday`).
/// Also takes the primary version (`version1`), a compare flag, a secondary version (`version2`),
/// and a dominica counter (`domlet_counter`). Returns a 5–tuple:
/// (epact cycle, dominica letter, romanday, numeric day, kalendar entry).
pub fn table_row(
    date: &str,
    cday: i32,
    version1: &str,
    compare: bool,
    version2: &str,
    domlet_counter: i32,
) -> (String, String, String, i32, String) {
    let d: i32 = date.get(3..5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut c = kalendar_entry(date, version1);
    if compare {
        let c2 = kalendar_entry(date, version2);
        c.push_str(&format!(
            "&nbsp;<br/>{}",
            if c2.is_empty() { "&nbsp;" } else { &c2 }
        ));
    }
    (
        epactcycle(cday, version1),
        domlet(domlet_counter),
        romanday(date, version1),
        d,
        c,
    )
}

/// Returns an HTML table row for a given note.
/// Loads the comment text from "Psalterium/Comment.txt" and wraps it in a table row.
pub fn note(note: &str, lang: &str, lang1: &str) -> String {
    let comm_map = setupstring(lang1, "Psalterium/Comment.txt", &[]).unwrap_or_default();
    let comment_text = comm_map.get(&format!("{} note", note)).unwrap_or(&String::new());
    format!(
        r#"<TR><TD COLSPAN="5" ALIGN="LEFT">{}</TD></TR>"#,
        setfont("1", comment_text)
    )
}

/// Produces the HTML header for the kalendar page.
/// Uses Rust formatting and builds the output as a single String.
pub fn html_header() -> String {
    // Build the header line by line.
    let mut output = String::new();

    // Add the top anchor.
    output.push_str("<A ID=\"top\"></A>\n");

    // Start the H1 header.
    output.push_str("<H1>\n");
    output.push_str("<FONT COLOR=\"MAROON\" SIZE=\"+1\"><B><I>Kalendarium</I></B></FONT>&ensp;\n");

    // Get version string. (Assume get_version() is defined in globals.)
    let version1 = crate::globals::get_version();
    // For demonstration, we assume there's no version comparison.
    let compare = false;
    let vers = if compare {
        format!("{} / {}", version1, "version2")
    } else {
        version1
    };
    output.push_str(&format!("<FONT COLOR=\"RED\" SIZE=\"+1\">{}</FONT>\n", vers));
    output.push_str("</H1>\n");

    // Add navigation links.
    output.push_str("<P ALIGN=\"CENTER\">\n");
    output.push_str("<A HREF=\"#\" onclick=\"callbrevi();\">Divinum Officium</A>&nbsp;&ensp;\n");
    output.push_str("<A HREF=\"#\" onclick=\"callmissa();\">Sancta Missa</A>&nbsp;&ensp;\n");
    output.push_str("<A HREF=\"#\" onclick=\"setkm(0);\">Ordo</A>\n");
    output.push_str("</P>\n");

    // Add month navigation (links for months 2 to 12; month 1 is plain text).
    output.push_str("<P ALIGN=\"CENTER\">\n");
    // Assume MONTH_NAMES is defined in a constants module.
    for i in 1..=12 {
        // Take the first three characters of the month name.
        let mn = &MONTH_NAMES[i][..3];
        if i == 1 {
            output.push_str(mn);
        } else {
            // Use an HTML link with an anchor.
            output.push_str(&format!("<A HREF=\"#{}\">{}</A>", mn, mn));
        }
        output.push_str("&nbsp;&ensp;");
    }
    output.push_str("</P>\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_romannumber() {

        assert_eq!(romannumber(4, "Monastic 1962"), "iv".to_string());
        assert_eq!(romannumber(9, "Monastic 1962"), "ix".to_string());
        if romannumber(3, "Monastic 1962").ends_with('j') {
            panic!("Expected romannumber to end with 'i'");
        }

        // For a version NOT containing "196", trailing "i" becomes "j" 
        // ( iii -> iij )
        if romannumber(3, "Tridentine 1570").ends_with('i') {
            panic!("Expected trailing 'i' to be replaced with 'j'");
        }
    }

    #[test]
    fn test_romanday() {
        // For a date with day 01, we return "{Kal.}"
        assert_eq!(romanday("03-01", "Monastic 1962"), "{Kal.}".to_string());
        // For a date with day 15 in month 3 (since 3 is in {3,5,7,10}), we return "{Idib.}"
        assert_eq!(romanday("03-15", "Monastic 1962"), "{Idib.}".to_string());
    }

    #[test]
    fn test_epactcycle() {
        // For day 365, expect "19 {xx}"
        assert_eq!(epactcycle(365, "Monastic 1962"), "19 {xx}".to_string());
    }

    #[test]
    fn test_latin_uppercase() {
        let s = "æther";
        assert_eq!(latin_uppercase(s), "ÆTHER".to_string());
    }

    #[test]
    fn test_html_header_contains_links() {
        let header = html_header();
        assert!(header.contains("Kalendarium"));
        assert!(header.contains("Divinum Officium"));
    }
}
