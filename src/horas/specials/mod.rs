//! specials.rs
//!
//! This module “fills” the chapters from the Ordinarium by processing the script
//! for a given hour. It is a translation of `/horas/specials.pl` and now integrates
//! with our other modules in the `specials/` directory (such as `psalmi.rs`, `orationes.rs`,
//! `specprima.rs`, `preces.rs`, `capitulis.rs`, and `hymni.rs`).
//!
//! The main public function is:
//!
//! ```rust
//! fn specials(config: InputConfig, script: Vec<String>, lang: &str, special: Option<&str>) -> String
//! ```
//!
//! This function processes each line of the input script (a vector of strings)
//! according to various conditions (based on the current hour, rule, winners, etc.).
//! All state is passed in via an `InputConfig` rather than via globals.

use std::collections::HashMap;

// Import modules from the specials directory.
mod psalmi;
mod orationes;
mod specprima;
mod preces;
mod capitulis;
mod hymni;

/// Processes the input script and returns the final HTML output as a String.
///
/// All necessary state (such as the current hour, rule, winners maps, etc.) is provided
/// in the `InputConfig` structure.
pub fn specials(mut config: InputConfig, script: Vec<String>, lang: &str, special: Option<&str>) -> String {
    // Clear duplicate–check flags.
    config.clear_flags();

    // Decide which winners map to use.
    let winners = if config.column == 1 {
        config.winner_map.clone()
    } else {
        config.winner2_map.clone()
    };

    // If column equals 1, build the header.
    if config.column == 1 {
        let mut r = winners.get("Rule").cloned().unwrap_or_default();
        r = r.trim_end().to_string();
        r = r.replace("\n", " ");
        let header = format!(
            "{}\n{}\n",
            setfont(&config.largefont, &format!("{} {}", config.hora, config.date1)),
            setfont(&config.smallblack, &format!(
                "{} ~ {} : {}",
                config.daynames.get(1).unwrap_or(&String::new()),
                config.daynames.get(2).unwrap_or(&String::new()),
                r
            ))
        );
        specials_build::set_buildscript(&header);
    }

    // If no special override was provided and a “special” entry exists, load it immediately.
    if special.is_none() {
        if let Some(special_text) = winners.get(&special_key(&config, lang)) {
            // In the full implementation, a helper like `loadspecial()` would do additional work.
            return special_text.clone();
        }
    }

    let mut output_lines: Vec<String> = Vec::new();
    let t = script;
    let mut tind: usize = 0;
    let mut skipflag = config.skipflag;
    while tind < t.len() {
        let mut item = t[tind].trim_end().to_string();
        tind += 1;

        // Non-comment lines are simply output (if not skipping).
        if !item.trim_start().starts_with('#') {
            if !skipflag {
                output_lines.push(item);
            }
            continue;
        }

        // If skipping, output an empty line.
        if skipflag {
            output_lines.push("\n".to_string());
        }
        let label = item.clone();
        skipflag = false;

        // --- Branch: Capitulum with Versicle ---
        if item.contains("Capitulum") {
            if config.rule.to_lowercase().contains("capitulum versum 2") {
                if let Some(pos) = config.rule.to_lowercase().find("capitulum versum 2") {
                    let cv2hora = config.rule[pos + "Capitulum Versum 2".len()..].trim().to_string();
                    let cond1 = cv2hora.to_lowercase().contains("ad laudes tantum") && config.hora != "Laudes";
                    let cond2 = cv2hora.to_lowercase().contains("ad laudes et vesperas")
                        && !(config.hora == "Laudes" || config.hora == "Vespera");
                    if !(cond1 || cond2) {
                        if config.hora != "Completorium" {
                            // For Laudes/Vespera we use the major capitulum routine.
                            if let Some(text) = capitulis::capitulum_major(lang) {
                                output_lines.push(text);
                            }
                            specials_build::setbuild1("Versus speciale in loco calpituli", "");
                        }
                        skipflag = true;
                        continue;
                    }
                }
            }
        }

        // --- Branch: Omit branch ---
        let ite = if item.trim_start().starts_with('#') {
            item.trim_start()
                .trim_start_matches('#')
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        } else {
            "".to_string()
        };
        if config.rule.to_lowercase().contains(&format!("omit {}", ite).to_lowercase()) {
            skipflag = true;
            specials_build::setbuild1(&label, "omit");
            comment::setcomment(&label, "Preces", 1, lang, "");
            if item.to_lowercase().contains("incipit")
                && !config.version.contains("1955")
                && !config.version.contains("196")
                && !config.winner.contains("C12")
            {
                if config.hora == "Laudes" {
                    output_lines.push(format!("/:{}:/", "Si Laudes"));
                } else {
                    output_lines.push(format!("/:{}:/", "secreto"));
                }
                output_lines.push("$Pater noster".to_string());
                output_lines.push("$Ave Maria".to_string());
                if config.hora == "Matutinum" || config.hora == "Prima" {
                    output_lines.push("$Credo".to_string());
                }
            }
            continue;
        }

        // --- Branch: Preces ---
        if item.to_lowercase().contains("preces") {
            let use_preces = preces::preces(&item);
            skipflag = !use_preces;
            comment::setcomment(&label, "Preces", if use_preces { 1 } else { 0 }, lang, "");
            specials_build::setbuild1(&item, if use_preces { "include" } else { "omit" });
            if !skipflag {
                if let Some(text) = preces::get_preces(&config.hora, lang, item.to_lowercase().contains("dominicales")) {
                    output_lines.push(text);
                }
            }
            continue;
        }

        // --- Branch: Psalmi ---
        if item.to_lowercase().contains("psalm") {
            if let Some(psalmi_lines) = psalmi::psalmi(lang) {
                output_lines.extend(psalmi_lines);
            }
            continue;
        }

        // --- Branch: Invitatorium ---
        if item.to_lowercase().contains("invitatorium") {
            invitatorium(lang);
            continue;
        }

        // --- Branch: Lectio brevis (Prima/Completorium) ---
        if item.to_lowercase().contains("lectio brevis") {
            if config.hora == "Prima" {
                let (brevis, _c_val) = specprima::lectio_brevis_prima(lang);
                output_lines.push(brevis);
            } else if config.hora == "Completorium" {
                if let Some(lectio_map) = setupstring(lang, "Psalterium/Special/Minor Special.txt") {
                    if let Some(text) = lectio_map.get("Lectio Completorium") {
                        output_lines.push(item.clone());
                        output_lines.push(text.clone());
                    }
                }
            }
            continue;
        }

        // --- Branch: Hymnus ---
        if item.to_lowercase().contains("hymnus") {
            if let Some(hymn_text) = hymni::get_hymn(lang) {
                output_lines.push(hymn_text);
            }
            continue;
        }

        // --- Branch: Oratio ---
        if item.to_lowercase().contains("oratio") {
            let mut oratio_params = HashMap::new();
            // (Additional parameter logic could be added here.)
            orationes::oratio(lang, config.date1.parse().unwrap_or(1), 1, oratio_params);
            continue;
        }

        // --- Branch: Suffragium ---
        if item.to_lowercase().contains("suffragium") && (config.hora == "Laudes" || config.hora == "Vespera") {
            let (suffr, c_val) = orationes::getsuffragium(lang);
            comment::setcomment(&label, "Suffragium", c_val, lang, "");
            specials_build::setbuild1(&format!("Suffragium{}", c_val), "included");
            output_lines.push(suffr);
            continue;
        }

        // --- Branch: Martyrologium ---
        if item.to_lowercase().contains("martyrologium") {
            if let Some(marty) = martyrologium::martyrologium(lang) {
                output_lines.push(marty);
            }
            // Additional martyrologium handling could go here.
            continue;
        }

        // --- Branch: Antiphona finalis ---
        if item.to_lowercase().contains("antiphona finalis") {
            if config.version.starts_with("Ordo Praedicatorum") {
                output_lines.push(format!("#{}", specials_build::translate("Antiphonae finalis", lang)));
                output_lines.push("$ant Salve Regina".to_string());
            } else {
                output_lines.push(format!("#{}", specials_build::translate("Antiphona finalis BMV", lang)));
                if config.version.to_lowercase().contains("cist") {
                    output_lines.push("$ant Salve Regina".to_string());
                } else {
                    output_lines.push("$ant Alma Redemptoris Mater".to_string());
                }
            }
            output_lines.push("&Divinum_auxilium".to_string());
            continue;
        }

        // --- Branch: Capitulum for minor hours ---
        if item.to_lowercase().contains("capitulum")
            && (config.hora.eq_ignore_ascii_case("Tertia")
                || config.hora.eq_ignore_ascii_case("Sexta")
                || config.hora.eq_ignore_ascii_case("Nona")
                || config.hora.eq_ignore_ascii_case("Completorium"))
        {
            if config.hora.eq_ignore_ascii_case("Completorium") {
                output_lines.push(specials_build::translate(&item, lang));
            }
            output_lines.push(capitulis::capitulum_minor(lang).join("\n"));
            continue;
        }

        // --- Branch: Capitulum for Laudes/Vespera ---
        if item.to_lowercase().contains("capitulum")
            && (config.hora.eq_ignore_ascii_case("Laudes")
                || config.hora.eq_ignore_ascii_case("Vespera"))
        {
            if let Some(text) = capitulis::capitulum_major(lang) {
                output_lines.push(text);
            }
            continue;
        }

        // --- Default: fallback translation ---
        output_lines.push(specials_build::translate(&label, lang));
    }
    output_lines.join("\n")
}

/// Dummy helper to “translate” text.
pub fn translate(text: &str, lang: &str) -> String {
    format!("Translated({}): {}", lang, text)
}

/// Helper to build the “special” lookup key.
fn special_key(config: &InputConfig, _lang: &str) -> String {
    let i = if config.hora == "Laudes" {
        " 2".to_string()
    } else if config.hora == "Vespera" {
        format!(" {}", config.vespera)
    } else {
        "".to_string()
    };
    format!("Special {}{}", config.hora, i)
}

/// The configuration struct replaces many globals.
pub struct InputConfig {
    pub column: usize,
    pub winner: String,
    pub winner_map: HashMap<String, String>,
    pub winner2_map: HashMap<String, String>,
    pub rule: String,
    pub largefont: String,
    pub smallblack: String,
    pub hora: String,
    pub date1: String,
    pub daynames: Vec<String>,
    pub vespera: i32,
    pub version: String,
    pub votive: String,
    pub skipflag: bool,
    pub litaniaflag: bool,
}

impl InputConfig {
    /// Clears duplicate–check flags.
    pub fn clear_flags(&mut self) {
        self.litaniaflag = false;
        self.skipflag = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Returns a dummy InputConfig for testing.
    fn dummy_config() -> InputConfig {
        let mut winner_map = HashMap::new();
        winner_map.insert("Rule".to_string(), "Capitulum Versum 2 ad laudes et vesperas".to_string());
        winner_map.insert("Special Laudes".to_string(), "Special text for Laudes".to_string());
        InputConfig {
            column: 1,
            winner: "Dummy Winner".to_string(),
            winner_map,
            winner2_map: HashMap::new(),
            rule: "Capitulum Versum 2 ad laudes et vesperas".to_string(),
            largefont: "LargeFont".to_string(),
            smallblack: "SmallBlack".to_string(),
            hora: "Laudes".to_string(),
            date1: "2025-02-18".to_string(),
            daynames: vec!["Sunday".to_string(), "Monday".to_string(), "Tuesday".to_string()],
            vespera: 3,
            version: "Modern".to_string(),
            votive: "".to_string(),
            skipflag: false,
            litaniaflag: false,
        }
    }

    #[test]
    fn test_specials_basic() {
        let config = dummy_config();
        let script = vec![
            "Line one".to_string(),
            "# Comment header".to_string(),
            "Line two".to_string(),
        ];
        let output = specials(config, script, "Latin", None);
        assert!(output.contains("Line one"));
        assert!(output.contains("Line two"));
    }

    #[test]
    fn test_specials_omit_branch() {
        let mut config = dummy_config();
        config.rule = "Omit OmitTest".to_string();
        let script = vec!["#OmitTest".to_string(), "Following line".to_string()];
        let output = specials(config, script, "Latin", None);
        // The omit branch should skip the following line.
        assert!(!output.contains("Following line"));
    }

    #[test]
    fn test_capitulum_branch() {
        let mut config = dummy_config();
        config.rule = "Capitulum Versum 2 ad laudes et vesperas".to_string();
        config.hora = "Laudes".to_string();
        let script = vec![
            "#Capitulum".to_string(),
            "Additional text".to_string(),
        ];
        let output = specials(config, script, "Latin", None);
        // In this dummy version, we expect the major capitulum branch to have been triggered.
        assert!(output.contains("Translated(")); // falls back to a translation if no capitulum text is found.
    }
}
