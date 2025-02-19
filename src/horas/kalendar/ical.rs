//! ical.rs
//!
//! This module produces an iCalendar (ICS) output for the Divinum Officium kalendar.
//!
//! It builds a vector of lines (each a `String`) using Rust’s native formatting
//! (via `format!()`) rather than Perl’s heredoc, and it replaces regex‐based
//! substitutions (used to abbreviate entries) with a custom function.
//!
//! The main public function is `ical_output(version1: &str, kyear: i32) -> Vec<String>`.

use chrono::{Local, NaiveDate, Timelike, Datelike};
use std::fmt::Write as FmtWrite;
use std::path::Path;
use crate::date::leap_year;
use crate::date::ydays_to_date;
use crate::horas::kalendar::ordo::ordo_entry;
use crate::regex::{replace_word_prefix, replace_all_case_insensitive};

/// Abbreviates an entry string by performing a series of replacements,
/// in the same order as in the original Perl code.
pub fn abbreviate_entry(entry: &str) -> String {
    let mut s = entry.to_string();
    
    // Literal (exact-match) replacements in order.
    // (Order matters: e.g. "Duplex majus" must be replaced before "Duplex".)
    let literal_replacements: &[(&str, &str)] = &[
        ("Duplex majus", "dxm"),
        ("Duplex", "dx"),
        ("Semiduplex", "sdx"),
        ("Simplex", "splx"),
        ("classis", "cl."),
        (" Domini Nostri Jesu Christi", " D.N.J.C."),
        ("Beatæ Mariæ Virginis", "B.M.V."),
        ("Abbatis", "Abb."),
        ("Apostoli", "Ap."),
        ("Apostolorum", "App."),
        ("Doctoris", "Doct."),
        ("Ecclesiæ", "Eccl."),
        ("Episcopi", "Ep."),
        ("Episcoporum", "Epp."),
        ("Evangelistæ", "Evang."),
        ("Martyris", "M."),
        ("Martyrum", "Mm."),
        ("Papæ", "P."),
        ("Viduæ", "Vid."),
        ("Secunda", "II"),
        ("Tertia", "III"),
        ("Quarta", "IV"),
        ("Quinta", "V"),
        ("Sexta", "VI"),
        ("Dominica minor", "Dom. min."),
        (" Ferial", ""),
        ("Feria major", "Fer. maj."),
        ("Feria privilegiata", "Fer. priv."),
        ("post Octavam", "post Oct."),
        ("Augusti", "Aug."),
    ];
    
    for &(from, to) in literal_replacements {
        s = s.replace(from, to);
    }
    
    // Case–insensitive literal replacements using our helper.
    // (The original Perl uses the /i flag for these.)
    s = replace_all_case_insensitive(&s, "Hebdomadam", "Hebd.");
    s = replace_all_case_insensitive(&s, "Quadragesim.", "Quad.");
    
    // Now, for patterns that in Perl were written as regexes with \w+,
    // we use a special function that splits the text into words and,
    // if a word starts with the given prefix (ignoring case), replaces the whole word.
    s = replace_word_prefix(&s, "Confessor", "Conf.");
    s = replace_word_prefix(&s, "Virgin", "Vir.");
    
    // Finally, perform the month abbreviation replacement.
    s = replace_months_abbrev(&s);
    
    s
}


/// Helper function to replace occurrences of the pattern `(Septem|Octo|Novem|Decem)bris`
/// with the captured group followed by "b.".
///
/// For example, "Septembers" becomes "Septemb.".
pub fn replace_months_abbrev(s: &str) -> String {
    // The candidate prefixes (in lowercase) that we want to match.
    const CANDIDATES: [&str; 4] = ["septem", "octo", "novem", "decem"];
    // Suffix we expect after the prefix.
    const SUFFIX: &str = "bris";

    // We'll work on a lowercase copy for matching.
    let s_lower = s.to_lowercase();
    let s_bytes = s.as_bytes(); // original string as bytes (to help with slicing)
    let s_lower_bytes = s_lower.as_bytes();

    let mut output = String::with_capacity(s.len());
    let mut pos = 0;
    let len = s.len();

    while pos < len {
        let mut matched = false;
        // For each candidate prefix:
        for &cand in &CANDIDATES {
            let cand_full = format!("{}{}", cand, SUFFIX);
            let cand_full_len = cand_full.len();
            // Ensure there is room for the candidate + suffix.
            if pos + cand_full_len <= len {
                // Get the slice from our lower-case copy.
                let slice = &s_lower[pos..pos + cand_full_len];
                if slice == cand_full {
                    // We have a match.
                    // Append the original prefix (preserving its original case)
                    // i.e. the first cand.len() characters from the original.
                    let prefix = &s[pos..pos + cand.len()];
                    output.push_str(prefix);
                    output.push_str("b.");
                    pos += cand_full_len;
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            // If no candidate matched at the current position, copy one character.
            // (Assuming valid UTF-8, copying by char is safe.)
            let ch = s[pos..].chars().next().unwrap();
            output.push(ch);
            pos += ch.len_utf8();
        }
    }
    output
}

/// Prepares iCalendar (ICS) output as a vector of lines.
/// 
/// # Parameters
/// - `version1`: the version string (e.g. "1960")
/// - `kyear`: the calendar year (e.g. 2024)
/// 
/// # Returns
/// A vector of lines (`Vec<String>`) representing the ICS file.
pub fn ical_output(version1: &str, kyear: i32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push("Content-Type: text/calendar; charset=utf-8".to_string());
    lines.push(format!(
        "Content-Disposition: attachment; filename=\"{} - {}.ics\"",
        version1, kyear
    ));
    lines.push("".to_string());
    lines.push("BEGIN:VCALENDAR".to_string());
    lines.push("VERSION:2.0".to_string());
    lines.push("PRODID:-//divinumofficium.com//".to_string());
    lines.push("CALSCALE:GREGORIAN".to_string());
    lines.push("SOURCE:https://divinumofficium.com/cgi-bin/horas/kalendar.pl".to_string());

    // Calculate the total number of days in the year.
    let to = 365 + if leap_year(kyear) { 1 } else { 0 };

    // Build DTSTAMP using the current local time.
    let now = Local::now();
    let dtstamp = format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );

    // For each day of the year, create a VEVENT.
    for cday in 1..=to {
        let (yday, ymonth, yyear) = ydays_to_date(cday, kyear);
        let dtstart = format!("{:04}{:02}{:02}", yyear, ymonth, yday);
        let day_str = format!("{:02}-{:02}-{:04}", ymonth, yday, yyear);
        // Call ordo_entry with compare = false and winneronly = true.
        let (e, _c2, _cv) = ordo_entry(&day_str, version1, false, true);
        // Abbreviate the entry.
        let e = abbreviate_entry(&e);
        lines.push("BEGIN:VEVENT".to_string());
        lines.push(format!("UID:{}", cday));
        lines.push(format!("DTSTAMP:{}", dtstamp));
        lines.push(format!("SUMMARY:{}", e));
        lines.push(format!("DTSTART;VALUE=DATE:{}", dtstart));
        lines.push("END:VEVENT".to_string());
    }
    lines.push("END:VCALENDAR".to_string());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_months_abbrev_basic() {
        // Basic tests using different case variants.
        assert_eq!(
            replace_months_abbrev("Septembris"),
            "Septemb."
        );
        assert_eq!(
            replace_months_abbrev("octobris"),
            "octob."
        );
        assert_eq!(
            replace_months_abbrev("NOVEMBRIS"),
            "NOVEMB."
        );
        assert_eq!(
            replace_months_abbrev("Decembris"),
            "Decemb."
        );
    }

    #[test]
    fn test_replace_months_abbrev_embedded() {
        // When the month appears within a longer string.
        let input = "Today is Septembris and tomorrow is Octobris.";
        let expected = "Today is Septemb. and tomorrow is Octob.";
        assert_eq!(replace_months_abbrev(input), expected);
    }

    #[test]
    fn test_replace_months_abbrev_no_match() {
        // Strings that don't contain any of the target patterns should remain unchanged.
        let input = "This is a test string without a matching month.";
        assert_eq!(replace_months_abbrev(input), input);
    }

    #[test]
    fn test_abbreviate_entry() {
        let original = "Duplex majus and Duplex and Semiduplex and Simplex, classis Domini Nostri Jesu Christi, Beatæ Mariæ Virginis, Abbatis, Apostoli, Apostolorum, Confessorsomething, Doctoris, Ecclesiæ, Episcopi, Episcoporum, Evangelistæ, Martyris, Martyrum, Papæ, Viduæ, Virginity, Hebdomadam, Quadragesim., Secunda, Tertia, Quarta, Quinta, Sexta, Dominica minor, Ferial, Feria major, Feria privilegiata, post Octavam, Augusti, Septembris";
        let abbreviated = abbreviate_entry(original);
        // We expect the following substitutions (one or more of these):
        assert!(abbreviated.contains("dxm"));
        assert!(abbreviated.contains("dx"));
        assert!(abbreviated.contains("sdx"));
        assert!(abbreviated.contains("splx"));
        assert!(abbreviated.contains("cl."));
        assert!(abbreviated.contains("D.N.J.C."));
        assert!(abbreviated.contains("B.M.V."));
        assert!(abbreviated.contains("Abb."));
        assert!(abbreviated.contains("Ap."));
        assert!(abbreviated.contains("App."));
        assert!(abbreviated.contains("Conf."));
        assert!(abbreviated.contains("Doct."));
        assert!(abbreviated.contains("Eccl."));
        assert!(abbreviated.contains("Ep."));
        assert!(abbreviated.contains("Epp."));
        assert!(abbreviated.contains("Evang."));
        assert!(abbreviated.contains("M."));
        assert!(abbreviated.contains("Mm."));
        assert!(abbreviated.contains("P."));
        assert!(abbreviated.contains("Vid."));
        assert!(abbreviated.contains("Vir."));
        assert!(abbreviated.contains("Hebd."));
        assert!(abbreviated.contains("Quad."));
        // Test the month abbreviation replacement:
        assert!(abbreviated.contains("Septemb."));
    }

    #[test]
    fn test_abbreviate_entry_basic() {
        let input = "Duplex majus Duplex Semiduplex Simplex classis Domini Nostri Jesu Christi \
                     Beatæ Mariæ Virginis Abbatis Apostoli Apostolorum Confessorus \
                     Doctoris Ecclesiæ Episcopi Episcoporum Evangelistæ Martyris Martyrum \
                     Papæ Viduæ Virginibus Hebdomadam Quadragesim. Secunda Tertia Quarta \
                     Quinta Sexta Dominica minor Ferial Feria major Feria privilegiata post Octavam \
                     Augusti Septembers";
        let expected = "dxm dx sdx splx cl. D.N.J.C. B.M.V. Abb. Ap. App. Conf. Doct. Eccl. Ep. \
                        Epp. Evang. M. Mm. P. Vid. Vir. Hebd. Quad. II III IV V VI Dom. min. \
                        Fer. maj. Fer. priv. post Oct. Aug. Septemb.";
        let output = abbreviate_entry(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_abbreviate_entry_overlap() {
        // "Duplex majus" should be replaced before "Duplex".
        let input = "Duplex majus and Duplex";
        let expected = "dxm and dx";
        let output = abbreviate_entry(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_abbreviate_entry_no_change() {
        let input = "This entry has no abbreviations.";
        let output = abbreviate_entry(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_replace_months_abbrev() {
        // Test the custom rule for months.
        let input = "Septembers Octobris Novembers Decembers";
        let expected = "Septemb. Octob. Novemb. Decemb.";
        let output = replace_months_abbrev(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_replace_all_case_insensitive() {
        // Test that a pattern like "Confessor\w+" is replaced case-insensitively.
        let input = "Confessorus confessorABC CONFESSORtest";
        let expected = "Conf. conf. Conf.";
        let output = replace_word_prefix(&s, "Confessor", "Conf.");;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_ical_output() {
        // For testing, we call ical_output with dummy version1 and kyear.
        let lines = ical_output("TestVersion", 2023);
        // The first line should be the content-type.
        assert!(lines[0].starts_with("Content-Type:"));
        // There should be a "BEGIN:VCALENDAR" line.
        assert!(lines.iter().any(|l| l == "BEGIN:VCALENDAR"));
        // The last line should be "END:VCALENDAR".
        assert_eq!(lines.last().unwrap(), "END:VCALENDAR");
    }

    #[test]
    fn test_ical_output_contains_header_and_one_event() {
        // Use a fixed version string and year.
        let version1 = "TestVer";
        let kyear = 2024; // Leap year.
        let lines = ical_output(version1, kyear);
        // Check header lines.
        assert!(lines.contains(&"BEGIN:VCALENDAR".to_string()));
        assert!(lines.contains(&format!("Content-Disposition: attachment; filename=\"{} - {}.ics\"", version1, kyear)));
        // Check that there is at least one VEVENT.
        let event_begin_count = lines.iter().filter(|l| l == &"BEGIN:VEVENT").count();
        assert!(event_begin_count > 0, "Expected at least one VEVENT, found none.");
        // Check that the final line is END:VCALENDAR.
        assert_eq!(lines.last().unwrap(), "END:VCALENDAR");
    }
}
