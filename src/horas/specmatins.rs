use std::collections::HashMap;
use crate::regex::ci_contains;
use crate::regex::contains_any_ci;
use crate::regex::contains_in_order;
use crate::regex::contains_digit_followed_by_dot_or_comma;

pub const LT1960_DEFAULT: i32 = 0;
pub const LT1960_FERIAL: i32 = 1;
pub const LT1960_SUNDAY: i32 = 2;
pub const LT1960_SANCTORAL: i32 = 3;
pub const LT1960_OCTAVEII: i32 = 4;
pub const LT1960_OCTAVE: i32 = 5;

/// 1. dayofweek2i
/// Given a day-of-week (as a 1–based number), if greater than 3 subtract 3.
/// If dayofweek is 0, default to 1.
pub fn dayofweek2i(dayofweek: u32) -> u32 {
    let mut i = if dayofweek == 0 { 1 } else { dayofweek };
    if i > 3 {
        i -= 3;
    }
    i
}

/// 2. cujus_q
/// Compute a shift value based on the given parameters.
/// Parameters:
/// - `rule`: the rule string
/// - `commune`: the commune string
/// - `input`: the string to test (formerly shift from Cujus festum)
///
/// The original Perl logic:
///   return 1 if $rule =~ /Quorum Festum/;
///   return 4 if $commune =~ /C11|08-15|09-08|12-08/;
///   then if input contains "basilic" return -2;
///   if it contains "S. P. N. Benedicti Abbatis" return 5;
///   then if input contains any of ["virgin", "vidu[aæ]", "poenitentis", "pœnitentis", "C6", "C7"]
///     (unless it contains any of "C2", "C3", "C4", "C5") add 2;
///   and if input contains any of ["ss.", "sanctorum", "sociorum"] add 1.
pub fn cujus_q(rule: &str, commune: &str, input: &str) -> i32 {
    if ci_contains(rule, "Quorum Festum") {
        return 1;
    }
    if contains_any_ci(commune, &["C11", "08-15", "09-08", "12-08"]) {
        return 4;
    }

    // Now work on the input string.
    let text = input;
    if ci_contains(text, "basilic") {
        return -2;
    }
    if ci_contains(text, "S. P. N. Benedicti Abbatis") {
        return 5;
    }

    let mut j = 0;
    let check_terms = ["virgin", "vidua", "viduæ", "poenitentis", "pœnitentis", "C6", "C7"];
    if contains_any_ci(text, &check_terms) && !contains_any_ci(text, &["C2", "C3", "C4", "C5"]) {
        j += 2;
    }
    if contains_any_ci(text, &["ss.", "sanctorum", "sociorum"]) {
        j += 1;
    }
    j
}

/// 3. get_c10_readingname
/// Given:
/// - `version`: version string (e.g. "1960", "1963", etc.)
/// - `month`: month number
/// - `day`: day number
///
/// Returns a reading name:
///   * If version does not contain "196" and month==9 and day in (9..14), returns "Lectio M101".
///   * Otherwise, computes a "satnum" = ((day - 1) / 7) + 1 (but if that equals 5, use 4),
///     and if version contains "1963" then returns "Lectio MXX<satnum>" (with month as two digits)
///     otherwise returns "Lectio MXX".
pub fn get_c10_readingname(version: &str, month: u32, day: u32) -> String {
    if !version.contains("196") && month == 9 && day > 8 && day < 15 {
        return "Lectio M101".to_string();
    }
    let mut satnum = ((day - 1) / 7) + 1;
    if satnum == 5 {
        satnum = 4;
    }
    if ci_contains(version, "1963") {
        format!("Lectio M{:02}{}", month, satnum)
    } else {
        format!("Lectio M{:02}", month)
    }
}

/// 4. lectiones_ex3_fiunt4
/// Given a hashmap `scrip` whose keys "Lectio1", "Lectio2", "Lectio3" hold string values,
/// split each value into parts if it contains the literal "¶\n". Otherwise, use the full value.
/// Then return the part at index `num - 1` (assuming 1-based numbering).
pub fn lectiones_ex3_fiunt4(scrip: &HashMap<String, String>, num: usize) -> Option<String> {
    let mut pieces = Vec::new();
    for i in 1..=3 {
        let key = format!("Lectio{}", i);
        if let Some(text) = scrip.get(&key) {
            if !text.contains('¶') {
                pieces.push(text.clone());
            } else {
                // Split on the literal substring "¶\n"
                let splits: Vec<&str> = text.split("¶\n").collect();
                pieces.extend(splits.into_iter().map(|s| s.to_string()));
            }
        }
    }
    // Return the (num - 1)th piece if it exists.
    pieces.get(num - 1).cloned()
}

/// 5. parenthesised_text
/// If the given `text` is less than 20 characters long OR contains a digit immediately followed by a dot or comma,
/// then return the text "formatted" in a small font (simulated here by wrapping in "<small>…</small>").
/// Otherwise, return the text wrapped in parentheses.
pub fn parenthesised_text(text: &str, smallfont: &str) -> String {
    if text.len() < 20 || contains_digit_followed_by_dot_or_comma(text) {
        // Simulate setfont(smallfont, text)
        format!("<{}>{}</{}>", smallfont, text, smallfont)
    } else {
        format!("({})", text)
    }
}

/// 6. beginwith
/// Splits the input string on whitespace, takes the first two words (if available),
/// then replaces any newline characters with a space.
pub fn beginwith(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().take(2).collect();
    let joined = words.join(" ");
    joined.replace('\n', " ")
}

/// 7. gettype1960
///
/// Determines the office “type” for 1960–style rubrics based on several parameters.
/// Parameters:
/// - `version`: version string (e.g. "1960", "Monastic", etc.)
/// - `votive`: a string indicating votive properties
/// - `dayname`: a string (e.g. "Dominica semiduplex", "post Nativitatem", etc.)
/// - `rule`: the rule string
/// - `rank`: a numeric rank (f64)
/// - `winner`: a string (e.g. may contain "Pasc1-0")
///
/// Returns a `LT160_` constant (= i32)
pub fn gettype1960(
    version: &str,
    votive: &str,
    dayname: &str,
    rule: &str,
    rank: f64,
    winner: &str,
) -> i32 {
    let mut type_code = LT1960_DEFAULT;
    // If version contains "196" and votive does not contain "C9" or "Defunctorum"
    if ci_contains(version, "196") && !contains_any_ci(votive, &["C9", "Defunctorum"]) {
        if ci_contains(dayname, "post Nativitatem") {
            type_code = LT1960_OCTAVEII;
        } else if rank < 2.0 || contains_any_ci(dayname, &["feria", "vigilia", "die"]) {
            type_code = LT1960_FERIAL;
        } else if !ci_contains(version, "Monastic")
            && (contains_in_order(dayname, "dominica", "semiduplex") || ci_contains(winner, "Pasc1-0"))
        {
            type_code = LT1960_SUNDAY;
        } else if rank < 5.0 {
            type_code = LT1960_SANCTORAL;
        }
    } else if ci_contains(version, "monastic") && !contains_any_ci(votive, &["C9", "Defunctorum"]) {
        if rank < 2.0
            || (contains_any_ci(dayname, &["feria", "vigilia", "die"])
                && !ci_contains(dayname, "infra octavam"))
        {
            type_code = LT1960_FERIAL;
        } else if ci_contains(dayname, "infra octavam") {
            type_code = LT1960_OCTAVE;
        } else if !ci_contains(version, "trident") && rank < 4.0 {
            type_code = LT1960_SANCTORAL;
        }
    }
    if contains_any_ci(rule, &["9 lectiones 1960", "12 lectiones"]) {
        type_code = LT1960_DEFAULT;
    }
    type_code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dayofweek2i() {
        assert_eq!(dayofweek2i(0), 1);
        assert_eq!(dayofweek2i(1), 1);
        assert_eq!(dayofweek2i(2), 2);
        assert_eq!(dayofweek2i(3), 3);
        assert_eq!(dayofweek2i(4), 1); // 4 - 3 = 1
        assert_eq!(dayofweek2i(7), 4); // 7 - 3 = 4
    }

    #[test]
    fn test_ci_contains() {
        assert!(ci_contains("Hello World", "world"));
        assert!(!ci_contains("Hello World", "mars"));
    }

    #[test]
    fn test_contains_any_ci() {
        let needles = ["foo", "bar"];
        assert!(contains_any_ci("this is a Bar test", &needles));
        assert!(!contains_any_ci("nothing here", &needles));
    }

    #[test]
    fn test_contains_in_order() {
        assert!(contains_in_order("This is dominica and then semiduplex", "dominica", "semiduplex"));
        assert!(!contains_in_order("semiduplex comes before dominica", "dominica", "semiduplex"));
        assert!(!contains_in_order("no match here", "dominica", "semiduplex"));
    }

    #[test]
    fn test_cujus_q() {
        // When rule contains "Quorum Festum", return 1.
        assert_eq!(cujus_q("Some Quorum Festum text", "", "irrelevant"), 1);
        // When commune contains one of the substrings, return 4.
        assert_eq!(cujus_q("", "C11", "irrelevant"), 4);
        // When input contains "basilic" then return -2.
        assert_eq!(cujus_q("", "", "This is basilic style"), -2);
        // When input contains "S. P. N. Benedicti Abbatis", return 5.
        assert_eq!(cujus_q("", "", "S. P. N. Benedicti Abbatis appears"), 5);
        // Otherwise, test accumulation:
        // If input contains "virgin" and no "C2", etc.
        assert_eq!(cujus_q("", "", "This virgin text"), 2);
        // And if also contains "sanctorum", add 1.
        assert_eq!(cujus_q("", "", "virgin and sanctorum"), 3);
        // If input contains forbidden pattern "C3", then no +2.
        assert_eq!(cujus_q("", "", "virgin but also C3 appears"), 0);
    }

    #[test]
    fn test_get_c10_readingname() {
        // When version does not contain "196", month==9, day in 9..14:
        assert_eq!(get_c10_readingname("SomeOther", 9, 10), "Lectio M101");
        // Otherwise, use calculated satnum.
        // For example, day = 15 gives ((15-1)/7)+1 = 3; assume version "1963" so include satnum.
        assert_eq!(get_c10_readingname("1963", 10, 15), "Lectio M10 3".replace(" ", ""));
        // We adjust the format: our function returns "Lectio M10" if not 1963.
        assert_eq!(get_c10_readingname("1960", 10, 15), "Lectio M10");
    }

    #[test]
    fn test_lectiones_ex3_fiunt4() {
        let mut scrip = HashMap::new();
        scrip.insert("Lectio1".to_string(), "Text one".to_string());
        scrip.insert("Lectio2".to_string(), "Text two¶\nPart A¶\nPart B".to_string());
        scrip.insert("Lectio3".to_string(), "Text three".to_string());
        // For Lectio1, no splitting
        assert_eq!(lectiones_ex3_fiunt4(&scrip, 1).unwrap(), "Text one");
        // For Lectio2, split into ["Text two", "Part A", "Part B"] – so if we ask for piece 2 (num==2)
        assert_eq!(lectiones_ex3_fiunt4(&scrip, 2).unwrap(), "Part A");
        // For Lectio3, returns full text.
        assert_eq!(lectiones_ex3_fiunt4(&scrip, 3).unwrap(), "Text three");
    }

    #[test]
    fn test_parenthesised_text() {
        // With text shorter than 20 characters, should get "small font" formatting.
        let formatted = parenthesised_text("Short text", "small");
        assert_eq!(formatted, "<small>Short text</small>");
        // With longer text not matching digit+punctuation, wrap in parentheses.
        let paren = parenthesised_text("This is a rather long text without digits", "small");
        assert_eq!(paren, "(This is a rather long text without digits)");
    }

    #[test]
    fn test_beginwith() {
        let input = "Hello world! This is a test.\nNewline here.";
        assert_eq!(beginwith(input), "Hello world!");
        let input2 = "SingleWord";
        assert_eq!(beginwith(input2), "SingleWord");
    }

    #[test]
    fn test_gettype1960() {
        // Some basic tests: these depend on our simple substring checks.
        // Case 1: version contains "196", votive does not contain "C9" or "Defunctorum".
        let t1 = gettype1960("1960", "", "post Nativitatem", "", 3.0, "");
        assert_eq!(t1, LT1960_OCTAVEII);
        let t2 = gettype1960("1960", "", "feria", "", 1.5, "");
        assert_eq!(t2, LT1960_FERIAL);
        let t3 = gettype1960("1960", "", "dominica semiduplex", "", 3.0, "Pasc1-0");
        assert_eq!(t3, LT1960_SUNDAY);
        let t4 = gettype1960("1960", "", "whatever", "", 4.0, "");
        assert_eq!(t4, LT1960_SANCTORAL);
        // Case 2: monastic branch.
        let t5 = gettype1960("monastic", "", "feria", "", 1.0, "");
        assert_eq!(t5, LT1960_FERIAL);
        let t6 = gettype1960("monastic", "", "infra octavam", "", 3.0, "");
        assert_eq!(t6, LT1960_OCTAVE);
        let t7 = gettype1960("monastic", "", "whatever", "", 3.5, "");
        assert_eq!(t7, LT1960_SANCTORAL);
        // Finally, if rule contains "9 lectiones 1960" then default.
        let t8 = gettype1960("1960", "", "anything", "9 lectiones 1960", 1.0, "");
        assert_eq!(t8, LT1960_DEFAULT);
    }
}
