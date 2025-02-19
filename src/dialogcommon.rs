//! dialogcommon.rs
//!
//! This module corresponds to `dialogcommon.pl` from Divinum Officium,
//! handling certain aspects of reading and processing dialog/setup data.
//!
//! In the original Perl, many of these routines rely on global variables
//! and dynamic `eval`. We adapt them here in a more Rust-friendly way,
//! returning parsed results or storing them in structures rather than
//! executing them directly.
//!
//! # Overview
//!
//! - **`get_ini(file)`**: Reads lines from an `.ini` file (each line is
//!   `$var='value'`), returning a mapping of variable names to values.
//!   (In Perl, the code used `eval`, but here we parse them safely.)
//!
//! - **`chompd(s)`**: Equivalent to Perl’s `chomp + s/\r//g`, removing
//!   trailing newlines/`\r`. Returns the trimmed string.
//!
//! - **`get_dialog(name)`**: Looks up an entry in an internal dialog map,
//!   returning either the entire line (comma-separated) or a vector if
//!   you need to split it. In Perl, it used “list/scalar” context; here
//!   we provide a function returning a full `String` plus one returning
//!   a `Vec<String>`. Behind the scenes, it lazy-loads a data file
//!   (either `horas.dialog` or `missa.dialog`) and caches it.
//!
//! - **`get_horas(c9f)`**: Retrieves an array of “hour” labels from the
//!   “horas” entry in the dialog file. If `c9f` is true, restricts the
//!   returned subset to indexes [0,1,6] only, per the original code
//!   logic. Also strips trailing whitespace from the last element.
//!
//! - **`set_runtime_options(name, ... setup calls ...)`**: In the original
//!   Perl, this reads a “parameters” line from the dialog, then calls
//!   `getsetup(...)` from the setup code, merges, and reassigns to certain
//!   global variables. We replicate that logic at a high level. In Rust,
//!   you may need references to the “setup store” or “global config.” We
//!   provide placeholders here to illustrate the approach.
//!
//! - **`version_displayname(version)`**: Given a version identifier, tries
//!   to find a “display name” substring from the “versions” line in the
//!   dialog. If not found, returns the input `version`. This is used
//!   to produce a more user-friendly label on the UI.

use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Global dialog cache using DialogData (default mode "horas")
static DIALOG_DATA: Lazy<Mutex<DialogData>> =
    Lazy::new(|| Mutex::new(DialogData::new("horas")));

/// Mimics the original Perl getdialog($name) function:
/// - On first call, loads the dialog file ("horas.dialog") and caches its contents.
/// - Returns the stored value for `name` split on commas.
pub fn get_dialog(name: &str) -> Vec<String> {
    DIALOG_DATA.lock().unwrap().get_dialog_array(name)
}

/// Reads a “file.ini” containing lines of the form `$var='value'`.
/// Returns a map of variable => value. In the original Perl code,
/// these lines were `eval`ed to define global variables dynamically.
/// Here, we simply parse them safely.
///
/// Any line not matching the `$var='value'` pattern is ignored.
/// Reads a “file.ini” containing lines of the form `$var='value'`.
/// Returns a map of variable => value. Any line not matching the pattern is ignored.
pub fn get_ini(file_path: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    if let Ok(content) = std::fs::read_to_string(file_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            // Skip empty lines or comments.
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((var, value)) = parse_line(trimmed) {
                result.insert(var, value);
            }
        }
    }
    result
}

/// Parses a single line of the form `$var='value'` and returns Some((var, value)) if successful.
///
/// This function mimics the behavior of the regex:
///   ^\$(\w+)\s*=\s*'(.*)'
/// It requires the line to start with a `$`, followed by a word, an equals sign (with
/// optional spaces), and a value enclosed in single quotes. If the line does not match,
/// None is returned.
fn parse_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if !line.starts_with('$') {
        return None;
    }
    // Remove the leading '$'
    let rest = &line[1..];
    // Look for the '=' sign that separates the variable name from the value.
    let eq_index = rest.find('=')?;
    let var_part = rest[..eq_index].trim();
    // Ensure the variable name is nonempty and contains only word characters.
    if var_part.is_empty() || !var_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let var_name = var_part.to_string();

    // Get the part after the '=' sign and trim any whitespace.
    let after_eq = rest[eq_index + 1..].trim();
    // The value must start with a single quote.
    if !after_eq.starts_with('\'') {
        return None;
    }
    // Remove the opening quote.
    let after_quote = &after_eq[1..];
    // Find the last (closing) quote.
    let last_quote_index = after_quote.rfind('\'')?;
    let var_value = &after_quote[..last_quote_index];
    Some((var_name, var_value.to_string()))
}

/// Removes trailing newlines and `\r` from a string.
///
/// Equivalent to Perl’s `chomp($a); $a =~ s/\r//g;`
///
/// ```
/// # use divinum_officium::dialogcommon::chompd;
/// let s = chompd("Hello\r\n");
/// assert_eq!(s, "Hello");
/// ```
pub fn chompd(s: &str) -> String {
    // In Perl: chomp + remove \r
    let mut trimmed = s.trim_end_matches(&['\n', '\r'][..]).to_string();
    trimmed = trimmed.replace('\r', "");
    trimmed
}

/// A structure to hold the cached dialog lines. In Perl, `%_dialog` was used,
/// and we stored a “loaded” flag to indicate it had been read. In Rust, we can
/// keep an `Option<HashMap<String,String>>` or store the map and track a boolean.
pub struct DialogData {
    loaded: bool,
    data: HashMap<String, String>,
    /// E.g. either “missa” or “horas”
    pub mode: String,
}

impl DialogData {
    /// Create an empty structure. You must call `load_dialog` to fill it.
    pub fn new(mode: &str) -> Self {
        Self {
            loaded: false,
            data: HashMap::new(),
            mode: mode.to_string(),
        }
    }

    /// Loads the `.dialog` file (e.g. `horas.dialog` or `missa.dialog`) via
    /// the `setupstring` approach. In Perl, this used `setupstring('', "$1.dialog")`.
    ///
    /// For demonstration, we mimic that by simply reading a plain text file
    /// named e.g. `mode.dialog` from some data folder. In Divinum Officium,
    /// that typically is `$datafolder/horas/horas.dialog` or `$datafolder/missa/missa.dialog`.
    pub fn load_dialog(&mut self, full_path: &str) {
        if self.loaded {
            return;
        }
        // In Perl, we do: %_dialog = %{ setupstring('', "$1.dialog") };
        // then chomp all lines. We'll approximate that by reading each line
        // as key => value. The real file might have multiple lines, or
        // sections. This is a partial mock. In the real code, you'd integrate
        // with `setupstring` from SetupString.
        if let Ok(content) = std::fs::read_to_string(full_path) {
            // Suppose each line is "key=value" for demonstration. The real
            // "horas.dialog" might have a different structure.
            for line in content.lines() {
                let mut line = line.to_string();
                line = chompd(&line);
                // Possibly parse "key=value" or something. The real code in
                // DO might store entire lines in a single key. We'll assume
                // a simplistic approach:
                if let Some(idx) = line.find('=') {
                    let key = &line[..idx].trim();
                    let val = &line[idx+1..].trim();
                    self.data.insert(key.to_string(), val.to_string());
                }
            }
        }
        self.loaded = true;
    }

    /// Returns the raw (comma-separated) string from the dialog data,
    /// for a given key (`name`). If the key is missing, returns an empty string.
    ///
    /// This replicates the “scalar context” usage in Perl.
    pub fn get_dialog_line(&mut self, name: &str) -> String {
        // In Perl: if (!$_dialog{'loaded'}) { ... load ... }
        if !self.loaded {
            // You’d typically build the path from something like:
            // format!("{}/{}.dialog", datafolder, self.mode)
            let path = format!("{}.dialog", self.mode);
            self.load_dialog(&path);
        }
        let mut value = self.data.get(name).cloned().unwrap_or_default();
        value = chompd(&value);
        value
    }

    /// Returns a vector by splitting the line on commas, akin to
    /// “list context” in the original Perl. Each element is also trimmed.
    ///
    /// ```ignore
    /// let mut dd = DialogData::new("horas");
    /// let items = dd.get_dialog_array("horas");
    /// // e.g. ["Matins", "Lauds", "Prime", ...]
    /// ```
    pub fn get_dialog_array(&mut self, name: &str) -> Vec<String> {
        let line = self.get_dialog_line(name);
        line.split(',')
            .map(|s| chompd(s.trim()))
            .collect()
    }
}

/// A convenience function replicating the original `gethoras($C9f)`.
/// Under the hood, it calls `get_dialog_array("horas")` and possibly
/// restricts the array to indexes [0,1,6].
///
/// # Arguments
///
/// * `data` - A mutable reference to your `DialogData` holding the `horas` map.
/// * `c9f` - If true, only return [0,1,6].
///
/// ```ignore
/// let mut dd = DialogData::new("horas");
/// let horas_list = get_horas(&mut dd, false);
/// // e.g. ["Matins", "Lauds", "Prime", "Terce", "Sext", "None", "Vespers", "Compline"]
///
/// let short_list = get_horas(&mut dd, true);
/// // e.g. ["Matins", "Lauds", "Vespers"]
/// ```
pub fn get_horas(data: &mut DialogData, c9f: bool) -> Vec<String> {
    let mut horas = data.get_dialog_array("horas");
    if c9f {
        // If the array has at least 7 elements, we keep indexes 0,1,6
        // If not, we just do best-effort slicing
        let mut subset = Vec::new();
        if horas.len() > 0 {
            subset.push(horas[0].clone());
        }
        if horas.len() > 1 {
            subset.push(horas[1].clone());
        }
        if horas.len() > 6 {
            subset.push(horas[6].clone());
        }
        horas = subset;
    }
    // Trim trailing whitespace from the last item
    if let Some(last) = horas.last_mut() {
        *last = last.trim_end().to_string();
    }
    horas
}

/// The original Perl code does a more advanced rewriting of certain runtime
/// parameters by reading them from the “dialog” file, splitting by `;;\r?\n`,
/// then re-syncing with a “setup” system. For illustration, we show a
/// simplified version that splits lines, tries to parse them, and then might
/// call external “setup” functions. You should adapt this to your real
/// Rust-based config logic.
pub fn set_runtime_options(
    dialog_data: &mut DialogData,
    name: &str,
    setup_data: &HashMap<String, String>,
    // placeholders for the "global" variables from the original
    blackfont: &mut String,
    smallblack: &mut String,
) {
    // This replicates: `my @parameters = split(/;;\r?\n/, getdialog($name));`
    // We'll just look up that line and do a naive split.
    let dialog_line = dialog_data.get_dialog_line(name);
    let parameters: Vec<&str> = dialog_line.split(";;\r\n").collect();

    // Then: `my @setupt = split(/;;/, getsetup($name));`
    // We assume `setup_data` might hold `name` => "abc;;def;;ghi"
    // or we have a more advanced structure. For now, we do a naive approach.
    let setup_line = setup_data.get(name).cloned().unwrap_or_default();
    let setupt: Vec<&str> = setup_line.split(";;").collect();

    // In Perl, we do index-based logic. We'll replicate an approximate approach:
    let mut i = 1;
    for param_line in parameters {
        let mut parts = param_line.split("~>").collect::<Vec<_>>();
        // E.g. "parname~>parvalue~>parmode~>parpar~>parpos~>parfunc~>parhelp"
        // We'll just handle the subset we need.
        let parpos = if parts.len() > 4 { parts[4] } else { "" };
        let mut pos_num = parpos.parse::<usize>().unwrap_or(0);
        if pos_num == 0 {
            pos_num = i;
            i += 1;
        }
        // The code also does: `parvalue = substr($parvalue, 1);` => remove leading char?
        // We can approximate that logic if needed.
        // Then calls `strictparam($parvalue)` => sets or else read from `setupt[pos-1]`.
        // We'll just do a placeholder.
        let p = if pos_num - 1 < setupt.len() {
            setupt[pos_num - 1].to_string()
        } else {
            "".to_string()
        };
        // In Perl, it might do `$blackfont =~ s/black//;`.
        // We'll do that last.
    }

    // Approximate the final lines:
    // `$blackfont =~ s/black//;`
    *blackfont = blackfont.replace("black", "");
    // `$smallblack =~ s/black//;`
    *smallblack = smallblack.replace("black", "");
}

/// The original code tries to find a user-friendly portion of a
/// “versions” string that references the given `version`. If not found,
/// just returns the input. This is a partial replication of the logic.
pub fn version_displayname(dialog_data: &mut DialogData, version: &str) -> String {
    let s = dialog_data.get_dialog_line("versions");
    // The Perl approach: `my $i = index($s, $version) - 1;`
    // if i == -1 or s[i] = ',' => return version
    // else find the preceding comma => substring
    if let Some(idx) = s.find(version) {
        let i = idx as i32 - 1;
        if i < 0 || &s.as_bytes()[i as usize..i as usize+1] == b"," {
            return version.to_string();
        }
        // find rindex of ',' in s[..i-1]
        let scope_end = (i - 1).max(0) as usize;
        let sub_slice = &s[..scope_end];
        if let Some(k) = sub_slice.rfind(',') {
            let start = k + 1;
            let extracted = &s[start..(i as usize)];
            return extracted.trim().to_string();
        } else {
            return version.to_string();
        }
    } else {
        version.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Tests the helper function with several inputs.
    #[test]
    fn test_parse_line() {
        // Valid cases.
        assert_eq!(
            parse_line("$datafolder='/some/path'"),
            Some(("datafolder".to_string(), "/some/path".to_string()))
        );
        assert_eq!(
            parse_line("$blackfont = 'some-value'"),
            Some(("blackfont".to_string(), "some-value".to_string()))
        );
        assert_eq!(
            parse_line("$empty=''"),
            Some(("empty".to_string(), "".to_string()))
        );
        // Even if there is extra text after the closing quote,
        // our implementation (like the original regex) takes everything up to the last quote.
        assert_eq!(
            parse_line("$var='value' extra"),
            Some(("var".to_string(), "value".to_string()))
        );

        // Invalid cases.
        assert_eq!(parse_line("not a valid line"), None);
        assert_eq!(parse_line("$var=value'"), None);  // missing opening quote
        assert_eq!(parse_line("$var='value"), None);    // missing closing quote
    }
}
