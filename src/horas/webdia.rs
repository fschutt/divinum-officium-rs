//! A module that translates the given Perl web-dialog functions into Rust.

/// Build the HTML head.
///
/// # Arguments
///
/// * `title` – the page title
/// * `onload` – an optional onload attribute
/// * `link` – color for links
/// * `visited_link` – color for visited links
/// * `dialog_background` – background color for the dialog
/// * `white_background` – if true, alternate dark-mode styles are output
/// * `dialog_font` – font description for dialog
/// * `officium` – the form action URL
/// * `horasjs` – a function that returns a JavaScript snippet
///
/// Returns the complete HTML head as a String.
pub fn html_head(
    title: &str,
    onload: Option<&str>,
    link: &str,
    visited_link: &str,
    dialog_background: &str,
    white_background: bool,
    dialog_font: &str,
    officium: &str,
    horasjs: impl Fn() -> String,
) -> String {
    let mut output = String::new();

    // Build the horasjs block.
    let horasjs_str = format!(
        "<SCRIPT TYPE='text/JavaScript' LANGUAGE='JavaScript1.2'>\n{}\
        </SCRIPT>",
        horasjs()
    );
    let onload_str = if let Some(s) = onload {
        format!(" onload=\"{}\";", s)
    } else {
        String::new()
    };

    output.push_str("Content-type: text/html; charset=utf-8\n\n");
    output.push_str("<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.01 Transitional//EN\">\n");
    output.push_str("<HTML><HEAD>\n");
    output.push_str("  <META NAME=\"Resource-type\" CONTENT=\"Document\">\n");
    output.push_str("  <META NAME=\"description\" CONTENT=\"Divine Office\">\n");
    output.push_str("  <META NAME=\"keywords\" CONTENT=\"Divine Office, Breviarium, Liturgy, Traditional, Zsolozsma\">\n");
    output.push_str("  <META NAME=\"Copyright\" CONTENT=\"Like GNU\">\n");
    output.push_str("  <meta name=\"color-scheme\" content=\"dark light\">\n");
    output.push_str("  <STYLE>\n");
    output.push_str("    /* https://www.30secondsofcode.org/css/s/offscreen/ */\n");
    output.push_str("    .offscreen {\n");
    output.push_str("      border: 0;\n");
    output.push_str("      clip: rect(0 0 0 0);\n");
    output.push_str("      height: 1px;\n");
    output.push_str("      margin: -1px;\n");
    output.push_str("      overflow: hidden;\n");
    output.push_str("      padding: 0;\n");
    output.push_str("      position: absolute;\n");
    output.push_str("      width: 1px;\n");
    output.push_str("    }\n");
    output.push_str("    h1, h2 {\n");
    output.push_str("      text-align: center;\n");
    output.push_str("      font-weight: normal;\n");
    output.push_str("    }\n");
    output.push_str("    h2 {\n");
    output.push_str("      margin-top: 4ex;\n");
    output.push_str("      color: maroon;\n");
    output.push_str("      font-size: 112%;\n");
    output.push_str("      font-weight: bold;\n");
    output.push_str("      font-style: italic;\n");
    output.push_str("    }\n");
    output.push_str("    p {\n");
    output.push_str("      color: black;\n");
    output.push_str("    }\n");
    output.push_str(&format!("    a:link {{ color: {}; }}\n", link));
    output.push_str(&format!("    a:visited {{ color: {}; }}\n", visited_link));
    output.push_str(&format!("    body {{ background: {}; }}\n", dialog_background));
    output.push_str("    .contrastbg { background: white; }\n");
    output.push_str("    .nigra { color: black; }\n");

    if white_background {
        output.push_str("    @media (prefers-color-scheme: dark) {\n");
        output.push_str("      body {\n");
        output.push_str("        background: black;\n");
        output.push_str("        color: white;\n");
        output.push_str("      }\n");
        output.push_str("      table { color: white; }\n");
        output.push_str("      a:link { color: #AFAFFF; }\n");
        output.push_str("      a:visited { color: #AFAFFF; }\n");
        output.push_str("      p { color: white; }\n");
        output.push_str("      .contrastbg {\n");
        output.push_str("        background: #3F3F3F;\n");
        output.push_str("        color: white;\n");
        output.push_str("      }\n");
        output.push_str("      .nigra {  color: white;  }\n");
        output.push_str("      }\n");
    } else {
        output.push_str("    @media (prefers-color-scheme: dark) {\n");
        output.push_str(&format!(
            "      body {{\n        background: {};\n        color: black;\n      }}\n",
            dialog_background
        ));
        output.push_str("      select {\n        background: lightgrey;\n        color: black;\n      }\n");
        output.push_str("      input[type=\"select\"] {\n        background: lightgrey;\n        color: black;\n      }\n");
        output.push_str("      input[type=\"submit\"] {\n        background: grey;\n        color: black;\n      }\n");
        output.push_str("      input[type=\"text\"] {\n        background: white;\n        color: black;\n      }\n");
        output.push_str("    }\n");
    }

    output.push_str("  </STYLE>\n");
    output.push_str(&format!("  <TITLE>{}</TITLE>\n", title));
    output.push_str(&horasjs_str);
    output.push_str("</HEAD>\n");
    output.push_str(&format!("<BODY{}>\n", onload_str));
    output.push_str(&format!(
        "<FORM ACTION=\"{}\" METHOD=\"post\" TARGET=\"_self\">\n",
        officium
    ));

    output
}

/// Finish the HTML document.
///
/// If an error or debug message is provided these are inserted before closing the form.
pub fn html_end(error: Option<&str>, debug: Option<&str>) -> String {
    let mut output = String::new();
    if let Some(e) = error {
        output.push_str(&format!(
            "<P ALIGN='CENTER'><FONT COLOR='red'>{}</FONT></P>\n",
            e
        ));
    }
    if let Some(d) = debug {
        output.push_str(&format!(
            "<P ALIGN='center'><FONT COLOR='blue'>{}</FONT></P>\n",
            d
        ));
    }
    output.push_str("</FORM></BODY></HTML>");
    output
}

/// Build an HTML input widget.
///
/// The type of widget is determined by `parmode` (e.g. "label", "entry", "text", "checkbutton", etc.).
/// Other “external” values (for example, the URL for images or the dialog font) are passed in as arguments.
pub fn html_input(
    parname: &str,
    parvalue: &str,
    parmode: &str,
    parpar: Option<&str>,
    parfunc: Option<&str>,
    parhelp: Option<&str>,
    htmlurl: &str,
    dialog_font: &str,
) -> String {
    let mut output = String::new();
    let mode = parmode.to_lowercase();

    if mode.starts_with("label") {
        let ilabel = if let Some(pp) = parpar {
            wrap(parvalue, pp, "<br/>\n")
        } else {
            parvalue.to_string()
        };
        output.push_str(&ilabel);
        output.push_str(&format!(
            "<INPUT TYPE='HIDDEN' NAME='{}' VALUE='{}'>\n",
            parname, parvalue
        ));
        return output;
    }

    if mode.contains("entry") {
        let width: usize = parpar
            .and_then(|s| s.parse().ok())
            .filter(|&w| w != 0)
            .unwrap_or(3);
        let jsfunc = parfunc.map_or(String::new(), |f| format!("onchange=\"{};\"", f));
        output.push_str(&format!(
            "<INPUT TYPE='TEXT' NAME='{}' ID='{}' {} SIZE={} VALUE='{}'>\n",
            parname, parname, jsfunc, width, parvalue
        ));
        return output;
    }

    if mode.starts_with("text") {
        let size = parpar.unwrap_or("3x12");
        let sizes: Vec<&str> = size.split('x').collect();
        let (rows, cols) = if sizes.len() >= 2 {
            (sizes[0], sizes[1])
        } else {
            ("3", "12")
        };
        output.push_str(&format!(
            "<TEXTAREA NAME='{}' ID='{}' COLS='{}' ROWS='{}'>{}</TEXTAREA><br/>\n",
            parname, parname, cols, rows, parvalue
        ));
        output.push_str(&format!(
            "<A HREF='#' onclick='loadrut();'>{}Load</FONT></A>",
            setfont(dialog_font, "")
        ));
        return output;
    }

    if mode.contains("checkbutton") {
        let checked = if parvalue != "0" && !parvalue.is_empty() {
            "CHECKED"
        } else {
            ""
        };
        let jsfunc = parfunc.map_or(String::new(), |f| format!("onclick=\"{};\"", f));
        output.push_str(&format!(
            "<INPUT TYPE='CHECKBOX' NAME='{}' ID='{}' {} {}>\n",
            parname, parname, checked, jsfunc
        ));
        return output;
    }

    if mode.starts_with("radio") {
        // Assume parpar is a comma-separated list of options.
        let options: Vec<&str> = parpar.unwrap_or("").split(',').collect();
        let vertical = mode.contains("vert");
        if vertical {
            output.push_str("<TABLE>");
        }
        for (j, option) in options.iter().enumerate() {
            let checked = if parvalue == &(j + 1).to_string() {
                "CHECKED"
            } else {
                ""
            };
            if vertical {
                output.push_str("<TR><TD>");
            }
            let jsfunc = parfunc.map_or(String::new(), |f| format!("onclick=\"{};\"", f));
            output.push_str(&format!(
                "<INPUT TYPE=RADIO NAME='{}' ID='{}' VALUE={} {} {}>",
                parname,
                parname,
                j + 1,
                checked,
                jsfunc
            ));
            output.push_str(&format!("<FONT SIZE=-1> {} </FONT>\n", option));
            if vertical {
                output.push_str("</TD></TR>");
            }
        }
        if vertical {
            output.push_str("</TABLE>");
        }
        return output;
    }

    if mode.starts_with("updown") {
        // For updown we need a placeholder for parpos.
        let parpos = "0";
        let parvalue_num = if parvalue.is_empty() { "5" } else { parvalue };
        output.push_str(&format!(
            "<IMG SRC=\"{}/down.gif\" ALT=down ALIGN=TOP onclick=\"{}({},{})\">\n",
            htmlurl,
            parfunc.unwrap_or(""),
            parpos,
            -1
        ));
        output.push_str(&format!(
            "<INPUT TYPE=TEXT NAME='{}' ID='{}' SIZE={} VALUE={} onchange=\"{}({},{})\">\n",
            parname,
            parname,
            parpar.unwrap_or(""),
            parvalue_num,
            parfunc.unwrap_or(""),
            parpos,
            0
        ));
        output.push_str(&format!(
            "<IMG SRC=\"{}/up.gif\" ALT=up ALIGN=TOP onclick=\"{}({},{})\">\n",
            htmlurl,
            parfunc.unwrap_or(""),
            parpos,
            1
        ));
        return output;
    }

    if mode.starts_with("scale") {
        output.push_str(&format!(
            "<INPUT TYPE=TEXT SIZE=6 NAME='{}' ID='{}' VALUE={}> \n",
            parname, parname, parvalue
        ));
        return output;
    }

    if mode.contains("filesel") {
        if let Some(pp) = parpar {
            if pp.contains("stack") {
                output.push_str(&format!(
                    "<INPUT TYPE=RADIO NAME='mousesel' VALUE='stack' onclick='mouserut(\"stack{}\");'>\n",
                    "0" // placeholder for parpos
                ));
            }
        }
        output.push_str(&format!(
            "<INPUT TYPE=TEXT SIZE=16 NAME='{}' ID='{}' VALUE='{}'>\n",
            parname, parname, parvalue
        ));
        if let Some(pp) = parpar {
            if !pp.contains("stackonly") {
                output.push_str(&format!(
                    "<INPUT TYPE=BUTTON VALUE=' ' onclick='filesel(\"{}\", \"{}\");'>\n",
                    parname, pp
                ));
            }
        }
        return output;
    }

    if mode.contains("color") {
        let size = parpar.unwrap_or("3");
        output.push_str(&format!(
            "<INPUT TYPE=RADIO NAME='mousesel' VALUE='color' onclick='mouserut(\"color{}\");'>\n",
            "0" // placeholder for parpos
        ));
        output.push_str(&format!(
            "<INPUT TYPE=TEXT SIZE=8 NAME='{}' ID='{}' VALUE='{}'>\n",
            parname, parname, parvalue
        ));
        output.push_str(&format!(
            "<INPUT TYPE=BUTTON VALUE=' ' onclick='colorsel(\"{}\",{});'>\n",
            parname, size
        ));
        return output;
    }

    if mode.contains("font") {
        let size = parpar.unwrap_or("16");
        output.push_str(&format!(
            "<INPUT TYPE=TEXT SIZE={} NAME='{}' ID='{}' VALUE='{}'>\n",
            size, parname, parname, parvalue
        ));
        output.push_str(&format!(
            "<INPUT TYPE=BUTTON VALUE=' ' onclick='fontsel(\"{}\");'>\n",
            parname
        ));
        return output;
    }

    if mode.starts_with("option") {
        let a = parpar.unwrap_or("");
        if a.is_empty() {
            return String::new();
        }
        // For simplicity, assume a is a comma‐separated list.
        let optarray: Vec<&str> = if a.contains("@") {
            a.split(',').collect()
        } else if a.starts_with('{') && a.ends_with('}') {
            a[1..a.len() - 1].split(',').collect()
        } else {
            a.split(',').collect()
        };
        let onclick = if mode.contains("select") {
            "onchange='buttonclick(\"\");'".to_string() // placeholder
        } else if let Some(func) = parfunc {
            format!("onchange=\"{};\"", func)
        } else {
            String::new()
        };
        let mut opt_output = String::new();
        opt_output.push_str(&format!(
            "<SELECT ID={} NAME={} SIZE=1 {}>\n",
            parname, parname, onclick
        ));
        for opt in optarray {
            let parts: Vec<&str> = opt.split('/').collect();
            let display = parts.get(0).unwrap_or(&"");
            let value_opt = parts.get(1).unwrap_or(display);
            let selected = if *value_opt == parvalue { "SELECTED" } else { "" };
            opt_output.push_str(&format!(
                "<OPTION {} VALUE=\"{}\">{}\n",
                selected, value_opt, display
            ));
        }
        opt_output.push_str("</SELECT>\n");
        output.push_str(&opt_output);
        return output;
    }

    output
}

/// Helper function to “wrap” a string with a given parameter and break.
fn wrap(input: &str, par: &str, br: &str) -> String {
    // This is a placeholder for the actual wrap logic.
    format!("{}{}{}", input, par, br)
}

/// Cleanses a string by removing dangerous characters.
///
/// If the input string consists only of word characters then it is returned as is.
/// Otherwise it is split on semicolons and each part is checked; parts not matching the allowed patterns are replaced with an empty string.
pub fn cleanse(s: &str) -> String {
    if is_word(s) {
        return s.to_string();
    }
    let parts: Vec<&str> = s.split(';').collect();
    let mut cleansed_parts = Vec::new();
    for part in parts {
        if is_safe_part(part) {
            cleansed_parts.push(part);
        } else {
            cleansed_parts.push("");
        }
    }
    cleansed_parts.join(";")
}

/// Returns true if all characters in the string are alphanumeric or underscore.
fn is_word(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Checks if a part is “safe” (this is a simplified version of the Perl logic).
fn is_safe_part(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    if s.starts_with('\'') && s.ends_with('\'') {
        return true;
    }
    if s.starts_with('$') {
        let parts: Vec<&str> = s.split('=').collect();
        if parts.len() == 2 && parts[0].len() > 1 && is_word(&parts[0][1..])
            && parts[1].starts_with('\'')
            && parts[1].ends_with('\'')
        {
            return true;
        }
    }
    // Otherwise, ensure that none of the dangerous characters are present.
    !s.contains('\'')
        && !s.contains('`')
        && !s.contains('"')
        && !s.contains('\\')
        && !s.contains('=')
        && !s.contains('{')
        && !s.contains('}')
        && !s.contains('(')
        && !s.contains(')')
}

/// Returns a <FONT> tag string built from a font description and text.
///
/// The font description is expected to contain a size (e.g. "16") and a color (e.g. "red"),
/// as well as the words "bold" and/or "italic" if appropriate.
pub fn setfont(font_desc: &str, text: &str) -> String {
    if font_desc.is_empty() {
        return text.to_string();
    }
    let size = extract_first_number(font_desc).unwrap_or(0);
    let mut color = extract_last_word(font_desc).unwrap_or_default();
    if color.eq_ignore_ascii_case("italic") {
        color.clear();
    }
    let mut font_tag = String::from("<FONT ");
    if size != 0 {
        font_tag.push_str(&format!("SIZE='{}' ", size));
    }
    if !color.is_empty() && !color.eq_ignore_ascii_case("black") {
        font_tag.push_str(&format!("COLOR=\"{}\"", color));
    }
    font_tag.push('>');
    let bold = if font_desc.to_lowercase().contains("bold") {
        "<B>"
    } else {
        ""
    };
    let bold_end = if !bold.is_empty() { "</B>" } else { "" };
    let italic = if font_desc.to_lowercase().contains("italic") {
        "<I>"
    } else {
        ""
    };
    let italic_end = if !italic.is_empty() { "</I>" } else { "" };
    format!(
        "{}{}{}{}{}{}{}",
        font_tag, bold, italic, text, italic_end, bold_end, "</FONT>"
    )
}

/// Extracts the first integer (with optional + or - sign) found in the string.
fn extract_first_number(s: &str) -> Option<i32> {
    let mut num = String::new();
    for c in s.chars() {
        if c.is_digit(10) || ((c == '-' || c == '+') && num.is_empty()) {
            num.push(c);
        } else if !num.is_empty() {
            break;
        }
    }
    num.parse::<i32>().ok()
}

/// Extracts the last “word” (alphabetic characters) from the string.
fn extract_last_word(s: &str) -> Option<String> {
    let words: Vec<&str> = s.split_whitespace().collect();
    words.last().map(|w| {
        if w.chars().all(|c| c.is_alphabetic() || c == '#') {
            w.to_string()
        } else {
            String::new()
        }
    })
}

/// Replace sequences of plus characters with “cross” symbols.
///
/// This function splits the input on whitespace and replaces tokens:
/// - "+++" becomes "✙︎"
/// - "++" becomes "+"
/// - "+" becomes "✠"
///
/// (A simplified version of the Perl replacement logic.)
pub fn setcross(input: &str) -> String {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut output = String::new();
    for token in tokens {
        let replaced = if token == "+++" {
            "✙︎"
        } else if token == "++" {
            "+"
        } else if token == "+" {
            "✠"
        } else {
            token
        };
        output.push_str(replaced);
        output.push(' ');
    }
    output.trim_end().to_string()
}

/// Build a link radio-code input.
///
/// The name is sanitized by replacing characters (parentheses and apostrophes)
/// with HTML entities.
pub fn linkcode(name: &str, ind: i32, lang: &str, disabled: bool) -> String {
    let mut sanitized = name.to_string();
    sanitized = sanitized.replace("(", "&lpar");
    sanitized = sanitized.replace(")", "&rpar");
    sanitized = sanitized.replace("'", "&apos");
    let disabled_str = if disabled { "disabled" } else { "" };
    format!(
        "<INPUT TYPE='RADIO' NAME='link' {} onclick='linkit(\"{}\", {}, \"{}\");'>",
        disabled_str, sanitized, ind, lang
    )
}

/// A helper to generate a collapse radio button.
pub fn linkcode1() -> String {
    format!("&ensp;<INPUT TYPE='RADIO' NAME='collapse' onclick=\"linkit('','10000','');\">\n")
}

/// Build an option selector widget.
///
/// # Arguments
///
/// * `label` – the text label for the selector
/// * `onchange` – JavaScript code to execute on change
/// * `default` – the default value
/// * `options` – a slice of (display, value) pairs
pub fn option_selector(
    label: &str,
    onchange: &str,
    default: &str,
    options: &[(&str, &str)],
) -> String {
    let id = label.to_lowercase().replace(" ", "");
    let mut output = String::new();
    output.push_str(&format!(
        "&ensp;<LABEL FOR='{}' CLASS='offscreen'>{}</LABEL>\n",
        id, label
    ));
    output.push_str(&format!(
        "<SELECT ID='{}' NAME='{}' SIZE='1' onchange=\"{}\">\n",
        id, id, onchange
    ));
    for (display, value) in options {
        let selected = if *value == default { "SELECTED" } else { "" };
        output.push_str(&format!(
            "<OPTION {} VALUE=\"{}\">{}\n",
            selected, value, display
        ));
    }
    output.push_str("</SELECT>\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_head() {
        let html = html_head(
            "Test Title",
            Some("init()"),
            "blue",
            "purple",
            "#ffffff",
            true,
            "16 bold red",
            "officium_action",
            || "console.log('hi');".to_string(),
        );
        assert!(html.contains("Test Title"));
        assert!(html.contains("onload=\"init()\";"));
        assert!(html.contains("blue"));
        assert!(html.contains("purple"));
        assert!(html.contains("#ffffff"));
        assert!(html.contains("officium_action"));
    }

    #[test]
    fn test_html_end() {
        let html = html_end(Some("Error occurred"), Some("Debug info"));
        assert!(html.contains("Error occurred"));
        assert!(html.contains("Debug info"));
        assert!(html.contains("</FORM></BODY></HTML>"));
    }

    #[test]
    fn test_cleanse() {
        let safe = "abc123";
        assert_eq!(cleanse(safe), safe);
        let unsafe_str = "abc;bad'value";
        let cleansed = cleanse(unsafe_str);
        // In this simplified logic, the unsafe part is replaced with an empty string.
        assert_eq!(cleansed, "abc;;");
    }

    #[test]
    fn test_setfont() {
        let result = setfont("16 bold italic red", "Hello");
        assert!(result.contains("SIZE='16'"));
        assert!(result.contains("COLOR=\"red\""));
        assert!(result.contains("<B>"));
        assert!(result.contains("<I>"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_setcross() {
        let input = "This is +++ a test ++ of + crosses.";
        let output = setcross(input);
        // We expect the tokens replaced accordingly.
        assert!(output.contains("✙︎"));
        assert!(output.contains("+"));
        assert!(output.contains("✠"));
    }

    #[test]
    fn test_linkcode() {
        let result = linkcode("Test(name)", 5, "en", false);
        assert!(result.contains("&lpar"));
        assert!(result.contains("&rpar"));
        assert!(result.contains("linkit"));
    }

    #[test]
    fn test_option_selector() {
        let options = vec![("Option1", "1"), ("Option2", "2")];
        let result = option_selector("Select", "doChange()", "1", &options);
        assert!(result.contains("Select"));
        assert!(result.contains("onchange=\"doChange()\""));
        assert!(result.contains("Option1"));
    }
}

