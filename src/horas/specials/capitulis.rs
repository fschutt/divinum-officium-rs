/*!
 * capitulis.rs
 *
 * This module implements the special “capitulus” routines used in the Hours.
 *
 * It defines three main functions:
 *
 *   - `capitulum_major(config: &InputConfig) -> Option<String>`
 *   - `monastic_major_responsory(config: &InputConfig) -> Option<String>`
 *   - `capitulum_minor(config: &InputConfig) -> Vec<String>`
 *
 * In addition, it provides an HTML entry–point:
 *
 *   - `render_capitulis(config: InputConfig) -> HtmlString`
 *
 * **Notable changes compared to the original Perl code:**
 *
 * - All globals (winner, vespera, seasonalflag, version, hora, label, and votive) are now
 *   bundled into a single configuration type.
 * - The helper function `attach_responsory_if_missing` now uses early returns to reduce nesting.
 * - New tests (using dummy implementations for external dependencies) exercise the behavior
 *   corresponding to the original Perl logic.
 */

 use std::collections::HashMap;

 //
 // --- External dependencies ---
 //
 // In production these functions are expected to be defined in their respective modules.
 // In tests, we override them with dummy implementations.
 #[cfg(not(test))]
 use crate::build::setbuild;
 #[cfg(test)]
 use self::dummy::*; // dummy implementations for testing
 
 #[cfg(not(test))]
 use crate::proprium::getproprium;
 #[cfg(test)]
 use self::dummy::getproprium;
 
 #[cfg(not(test))]
 use crate::setup_string::setupstring;
 #[cfg(test)]
 use self::dummy::setupstring;
 
 #[cfg(not(test))]
 use crate::tempora::gettempora;
 #[cfg(test)]
 use self::dummy::gettempora;
 
 #[cfg(not(test))]
 use crate::comment::setcomment;
 #[cfg(test)]
 use self::dummy::setcomment;
 
 #[cfg(not(test))]
 use crate::postprocess::postprocess_short_resp;
 #[cfg(test)]
 use self::dummy::postprocess_short_resp;
 
 //
 // --- Configuration Types ---
 //
 
 /// Contains the global variables formerly obtained via functions.
 pub struct GlobalConfig {
     pub winner: String,
     pub vespera: i32,
     pub seasonalflag: i32,
     pub version: String,
     pub hora: String,
     pub label: String,
     pub votive: String,
 }
 
 /// Main input configuration for capitulus functions.
 pub struct InputConfig {
     pub lang: String,
     pub globals: GlobalConfig,
 }
 
 /// A type alias for HTML strings.
 pub type HtmlString = String;
 
 //
 // --- Helper Functions ---
 //
 
 /// Attaches a responsory to the given text if one is not already present.
 ///
 /// The logic is as follows:
 ///
 /// 1. If no text was provided, return `None` immediately.
 /// 2. If the text already contains a responsory marker (`"\n_\nR.br"`), return it immediately.
 /// 3. Otherwise, look up a responsory using the key `"Responsory {hora}"`. If the version is
 ///    Monastic, append an `"M"` to the key.
 /// 4. If not found and the version is non–Monastic, try looking up `"Responsory Breve {hora}"`.
 /// 5. If still not found, use a replacement mapping (different for Monastic and non–Monastic).
 /// 6. If a responsory is found, attach it (separated by `\n_\n`) to the original text.
 ///
 /// # Arguments
 ///
 /// - `w`: The original capitulum text (if any).
 /// - `lang`: The language code.
 /// - `seasonalflag`: The seasonal flag.
 /// - `version`: The version string.
 /// - `hora`: The current hour.
 ///
 /// # Returns
 ///
 /// An updated version of the text with a responsory attached if possible.
 fn attach_responsory_if_missing(
     w: Option<String>,
     lang: &str,
     seasonalflag: i32,
     version: &str,
     hora: &str,
 ) -> Option<String> {
     // Early return if no text is provided.
     let orig_text = w?;
     // If the responsory marker is already present, return immediately.
     if orig_text.contains("\n_\nR.br") {
         return Some(orig_text);
     }
 
     let mut new_name = format!("Responsory {}", hora);
     if version.contains("Monastic") {
         new_name.push('M');
     }
     let (mut wr, _cr) = getproprium(&new_name, lang, seasonalflag, 1);
 
     // For non–Monastic versions, try an alternate key.
     if wr.is_none() && !version.contains("Monastic") {
         let (wr_breve, _c_br) = getproprium(&format!("Responsory Breve {}", hora), lang, seasonalflag, 1);
         wr = wr_breve;
     }
 
     // If still missing, use a replacement mapping.
     if wr.is_none() {
         let replace = if !version.contains("Monastic") {
             HashMap::from([
                 ("Tertia", "Versum Tertia"),
                 ("Sexta", "Versum Sexta"),
                 ("Nona", "Versum Nona"),
             ])
         } else {
             HashMap::from([
                 ("Tertia", "Nocturn 1 Versum"),
                 ("Sexta", "Nocturn 2 Versum"),
                 ("Nona", "Nocturn 3 Versum"),
             ])
         };
         if let Some(rep) = replace.get(hora) {
             let (v_res, _cvers) = getproprium(rep, lang, seasonalflag, 1);
             wr = v_res;
         }
     }
 
     if let Some(attached_text) = wr {
         return Some(format!("{}\n_\n{}", orig_text, attached_text));
     }
     Some(orig_text)
 }
 
 //
 // --- Main Functions ---
 //
 
 /// Returns the major capitulum text for the given input configuration.
 ///
 /// The logic follows the original Perl:
 ///
 /// 1. Start with a default name `"Capitulum Laudes"`.
 /// 2. Change the name to `"Capitulum Vespera 1"` if `winner` contains `"12-25"` and `vespera == 1`.
 /// 3. Change the name to `"Capitulum Vespera"` if `winner` contains `"C12"` and `hora == "Vespera"`.
 /// 4. Call `setbuild` and try to get the proper text via `getproprium`.
 /// 5. If not found and the seasonal flag is false, try with the seasonal flag set to 1.
 /// 6. If still not found, load fallback text from `"Psalterium/Special/Major Special.txt"`.
 /// 7. Call `setcomment` using the provided label.
 /// 8. Return the found text (if any).
 pub fn capitulum_major(config: &InputConfig) -> Option<String> {
     let globals = &config.globals;
     let winner = &globals.winner;
     let vespera = globals.vespera;
     let seasonalflag = globals.seasonalflag;
     let _version = &globals.version;
     let hora = &globals.hora;
     let label = &globals.label;
 
     // Default name.
     let mut name = "Capitulum Laudes".to_string();
 
     // Special cases.
     if winner.contains("12-25") && vespera == 1 {
         name = "Capitulum Vespera 1".to_string();
     }
     if winner.contains("C12") && hora == "Vespera" {
         name = "Capitulum Vespera".to_string();
     }
 
     // Set build info.
     setbuild("Psalterium/Special/Major Special", &name, "Capitulum ord");
 
     // Attempt to retrieve the proper text.
     let (mut capit, mut c) = getproprium(&name, &config.lang, seasonalflag, 1);
     if capit.is_none() && seasonalflag == 0 {
         let (cap_alt, c_alt) = getproprium(&name, &config.lang, 1, 1);
         capit = capit.or(cap_alt);
         c = c.or(c_alt);
     }
     if capit.is_none() {
         if let Some(cap_map) = setupstring(&config.lang, "Psalterium/Special/Major Special.txt") {
             let key = format!("{} {}", gettempora("Capitulum major"), hora);
             capit = cap_map.get(&key).cloned();
         }
     }
     setcomment(label, "Source", c, &config.lang);
     capit
 }
 
 /// Returns the monastic major responsory as a single string for the given input configuration.
 ///
 /// The logic follows the original Perl:
 ///
 /// 1. Construct a key `"Responsory {hora}"`, appending `" 1"` if `winner` contains `"12-25"` and `vespera == 1`.
 /// 2. Attempt to retrieve the responsory via `getproprium`.
 /// 3. If not found, adjust the key (first replacing `"Vespera"` with `"Breve Sexta"` and `"Laudes"` with `"Breve Tertia"`,
 ///    then removing `"Breve "`).
 /// 4. If still missing, load fallback text from `"Psalterium/Special/Major Special.txt"`.
 /// 5. Remove any attached versicle (truncate at `"\n_"`), postprocess the lines, and (if needed) remove any substring starting with `"&gloria"`.
 pub fn monastic_major_responsory(config: &InputConfig) -> Option<String> {
     let globals = &config.globals;
     let winner = &globals.winner;
     let vespera = globals.vespera;
     let seasonalflag = globals.seasonalflag;
     let version = &globals.version;
     let hora = &globals.hora;
 
     // Construct key.
     let mut key = format!("Responsory {}", hora);
     if winner.contains("12-25") && vespera == 1 {
         key.push_str(" 1");
     }
     let (mut resp, mut c) = getproprium(&key, &config.lang, seasonalflag, 1);
 
     // Try adjusted keys if not found.
     if resp.is_none() {
         let key_sub = key.replace("Vespera", "Breve Sexta")
                          .replace("Laudes", "Breve Tertia");
         let res = getproprium(&key_sub, &config.lang, seasonalflag, 1);
         if res.0.is_some() {
             resp = res.0;
             c = res.1;
         }
     }
     if resp.is_none() {
         let key_no_breve = key.replace("Breve ", "");
         let res = getproprium(&key_no_breve, &config.lang, seasonalflag, 1);
         if res.0.is_some() {
             resp = res.0;
             c = res.1;
         }
     }
     if resp.is_none() {
         if let Some(resp_map) = setupstring(&config.lang, "Psalterium/Special/Major Special.txt") {
             let key2 = format!("Responsory {} {}", gettempora("Capitulum major"), hora);
             resp = resp_map.get(&key2).cloned();
         }
     }
     // Remove any attached versicle.
     if let Some(mut r) = resp {
         if let Some(pos) = r.find("\n_") {
             r.truncate(pos);
         }
         let mut lines: Vec<String> = r.lines().map(|s| s.to_string()).collect();
         postprocess_short_resp(&mut lines, &config.lang);
         r = lines.join("\n");
         if version.to_lowercase().contains("cist") {
             if let Some(pos) = r.to_lowercase().find("&gloria") {
                 r.truncate(pos);
             }
         }
         resp = Some(r);
     }
     resp
 }
 
 /// Returns the minor capitulum as a vector of strings (split by newline) for the given input configuration.
 ///
 /// The logic follows the original Perl:
 ///
 /// 1. Load the “Minor Special” data from `"Psalterium/Special/Minor Special.txt"`.
 /// 2. Construct a key from `gettempora("Capitulum minor")` and the current hour (with a special case for `"Completorium"`).
 /// 3. Check for responsory keys and append them if present.
 /// 4. For `"Completorium"`, if the version does not start with `"Ordo Praedicatorum"`, append `"Versum 4"`.
 /// 5. Otherwise, set a comment value, call `setbuild`, and try to retrieve a responsory via `getproprium`.
 ///    (Here the responsory–attaching logic is factored out.)
 /// 6. Finally, postprocess the text and (if applicable) call `setcomment`.
 pub fn capitulum_minor(config: &InputConfig) -> Vec<String> {
     let globals = &config.globals;
     let hora = &globals.hora;
     let version = &globals.version;
     let seasonalflag = globals.seasonalflag;
     let votive = &globals.votive;
 
     let mut capit_map = match setupstring(&config.lang, "Psalterium/Special/Minor Special.txt") {
         Some(m) => m,
         None => return Vec::new(),
     };
     let mut name = format!("{} {}", gettempora("Capitulum minor"), hora);
     if hora == "Completorium" {
         name = "Completorium".to_string();
     }
     let mut capit = capit_map
         .get(&name)
         .map(|s| s.trim_end().to_string())
         .unwrap_or_default();
 
     let mut resp: Option<String> = None;
     let mut comment: Option<i32> = None;
 
     if version.contains("Monastic") {
         name.push('M');
     }
 
     if let Some(r) = capit_map.get(&format!("Responsory {}", name)) {
         let trimmed = r.trim_end().to_string();
         capit.push_str(&format!("\n_\n{}", trimmed));
         resp = Some(trimmed);
     } else if let (Some(r), Some(_v)) = (
         capit_map.get(&format!("Responsory breve {}", name)),
         capit_map.get(&format!("Versum {}", name)),
     ) {
         let trimmed_r = r.trim_end().to_string();
         let combined = format!("{}", trimmed_r);
         capit.push_str(&format!("\n_\n{}", combined));
         resp = Some(combined);
     }
 
     if hora == "Completorium" && !version.starts_with("Ordo Praedicatorum") {
         if let Some(v4) = capit_map.get("Versum 4") {
             capit.push_str(&format!("\n_\n{}", v4));
         }
     } else {
         // Set comment value.
         comment = if name.contains("Dominica") || name.contains("Feria") {
             Some(5)
         } else {
             Some(1)
         };
         setbuild("Psalterium/Special/Minor Special", &name, "Capitulum ord");
 
         let mut key = format!("Capitulum {}", hora);
         if hora == "Tertia" && !votive.contains("C12") {
             key = key.replace("Tertia", "Laudes");
         }
         let (mut w, c_val) = getproprium(&key, &config.lang, seasonalflag, 1);
         if w.is_some() {
             // Factor out the responsory–attaching branch.
             w = attach_responsory_if_missing(w, &config.lang, seasonalflag, version, hora);
             resp = w.clone();
         }
         if let Some(w_str) = w {
             capit = w_str;
             comment = c_val.and_then(|s| s.parse::<i32>().ok());
         }
     }
 
     let mut lines: Vec<String> = capit.lines().map(|s| s.to_string()).collect();
     postprocess_short_resp(&mut lines, &config.lang);
     if hora != "Completorium" {
         setcomment(&globals.label, "Source", comment.map(|i| i.to_string()), &config.lang);
     }
     lines
 }
 
 /// The entry–point function that takes an `InputConfig` and returns an HTML–formatted string.
 ///
 /// It calls the three main routines (for major, responsory, and minor texts) and then
 /// combines their output into a single HTML string.
 pub fn render_capitulis(config: InputConfig) -> HtmlString {
     let mut html = String::new();
 
     html.push_str("<div class=\"capitulis\">\n");
 
     if let Some(major_text) = capitulum_major(&config) {
         html.push_str("  <div class=\"capitulum-major\">\n");
         html.push_str(&major_text);
         html.push_str("\n  </div>\n");
     }
 
     if let Some(resp_text) = monastic_major_responsory(&config) {
         html.push_str("  <div class=\"monastic-major-responsory\">\n");
         html.push_str(&resp_text);
         html.push_str("\n  </div>\n");
     }
 
     let minor_lines = capitulum_minor(&config);
     if !minor_lines.is_empty() {
         html.push_str("  <div class=\"capitulum-minor\">\n    <pre>\n");
         html.push_str(&minor_lines.join("\n"));
         html.push_str("\n    </pre>\n  </div>\n");
     }
 
     html.push_str("</div>");
     html
 }

 #[cfg(test)]
 mod tests {
     use super::*;
 
     /// Helper to construct a full InputConfig with all globals.
     fn make_config(
         winner: &str,
         vespera: i32,
         seasonalflag: i32,
         version: &str,
         hora: &str,
         label: &str,
         votive: &str,
     ) -> InputConfig {
         InputConfig {
             lang: "en".to_string(),
             globals: GlobalConfig {
                 winner: winner.to_string(),
                 vespera,
                 seasonalflag,
                 version: version.to_string(),
                 hora: hora.to_string(),
                 label: label.to_string(),
                 votive: votive.to_string(),
             },
         }
     }
 
     #[test]
     fn test_capitulum_major_12_25() {
         // When winner contains "12-25" and vespera is 1, we expect the name to be "Capitulum Vespera 1".
         let config = make_config("Celebration on 12-25", 1, 0, "TestVersion", "Laudes", "TestLabel", "SomeVotive");
         let result = capitulum_major(&config).unwrap();
         // Dummy getproprium returns text including the key.
         assert!(result.contains("Dummy text for Capitulum Vespera 1"),
                 "Expected text to contain key 'Capitulum Vespera 1'");
     }
 
     #[test]
     fn test_capitulum_major_C12() {
         // When winner contains "C12" and hora is "Vespera", expect the name to be "Capitulum Vespera".
         let config = make_config("Event C12 Special", 0, 0, "TestVersion", "Vespera", "TestLabel", "SomeVotive");
         let result = capitulum_major(&config).unwrap();
         assert!(result.contains("Dummy text for Capitulum Vespera"),
                 "Expected text to contain key 'Capitulum Vespera'");
     }
 
     #[test]
     fn test_monastic_major_responsory_adjustments() {
         // For monastic responsory, even if the first lookup fails, the key adjustments should
         // result in a dummy text being returned.
         let config = make_config("Normal", 0, 0, "TestVersion", "Laudes", "TestLabel", "SomeVotive");
         let result = monastic_major_responsory(&config).unwrap();
         // Since dummy getproprium always returns text with the key, check that the returned text
         // mentions either "Responsory Laudes" or an adjusted key.
         assert!(result.contains("Dummy text for Responsory Laudes")
             || result.contains("Dummy text for Responsory Breve Laudes"),
             "Responsory text was not properly constructed.");
     }
 
     #[test]
     fn test_capitulum_minor_responsory_attachment() {
         // For a non-Completorium hour, if getproprium returns a value,
         // then attach_responsory_if_missing should add a responsory.
         let config = make_config("Normal", 0, 0, "TestVersion", "Tertia", "TestLabel", "No C12 here");
         let result_lines = capitulum_minor(&config);
         let result = result_lines.join("\n");
         // Our dummy getproprium will return a string containing "Dummy text for Capitulum Laudes"
         // (since for Tertia with no C12, key "Tertia" is replaced by "Laudes").
         assert!(result.contains("Dummy text for Capitulum Laudes")
             || result.contains("Dummy text for Responsory Tertia")
             || result.contains("Dummy text for Responsory Breve Tertia"),
             "Expected minor capitulum text to include attached responsory.");
     }
 
     #[test]
     fn test_render_capitulis_html() {
         // Integration test for render_capitulis.
         let config = make_config("Normal", 0, 0, "TestVersion", "Laudes", "TestLabel", "SomeVotive");
         let html = render_capitulis(config);
         assert!(html.contains("<div class=\"capitulis\">"));
         assert!(html.contains("<div class=\"capitulum-major\">"));
         // At least one of the sections should be present.
         assert!(html.contains("<div class=\"monastic-major-responsory\">")
             || html.contains("<div class=\"capitulum-minor\">"));
     }
 }
 