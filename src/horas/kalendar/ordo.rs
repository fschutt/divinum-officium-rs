//! ordo.rs
//!
//! This module produces output for the Ordinarium (Ordo) used in the kalendar.
//! It builds one–day entries (via `ordo_entry()`), table rows, and an HTML header.
//!
//! Instead of using global variables, we pass in necessary parameters via two
//! context structs: `OrdoContext` and `HtmlHeaderContext`. All string
//! substitutions are implemented using built–in string methods (no regexes).

use std::collections::HashMap;
use std::mem;
use crate::liturgical_color;

/// Context for constructing one–day Ordo entries.
pub struct OrdoContext {
    pub version: String,            // e.g. "Rubrics 1960"
    pub day: i32,                   // numeric day
    pub month: i32,                 // numeric month
    pub year: i32,                  // numeric year
    pub daynames: Vec<String>,      // day names; index 0 is first part, index 2 used for cv, etc.
    pub commemoentries: Vec<String>,// list of commemo entries filenames (without extension)
    pub headline: String,           // headline string, e.g. "Headline1 ~ Headline2"
    pub winner: String,             // winner string
    pub winner_map: HashMap<String, String>, // mapping for winner (e.g. Lectio Prima)
    pub initia: bool,               // flag whether to append " *I*"
    pub hora: String,               // current hour, e.g. "Laudes"
    pub smallblack: String,         // e.g. "black"
    pub smallfont: String,          // e.g. "small"
}

/// Context for building the HTML header.
pub struct HtmlHeaderContext {
    pub version: String,            // version string for display
    pub kmonth: usize,              // current month (1..12)
    pub kyear: i32,                 // current year
    pub monthnames: Vec<String>,    // e.g. ["", "January", "February", …, "December"]
}

/// Splits the headline string (from setheadline) on the literal '~' and trims whitespace.
fn get_headline_parts(headline: &str) -> (String, String) {
    let parts: Vec<&str> = headline.split('~').map(|s| s.trim()).collect();
    if parts.len() >= 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        (headline.to_string(), String::new())
    }
}

/// Formats the first column (c1) using the given headline parts.
/// Applies setfont and then performs literal substitutions.
fn format_c1(h1: &str, h2: &str, ctx: &OrdoContext) -> String {
    let mut s = format!(
        "<B>{}</B>{}",
        setfont(&liturgical_color(h1), h1),
        setfont("1 maroon", &format!("&ensp;{}", h2))
    );
    s = s.replace("Hebdomadam", "Hebd");
    s = s.replace("Quadragesima", "Quadr");
    s
}

/// Splits the third element of daynames on the literal ": " into two parts.
/// If splitting yields two parts, returns (formatted, part1, part2); otherwise returns ("", "", first part).
fn format_c2(daynames: &[String], _ctx: &OrdoContext) -> (String, String, String) {
    if daynames.len() < 3 {
        return (String::new(), String::new(), String::new());
    }
    let c2_raw = &daynames[2];
    if let Some(pos) = c2_raw.find(": ") {
        let part1 = &c2_raw[..pos];
        let part2 = &c2_raw[pos + 2..];
        let mut c2 = setfont("smallblack", &format!("{}:", part1));
        c2.push_str(&format!(
            "<I>{}</I>",
            setfont(&liturgical_color(part2), &format!(" {}", part2))
        ));
        (c2, part1.to_string(), part2.to_string())
    } else {
        (String::new(), String::new(), c2_raw.to_string())
    }
}

/// For each additional commemo entry (if any), load the file "<entry>.txt" via setupstring (for language "Latin"),
/// then extract the "Rank" field up to the first occurrence of ";;" and append it (with formatting) to c2.
fn append_commemo_entries(c2: &mut String, commemoentries: &[String], lang: &str) {
    if commemoentries.len() <= 1 {
        return;
    }
    for ind in 1..commemoentries.len() {
        let filename = format!("{}.txt", commemoentries[ind]);
        if let Some(com_map) = setupstring("Latin", &filename, ResolveDirectives::None) {
            if let Some(comname_raw) = com_map.get("Rank") {
                let comname = if let Some(pos) = comname_raw.find(";;") {
                    &comname_raw[..pos]
                } else {
                    comname_raw
                };
                if !comname.is_empty() {
                    let appended = format!(
                        " <I>&amp; {}</I>",
                        setfont(&liturgical_color(comname), &format!(" {}", comname))
                    );
                    c2.push_str(&appended);
                }
            }
        }
    }
}

/// Possibly appends " *L1*" to c1 if:
/// - version does not contain "196"
/// - winner (case-insensitively) contains "sancti"
/// - winner_map has key "Lectio Prima" whose value does not contain "@Commune" nor patterns like "!Matt ..." (using simple substring checks)
fn maybe_append_l1(c1: &mut String, version: &str, winner: &str, winner_map: &HashMap<String, String>, smallfont: &str) {
    if !(!version.contains("196")
        && winner.to_lowercase().contains("sancti")
        && winner_map.contains_key("Lectio Prima"))
    {
        return;
    }

    let lectio = winner_map.get("Lectio Prima").unwrap();
    let lectio_lower = lectio.to_lowercase();
    if !(!lectio.contains("@Commune")
        && !lectio_lower.contains("!matt")
        && !lectio_lower.contains("!marc")
        && !lectio_lower.contains("!luc")
        && !lectio_lower.contains("!joannes"))
    {
        return;
    }

    c1.push_str(&setfont(smallfont, " *L1*"));
}

/// If the date (first five characters) is between "01-13" and "12-24", then if winner (case‑insensitively)
/// contains "sancti", swap c1 and c2; otherwise, clear c2 unless it contains "Commemoratio" or "Scriptura".
fn maybe_swap_or_clear_columns(date: &str, c1: &mut String, c2: &mut String, winner: &str) {
    let date_prefix = &date[0..5];
    if date_prefix < "12-24" && date_prefix > "01-13" {
        if winner.to_lowercase().contains("sancti") {
            mem::swap(c1, c2);
        }
    } else {
        if !(c2.contains("Commemoratio") || c2.contains("Scriptura")) {
            *c2 = String::new();
        }
    }
}

/// Appends extra strings to c1 based on whether a dirge should be said and whether initia is true.
/// (This function calls the helper `dirge()` below.)
fn append_dirge_and_initia(
    c1: &mut String, 
    version: &str, 
    day: i32, 
    month: i32, 
    year: i32, 
    initia: bool, 
    smallblack: &str, 
    smallfont: &str
) {
    if dirge(version, "Laudes", day as u32, month as u32, year) {
        c1.push_str(&setfont(smallblack, " dirge"));
    }
    if !version.contains("1960") && initia {
        c1.push_str(&setfont(smallfont, " *I*"));
    }
}

/// Computes capitulo / vespera by taking the third element of daynames 
/// and returning the substring starting at the first occurrence 
/// (case-insensitively) of either "vespera" or "a capitulo".
fn compute_cv(daynames: &[String]) -> String {
    if daynames.len() < 3 {
        return String::new();
    }
    let cv = &daynames[2];
    let lower = cv.to_lowercase();
    if let Some(pos) = lower.find("vespera") {
        return cv[pos..].trim().to_string();
    }
    if let Some(pos) = lower.find("a capitulo") {
        return cv[pos..].trim().to_string();
    }
    String::new()
}

/// The main ordo_entry function.
/// 
/// Returns a tuple (c1, c2, cv). All needed data is passed in via the OrdoContext and as arguments.
/// 
/// If winneronly is true, returns the headline parts joined by a comma.
pub fn ordo_entry(ctx: &OrdoContext, date: &str, compare: bool, winneronly: bool) -> (String, String, String) {
    let (headline1, headline2) = get_headline_parts(&ctx.headline);
    if winneronly {
        return (format!("{}, {}", headline1, headline2), String::new(), String::new());
    }
    let mut c1 = format_c1(&headline1, &headline2, ctx);
    let (mut c2, _, _) = format_c2(&ctx.daynames, ctx);
    append_commemo_entries(&mut c2, &ctx.commemoentries, "Latin");
    c2 = c2.replace("Hebdomadam", "Hebd").replace("Quadragesima", "Quadr");
    maybe_append_l1(&mut c1, &ctx.version, &ctx.winner, &ctx.winner_map, &ctx.smallfont);
    maybe_swap_or_clear_columns(date, &mut c1, &mut c2, &ctx.winner);
    append_dirge_and_initia(&mut c1, &ctx.version, ctx.day, ctx.month, ctx.year, ctx.initia, &ctx.smallblack, &ctx.smallfont);
    let mut cv = compute_cv(&ctx.daynames);
    if compare {
        if c2.is_empty() {
            c2 = "_".to_string();
        }
        if cv.is_empty() {
            cv = "_".to_string();
        }
    }
    (c1, c2, cv)
}

/// Prepares one table row for the kalendar.
/// 
/// Returns a 5-tuple: (link for day number, c1, c2, cv (in small font), day name).
pub fn table_row(ctx: &OrdoContext, date: &str, compare: bool, version1: &str, version2: &str, domlet_counter: i32, dayofweek: usize) -> (String, String, String, String, String) {
    let d: i32 = date.get(3..5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let (mut c1, mut c2, mut cv) = ordo_entry(ctx, date, compare, false);
    if compare {
        let (c21, c22, cv2) = ordo_entry(ctx, date, compare, false); // In a complete implementation version2 might be used differently.
        c1 = format!("{}<br/>{}", c1, c21);
        c2 = format!("{}<br/>{}", c2, c22);
        cv = format!("{}<br/>{}", cv, cv2);
    }
    let link = format!(r#"<A HREF="#" onclick=\"callbrevi('{}');\">{}</A>"#, date, d);
    let cv_font = format!(r#"<FONT SIZE="-2">{}</FONT>"#, cv);
    // Assume the day name for the table row comes from ctx.daynames[dayofweek]
    let dayname = ctx.daynames.get(dayofweek).cloned().unwrap_or_default();
    (link, c1, c2, cv_font, dayname)
}

/// Produces the HTML header for the Ordinarium page using values from the HtmlHeaderContext.
pub fn html_header(ctx: &HtmlHeaderContext) -> String {
    let vers = ctx.version.clone();
    let mut output = String::new();
    output.push_str("<A ID=\"top\"></A>\n");
    output.push_str("<H1>\n");
    output.push_str("<FONT COLOR=\"MAROON\" SIZE=\"+1\"><B><I>Divinum Officium</I></B></FONT>&nbsp;\n");
    output.push_str(&format!("<FONT COLOR=\"RED\" SIZE=\"+1\">{}</FONT>\n", vers));
    output.push_str("</H1>\n");
    output.push_str("<P ALIGN=\"CENTER\">\n");
    output.push_str("<A HREF=\"#\" onclick=\"callbrevi();\">Divinum Officium</A>&nbsp;&ensp;\n");
    output.push_str("<A HREF=\"#\" onclick=\"callmissa();\">Sancta Missa</A>&nbsp;&ensp;\n");
    output.push_str("<A HREF=\"#\" onclick=\"setkm(0);\">Ordo</A>\n");
    output.push_str("</P>\n");
    output.push_str("<P ALIGN=\"CENTER\">\n");
    let mut mmenu = Vec::new();
    if ctx.kmonth == 1 {
        mmenu.push("<A HREF=\"#\" onclick=\"setkm(-1)\">«</A>\n".to_string());
    }
    for i in 1..=12 {
        let mn = &ctx.monthnames[i][..3];
        let line = if i == ctx.kmonth {
            mn.to_string()
        } else {
            format!("<A HREF=\"#\" onclick=\"setkm({})\">{}</A>\n", i, mn)
        };
        mmenu.push(line);
    }
    if ctx.kmonth == 12 {
        mmenu.push("<A HREF=\"#\" onclick=\"setkm(13)\">»</A>\n".to_string());
    }
    output.push_str(&mmenu.join(&"&nbsp;".repeat(3)));
    output.push_str("</P>\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn dummy_ordo_context() -> OrdoContext {
        OrdoContext {
            version: "TestVersion".to_string(),
            day: 15,
            month: 3,
            year: 2024,
            daynames: vec![
                "FirstPart".to_string(),
                "Unused".to_string(),
                "Morning: Vespera extra".to_string(),
                "Extra".to_string(),
            ],
            commemoentries: vec!["Entry1".to_string(), "Entry2".to_string()],
            headline: "Headline1 ~ Headline2".to_string(),
            winner: "Sancti something".to_string(),
            winner_map: {
                let mut hm = HashMap::new();
                hm.insert("Lectio Prima".to_string(), "SomeLectio".to_string());
                hm
            },
            initia: true,
            hora: "Laudes".to_string(),
            smallblack: "black".to_string(),
            smallfont: "small".to_string(),
        }
    }

    fn dummy_html_header_context() -> HtmlHeaderContext {
        HtmlHeaderContext {
            version: "TestVersion".to_string(),
            kmonth: 3,
            kyear: 2024,
            monthnames: crate::MONTH_NAMES.iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_get_headline_parts() {
        let (h1, h2) = get_headline_parts("Test Headline ~ Second Part");
        assert_eq!(h1, "Test Headline");
        assert_eq!(h2, "Second Part");
    }

    #[test]
    fn test_format_c1() {
        let ctx = dummy_ordo_context();
        let s = format_c1("Hebdomadam", "Quadragesima", &ctx);
        assert!(s.contains("Hebd"));
        assert!(s.contains("Quadr"));
    }

    #[test]
    fn test_format_c2() {
        let ctx = dummy_ordo_context();
        let (c2, part1, part2) = format_c2(&ctx.daynames, &ctx);
        assert!(!c2.is_empty());
        // Given our dummy daynames[2] "Morning: Vespera extra"
        assert_eq!(part1, "Morning");
        assert_eq!(part2, "Vespera extra");
    }

    #[test]
    fn test_append_commemo_entries() {
        let mut s = "Test".to_string();
        append_commemo_entries(&mut s, &vec!["Entry1".to_string(), "Entry2".to_string()], "Latin");
        assert!(s.contains("&amp;"));
    }

    #[test]
    fn test_maybe_append_l1() {
        let mut s = "Test".to_string();
        let mut hm = HashMap::new();
        hm.insert("Lectio Prima".to_string(), "SomeLectio".to_string());
        maybe_append_l1(&mut s, "TestVersion", "Sancti something", &hm, "small");
        assert!(s.contains("*L1*"));
    }

    #[test]
    fn test_maybe_swap_or_clear_columns() {
        let mut c1 = "C1".to_string();
        let mut c2 = "C2".to_string();
        maybe_swap_or_clear_columns("02-15-2024", &mut c1, &mut c2, "sancti");
        assert_eq!(c1, "C2");
        assert_eq!(c2, "C1");
        let mut c1 = "C1".to_string();
        let mut c2 = "Other".to_string();
        maybe_swap_or_clear_columns("12-25-2024", &mut c1, &mut c2, "not");
        assert_eq!(c2, "");
    }

    #[test]
    fn test_append_dirge_and_initia() {
        let mut s = "Test".to_string();
        append_dirge_and_initia(&mut s, "TestVersion", 15, 3, 2024, true, "black", "small");
        assert!(s.contains("*I*"));
    }

    #[test]
    fn test_compute_cv() {
        let daynames = vec![
            "Something".to_string(),
            "Middle".to_string(),
            "Before Vespera extra".to_string(),
        ];
        let cv = compute_cv(&daynames);
        assert_eq!(cv.to_lowercase(), "vespera extra".to_string());
    }

    #[test]
    fn test_ordo_entry() {
        let ctx = dummy_ordo_context();
        let (c1, c2, cv) = ordo_entry(&ctx, "03-15-2024", false, false);
        assert!(!c1.is_empty());
        assert!(!c2.is_empty());
        assert_eq!(cv.to_lowercase(), "vespera extra".to_string());
    }

    #[test]
    fn test_table_row() {
        let ctx = dummy_ordo_context();
        let (link, c1, c2, cv_font, dayname) = table_row(&ctx, "05-12-2023", false, "v1", "v2", 2, 1);
        assert!(link.contains("12"));
        assert!(!dayname.is_empty());
    }

    #[test]
    fn test_html_header() {
        let ctx = dummy_html_header_context();
        let header = html_header(&ctx);
        assert!(header.contains("Divinum Officium"));
        assert!(header.contains("Kalendarium"));
        assert!(header.contains("Ordo"));
    }
}
