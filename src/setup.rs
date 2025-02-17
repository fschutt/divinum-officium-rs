//! setup.rs
//!
//! This module corresponds to the original `setup.pl`, which manages
//! certain global or session-based setup variables (`%_setup` in Perl).
//!
//! # Overview
//!
//! The original code in `setup.pl` provides these main routines:
//!
//! - **`getsetup($name)`**: Retrieves a string from `_setup{$name}`, or splits
//!   it into comma-separated parts depending on context. In Rust, we replace
//!   the “wantarray” distinction with two separate functions (one returning a
//!   `String`, another returning a `Vec<String>`).
//!
//! - **`loadsetup($setup)`**: Loads or parses the `_setup` hash from either a
//!   single string `$setup` or from a file (e.g. `horas.setup` or `missa.setup`)
//!   in conjunction with the code from `setupstring`. We replicate that logic
//!   in a simplified manner here.
//!
//! - **`setsetupvalue($name, $ind, $value)`**: Modifies the `_setup{$name}`
//!   item by splitting on `;;`, then changing the line at index `$ind` to
//!   something like `="value"`.
//!
//! - **`setsetup($name, @values)`**: Sets multiple values, calling
//!   `setsetupvalue` on each.
//!
//! - **`savesetup(\%hash, $sep)`**: Serializes the `_setup` hash into a
//!   string, either in a “key;;;value;;;key;;;value” format or a
//!   `key="value",` format, depending on the `$flag` argument.
//!
//! - **`setuptable($command, $title)`**: Generates an HTML table for
//!   presenting the user with options to edit. In Perl, it calls
//!   `getdialog($command)`, splits lines by `;;`, each line by `~>`, and
//!   then injects HTML code for inputs, linking to help if needed. We
//!   replicate it partially, leaving placeholders for the UI specifics.
//!
//! - **`getsetupvalue()`**: Fetches user-submitted parameters from a form
//!   (Perl’s `$q->param`) and updates `_setup{'parameters'}`. In Rust, we
//!   need to adapt to how form data is retrieved. We illustrate the logic,
//!   leaving placeholders for the actual retrieval method (“`cleanse`”,
//!   or “`q->param`”).
//!
//! The code below uses a struct `Setup` that contains a `HashMap<String, String>`
//! replicating the `_setup` hash. You may integrate this with your web framework
//! or UI to replicate the original CGI behavior.

use std::collections::HashMap;

/// Holds the internal `_setup` data (key → string). In Perl, this was `%_setup`.
#[derive(Default)]
pub struct Setup {
    store: HashMap<String, String>,
}

impl Setup {
    /// Creates a new, empty Setup instance.
    pub fn new() -> Self {
        Setup {
            store: HashMap::new(),
        }
    }

    /// Equivalent to the Perl `getsetup($name)` in scalar context.
    /// Returns the entire stored string for `_setup{$name}`.  
    /// If the key does not exist, returns an empty string.
    pub fn getsetup_string(&self, name: &str) -> String {
        self.store.get(name).cloned().unwrap_or_default()
    }

    /// Returns `_setup{$name}` split by commas, equivalent to the Perl
    /// `getsetup($name)` in list context.  
    /// If `_setup{$name}` is empty or does not exist, returns an empty vector.
    pub fn getsetup_array(&self, name: &str) -> Vec<String> {
        let raw = self.getsetup_string(name);
        if raw.is_empty() {
            vec![]
        } else {
            raw.split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>()
        }
    }

    /// Equivalent to `loadsetup($setup)`. In the original code:
    ///
    /// - If `$setup` is provided (non-empty), it splits it by `;;;` into
    ///   key-value pairs for `_setup`.
    /// - Otherwise, it tries to figure out if `$datafolder` ends in
    ///   “missa” or “horas” and loads `"$1.setup"` from `setupstring("", "$1.setup")`.
    ///
    /// Here, we provide two methods. One is `load_from_str`, the other
    /// is `load_from_file` (or from another function that calls your
    /// `setupstring` logic). The user can pick which approach is relevant.
    pub fn load_from_str(&mut self, setup_str: &str) {
        if setup_str.is_empty() {
            // skip
            return;
        }
        // The original code: `%_setup = split(';;;', $setup);`
        // That means the string is something like "key1 value1;;;key2 value2;;;...".
        // The real usage might differ. We'll guess it's "k1 v1;;;k2 v2;;;..."
        let parts = setup_str.split(";;;").collect::<Vec<_>>();
        // We interpret these in pairs: key, value
        let mut i = 0;
        while i + 1 < parts.len() {
            let key = parts[i].trim().to_string();
            let value = parts[i + 1].trim().to_string();
            self.store.insert(key, value);
            i += 2;
        }
    }

    /// A placeholder for loading from the typical "horas.setup" or "missa.setup".
    /// In the original code, it uses `setupstring("", "$1.setup")`.  
    /// You can adapt this to your own code that calls the Rust version of
    /// “setupstring(...)”.
    pub fn load_from_file(&mut self, path: &str) {
        // We pretend to read from a data file, parse it as though it were
        // a set of lines we store as a single string. The real code might
        // differ. This is just an example.
        if let Ok(content) = std::fs::read_to_string(path) {
            // This content might be in the same "k v;;;k2 v2" format or
            // might be a multi-section text from the `setupstring`.
            // We'll do a naive approach to match load_from_str.
            self.load_from_str(&content);
        }
    }

    /// Equivalent to `setsetupvalue($name, $ind, $value)`.  
    /// This method splits the existing `_setup{$name}` by `;;`, modifies
    /// the `$ind`th item by replacing `=.*/` with `='$value'`.
    ///
    /// The original code also does `$value =~ s/^'(.*)'$/$1/;`
    /// so we remove surrounding quotes from `$value`.
    pub fn setsetupvalue(&mut self, name: &str, ind: usize, value: &str) {
        let script = self.getsetup_string(name);
        let script_no_newlines = script.replace("\n", "").replace("\r", "");
        let mut parts: Vec<String> = script_no_newlines.split(";;").map(|s| s.to_string()).collect();

        // Remove surrounding single quotes from value if any.
        let mut val = value.to_string();
        if val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2 {
            val = val[1..val.len() - 1].to_string();
        }

        if ind < parts.len() {
            // Each chunk might be something like "variable='xxx'". We do
            // `parts[ind] =~ s/=.*?/='$value'/`.
            // We'll do a naive approach: find '=' and replace everything after.
            if let Some(eq_pos) = parts[ind].find('=') {
                let new_chunk = format!("{}='{}'", &parts[ind][..eq_pos], val);
                parts[ind] = new_chunk;
            }
        } else {
            // If we exceed, we might want to push a new item
            let new_chunk = format!("unknown='{}'", val);
            parts.push(new_chunk);
        }

        let new_joined = parts.join(";;");
        self.store.insert(name.to_string(), new_joined);
    }

    /// Equivalent to `setsetup($name, $value1, $value2, ...)`.  
    /// Calls `setsetupvalue` on each value in sequence.
    pub fn setsetup(&mut self, name: &str, values: &[&str]) {
        for (i, &val) in values.iter().enumerate() {
            self.setsetupvalue(name, i, val);
        }
    }

    /// Equivalent to `savesetup(\%hash, $sep)`.  
    /// In the original code:
    ///
    /// - If `$flag` is true, produce a string where each key-value pair is appended
    ///   in the format `"$key;;;$value;;;"`.
    /// - Otherwise, produce something like `"$key=\"$value\","`.
    ///
    /// We replicate that logic in Rust.
    pub fn savesetup(&self, flag: bool) -> String {
        let mut keys: Vec<&String> = self.store.keys().collect();
        keys.sort(); // sort them for consistent output

        let mut result = String::new();
        for &k in &keys {
            let v = self.store.get(k).cloned().unwrap_or_default();
            if flag {
                // Remove trailing semicolons/spaces
                let v_clean = v.trim_end_matches(|c: char| c == ';' || c.is_whitespace());
                // Append `$k;;;$v_clean;;;`
                result.push_str(&format!("{};;;{};;;", k, v_clean));
            } else {
                // `$k="$v",`
                result.push_str(&format!("{}=\"{}\",", k, v));
            }
        }
        result
    }

    /// In the original Perl, `setuptable($command, $title)` generates HTML
    /// for the user to edit options. It references `getdialog($command)`,
    /// splits lines by `;;`, then each line by `~>`, e.g.
    /// `(parname, parvar, parmode, parpar, parpos, parfunc, parhelp)`.
    ///
    /// We replicate that logic in Rust, returning a `String` of HTML. You can
    /// then serve it or integrate with your templating engine.  
    /// **Note**: This references `getdialog(...)`, `htmlInput(...)`, `$helpfile`,
    /// `$background`, etc. We provide stubs or placeholders for them.
    pub fn setuptable(&self, command: &str, title: &str) -> String {
        // In Perl, we do something like:
        // ```
        // $title =~ s/setupparameters/Options/i;
        // my $output = "<H1 ALIGN=CENTER>...some HTML..."
        // my $scripto = getdialog($command);
        // ...
        // foreach (split(';;', $scripto)) {
        //   my ($parname, $parvar, $parmode, $parpar, $parpos, $parfunc, $parhelp) = split('~>');
        //   ...
        // }
        // ...
        // return $output;
        // ```
        //
        let mut mod_title = title.replace("setupparameters", "Options");
        let background = " BGCOLOR=\"#FFFFF0\""; // For example
        let helpfile = r"C:\path\to\horashelp.html"; // approximate
        // We assume we have a function like `getdialog(command) -> String`:
        let scripto = getdialog_stub(command);

        if scripto.is_empty() {
            // in the original code: beep(); $error='No setup parameter'; return;
            return "<p>No setup parameter</p>".to_string();
        }

        let mut output = format!(
            r#"<H1 ALIGN=CENTER><FONT COLOR=MAROON><B><I>{}</I></B></FONT></H1>
<TABLE BORDER=2 CELLPADDING=5 ALIGN=CENTER{}>"#,
            mod_title, background
        );

        let parts = scripto.split(";;").collect::<Vec<_>>();
        let mut i = 1;
        for line in parts {
            let fields = line.split("~>").collect::<Vec<_>>();
            if fields.is_empty() {
                continue;
            }
            // fields: (parname, parvar, parmode, parpar, parpos, parfunc, parhelp)
            let parname = fields.get(0).unwrap_or(&"").trim();
            let parvar = fields.get(1).unwrap_or(&"").trim();
            let parmode = fields.get(2).unwrap_or(&"").trim();
            let _parpar = fields.get(3).unwrap_or(&"").trim();
            let mut parpos = fields.get(4).unwrap_or(&"").trim().to_string();
            let _parfunc = fields.get(5).unwrap_or(&"").trim();
            let parhelp = fields.get(6).unwrap_or(&"").trim();

            if parpos.is_empty() {
                parpos = i.to_string();
                i += 1;
            }
            if parmode.is_empty() {
                // skip
                continue;
            }

            output.push_str("<TR><TD ALIGN=left>\n");
            // If mode != "label"
            if !parmode.contains("label") {
                if parhelp.contains('#') {
                    output.push_str(&format!(r#"<A HREF="{}{}" TARGET='_new'>"#, helpfile, parhelp));
                }
                output.push_str(parname);
                if parhelp.contains('#') {
                    output.push_str("</A>\n");
                }
                output.push_str(" : </TD><TD ALIGN=right>");
            }
            // Here we’d normally call `htmlInput("I$parpos", parvalue, parmode, parpar, parfunc, parhelp)`.
            // We’ll just stub out something:
            output.push_str(&format!(
                r#"<INPUT TYPE="text" NAME="I{}" VALUE="{}">"#,
                parpos, parvar
            ));
            output.push_str("</TD></TR>\n");
        }
        output.push_str(
            r#"</TABLE>
<P ALIGN=CENTER>
<INPUT TYPE=SUBMIT NAME='button' VALUE=OK>
</P>"#,
        );
        output
    }

    /// In Perl, `getsetupvalue()` collects form input for the “parameters” line
    /// from the user, updating `_setup{'parameters'}`. We replicate that logic
    /// with placeholders for how you get form input in Rust.
    ///
    /// The code splits `getdialog('parameters')` by `;;\r?\n`, then each line
    /// by `~>` to get (parname, parvalue, ...). Then it fetches the user’s form
    /// submission for e.g. `I$parpos`, and sets `_setup{'parameters'}` accordingly.
    pub fn getsetupvalue(&mut self, form_data: &HashMap<String, String>) {
        let scripto = getdialog_stub("parameters");
        // We do split on ";;\r?\n" => but let's just do ";;"
        // for demonstration:
        let lines = scripto.split(";;\r?\n").collect::<Vec<_>>();
        let mut script_out = Vec::new();
        let mut i = 1;

        for line in lines {
            let fields = line.split("~>").collect::<Vec<_>>();
            if fields.is_empty() {
                continue;
            }
            let parname = fields.get(0).unwrap_or(&"").trim();
            let mut parvalue = fields.get(1).unwrap_or(&"").trim().to_string();
            // in Perl: `parvalue = substr($parvalue, 1);` => remove first char
            if !parvalue.is_empty() {
                parvalue.remove(0);
            }
            let parmode = fields.get(2).unwrap_or(&"").trim();
            let _parpar = fields.get(3).unwrap_or(&"").trim();
            let mut parpos = fields.get(4).unwrap_or(&"").trim().to_string();
            let _parfunc = fields.get(5).unwrap_or(&"").trim();
            let _parhelp = fields.get(6).unwrap_or(&"").trim();

            if parpos.is_empty() {
                parpos = i.to_string();
                i += 1;
            }

            let input_key = format!("I{}", parpos);
            // In the original code: `my $value = cleanse($q->param("I$parpos"));`
            // We do a placeholder:
            let mut value = form_data.get(&input_key).cloned().unwrap_or_default();
            if value.is_empty() && value != "0" {
                value = "".to_string();
            }
            if value == "on" {
                value = "1".to_string();
            }
            // Then Perl: `$$parvalue = $value;`
            // but $parvalue is a reference to a global var's name. We skip that.
            // Instead, we store it in a script array.
            script_out.push(format!("${}='{}'", parvalue, value));
        }
        // Then `_setup{'parameters'} = join(';;', @script);`
        let joined = script_out.join(";;");
        self.store.insert("parameters".to_string(), joined);
    }
}

//-------------------------------------
// Stubs for references from setup.pl
//-------------------------------------

/// In `setup.pl`, we have `getdialog($command)` from `dialogcommon`.
/// Here we stub it out. You can replace with a real call that returns
/// an appropriate string from the `DialogData` or similar.
fn getdialog_stub(_command: &str) -> String {
    // In the real code, it might read from `horas.dialog` or `missa.dialog`.
    // We'll just return a fake string for demonstration.
    // For example, "parname~>parvar~>parmode~>parpar~>parpos~>parfunc~>parhelp;; ..."
    "ExampleName~>$exampleVar~>text~>someMode~>1~>someFunc~>#helpRef;;".to_string()
}

/// A possible placeholder for the original `cleanse(...)`.  
/// In actual code, sanitize or process the user input to ensure no injection, etc.
#[allow(dead_code)]
fn cleanse(input: &str) -> String {
    // Minimal placeholder, could do advanced HTML escaping or other logic.
    input.trim().to_string()
}
