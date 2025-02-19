//! This module produces the Appendix output (for example, the Index) for the kalendar.

use std::collections::HashMap;

/// The configuration for producing the appendix output.
pub struct AppendixConfig {
    /// The appendix key. If None, the default "Index" is used.
    pub appendix: Option<String>,
    /// Primary language (for example, "Latin").
    pub lang1: String,
    /// Secondary language (for example, "English").
    pub lang2: String,
    /// The version string.
    pub version: String,
    /// Version string for the primary office.
    pub version1: String,
    /// Version string for the secondary office.
    pub version2: String,
    /// If true, only one column of output is produced.
    pub only: bool,
    /// An additional parameter (e.g. for expansion index).
    pub expandind: u32,
    /// The column indicator (not used in our output but provided for compatibility).
    pub column: u32,
    /// A function pointer to a function that, given a language, filename, and parameters,
    /// returns an optional mapping from section keys to text.
    pub setupstring: fn(&str, &str, &[&str]) -> Option<HashMap<String, String>>,
    /// A function pointer that processes a script (a vector of lines) for a given language.
    pub specials: fn(&[String], &str) -> Vec<String>,
    /// A function pointer that combines two scripts (for two languages) into an HTML string.
    pub print_content:
        fn(&str, &[String], &str, &[String]) -> String,
}

/// Removes a case–insensitive leading "appendix " from the input string.
fn strip_appendix_prefix(s: &str) -> String {
    let lower = s.to_lowercase();
    if lower.starts_with("appendix ") {
        s[8..].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

/// The main public function that produces the appendix HTML output.
/// It takes an `AppendixConfig` structure and returns the HTML as a String.
pub fn appendix(config: &AppendixConfig) -> String {

    // The process is as follows:
    //
    // 1. If an appendix key is provided (e.g. `"Appendix Contents"`), any 
    //    leading `"appendix "` (case–insensitive)
    //    is removed (so that `"Appendix Contents"` becomes `"Contents"`). 
    //    If no key is provided, the default `"Index"` is used.
    //
    // 2. An HTML header is built that includes an `<H2>` element with an ID based on the appendix key.
    //
    // 3. The filename is computed as `Appendix/{appendix}.txt`.
    //
    // 4. Using the provided `setupstring` function pointer, we load the 
    //    file’s contents (expected to be a mapping
    //    from section keys to multiline text). We then split the text for 
    //    the given key into lines and pass it
    //    to the provided `specials` function for additional processing.
    //
    // 5. If the configuration’s `only` flag is false, the same is done 
    //    for the secondary language.
    //
    // 6. Finally, the two sets of lines are combined using the 
    //    provided `print_content` function.

    // Determine the key from the provided appendix argument (or default "Index")
    let app_key = match &config.appendix {
        Some(s) => strip_appendix_prefix(s),
        None => "Index".to_string(),
    };

    // Build the header HTML.
    let mut output = String::new();
    output.push_str(&format!(
        "<H2 ID='{}top'>Appendix - {}</H2>\n",
        app_key, app_key
    ));

    // Build the filename: "Appendix/{appendix}.txt"
    let fname = format!("Appendix/{}.txt", app_key);

    // Load the primary script.
    let script1_raw = match (config.setupstring)(&config.lang1, &fname, &[]) {
        Some(map) => map.get(&app_key).cloned().unwrap_or_default(),
        None => String::new(),
    };
    let mut script1: Vec<String> = script1_raw.lines().map(|s| s.to_string()).collect();
    script1 = (config.specials)(&script1, &config.lang1);

    // Load the secondary script if needed.
    let mut script2: Vec<String> = Vec::new();
    if !config.only {
        script2 = match (config.setupstring)(&config.lang2, &fname, &[]) {
            Some(map) => map.get(&app_key).cloned().unwrap_or_default(),
            None => String::new(),
        }
        .lines()
        .map(|s| s.to_string())
        .collect();
        script2 = (config.specials)(&script2, &config.lang2);
    }

    // Combine the content.
    output.push_str(&(config.print_content)(&config.lang1, &script1, &config.lang2, &script2));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// A dummy implementation of `setupstring` for testing.
    fn dummy_setupstring(lang: &str, filename: &str, _params: &[&str]) -> Option<HashMap<String, String>> {
        let mut map = HashMap::new();
        if filename.contains("Index.txt") {
            // Return a mapping for key "Index".
            map.insert("Index".to_string(), "Line1\nLine2".to_string());
        } else if filename.contains("Contents.txt") {
            map.insert("Contents".to_string(), "Content line 1\nContent line 2".to_string());
        }
        Some(map)
    }

    /// A dummy implementation of `specials` that appends " SPECIAL" to each line.
    fn dummy_specials(script: &[String], _lang: &str) -> Vec<String> {
        script.iter().map(|s| format!("{} SPECIAL", s)).collect()
    }

    /// A dummy implementation of `print_content` that joins the lines with newlines.
    fn dummy_print_content(lang1: &str, script1: &[String], lang2: &str, script2: &[String]) -> String {
        let part1 = script1.join("\n");
        let part2 = script2.join("\n");
        format!(
            "<div lang=\"{}\">\n{}\n</div>\n<div lang=\"{}\">\n{}\n</div>",
            lang1, part1, lang2, part2
        )
    }

    #[test]
    fn test_strip_appendix_prefix() {
        assert_eq!(strip_appendix_prefix("Appendix Index"), "Index".to_string());
        assert_eq!(strip_appendix_prefix("appendix   MySection  "), "MySection".to_string());
        assert_eq!(strip_appendix_prefix("Section"), "Section".to_string());
    }

    #[test]
    fn test_appendix_default() {
        // When no appendix argument is provided, the default "Index" should be used.
        let config = AppendixConfig {
            appendix: None,
            lang1: "Latin".to_string(),
            lang2: "English".to_string(),
            version: "Rubrics 1960 - 1960".to_string(),
            version1: "Rubrics 1960 - 1960".to_string(),
            version2: "Divino Afflatu - 1954".to_string(),
            only: false,
            expandind: 0,
            column: 1,
            setupstring: dummy_setupstring,
            specials: dummy_specials,
            print_content: dummy_print_content,
        };

        let output = appendix(&config);
        assert!(output.contains("<H2 ID='Indextop'>Appendix - Index</H2>"));
        // Check that the primary script (Line1 SPECIAL) and secondary script (Content line 1 SPECIAL) are present.
        assert!(output.contains("Line1 SPECIAL"));
        assert!(output.contains("Content line 1 SPECIAL"));
    }

    #[test]
    fn test_appendix_with_argument_only_true() {
        let config = AppendixConfig {
            appendix: Some("Appendix Contents".to_string()),
            lang1: "Latin".to_string(),
            lang2: "English".to_string(),
            version: "Rubrics 1960 - 1960".to_string(),
            version1: "Rubrics 1960 - 1960".to_string(),
            version2: "Divino Afflatu - 1954".to_string(),
            only: true, // Only one column output
            expandind: 0,
            column: 1,
            setupstring: dummy_setupstring,
            specials: dummy_specials,
            print_content: dummy_print_content,
        };

        let output = appendix(&config);
        // The prefix "Appendix " should be removed so the key becomes "Contents"
        assert!(output.contains("<H2 ID='Contentstop'>Appendix - Contents</H2>"));
        // Since only is true, the secondary column should not appear.
        assert!(output.contains("<div lang=\"Latin\">"));
        assert!(!output.contains("<div lang=\"English\">"));
    }

    #[test]
    fn test_appendix_with_argument_only_false() {
        let config = AppendixConfig {
            appendix: Some("Appendix Details".to_string()),
            lang1: "Latin".to_string(),
            lang2: "English".to_string(),
            version: "Rubrics 1960 - 1960".to_string(),
            version1: "Rubrics 1960 - 1960".to_string(),
            version2: "Divino Afflatu - 1954".to_string(),
            only: false, // Two-column output
            expandind: 0,
            column: 1,
            setupstring: dummy_setupstring,
            specials: dummy_specials,
            print_content: dummy_print_content,
        };

        // For testing, we add a dummy entry for the key "Details" in both languages.
        fn dummy_setup_details(lang: &str, filename: &str, _params: &[&str]) -> Option<HashMap<String, String>> {
            let mut map = HashMap::new();
            if filename.contains("Details.txt") {
                map.insert("Details".to_string(), format!("{} details line 1\n{} details line 2", lang, lang));
            }
            Some(map)
        }

        let config = AppendixConfig {
            appendix: Some("Appendix Details".to_string()),
            lang1: "Latin".to_string(),
            lang2: "English".to_string(),
            version: "Rubrics 1960 - 1960".to_string(),
            version1: "Rubrics 1960 - 1960".to_string(),
            version2: "Divino Afflatu - 1954".to_string(),
            only: false,
            expandind: 0,
            column: 1,
            setupstring: dummy_setup_details,
            specials: dummy_specials,
            print_content: dummy_print_content,
        };

        let output = appendix(&config);
        assert!(output.contains("<H2 ID='Detailstop'>Appendix - Details</H2>"));
        assert!(output.contains("<div lang=\"Latin\">"));
        assert!(output.contains("<div lang=\"English\">"));
        // Check that the dummy text from both languages is present.
        assert!(output.contains("Latin details line 1 SPECIAL"));
        assert!(output.contains("English details line 1 SPECIAL"));
    }
}
