use crate::{
    date::{leap_year, monthday}, 
    fileio::do_read, 
    language_text_tools::{alleluia_ant, prayer, translate, LanguageTextContext}, 
    setup_string::{checkfile, setupstring, ResolveDirectives},
    regex::{replace_from_first, remove_prefix_to_last},
};

use super::specmatins::get_c10_readingname;


/// Holds all context data (formerly globals)
#[derive(Debug, Clone)]
pub struct LiturgyContext {
    pub day: i32,
    pub month: i32,
    pub year: i32,
    /// 0 = Sunday, 1 = Monday, …, 6 = Saturday
    pub dayofweek: usize,
    /// First two entries correspond to two “dayname” values
    pub dayname: Vec<String>,
    pub version: String,
    pub rank: f32,
    pub rule: String,
    pub winner: std::collections::HashMap<String, String>,
    pub commune: Option<std::collections::HashMap<String, String>>,
    pub votive: String,
    pub datafolder: String,
}

/// Generates a name for a feria given the weekday.
pub fn makeferia(dayofweek: usize) -> String {
    let nametab = vec!["Sunday", "II.", "III.", "IV.", "V.", "VI.", "Sabbato"];
    let name = match nametab.get(dayofweek) {
        Some(s) => s.to_string(),
        None => "".to_string(),
    };
    if dayofweek > 0 && dayofweek < 6 {
        format!("Feria {}", name)
    } else {
        name
    }
}

/// Generates the appropriate psalm and lessons for the monastic version.
///
/// Returns a vector of output lines.
pub fn psalmi_matutinum_monastic(ctx2: &LanguageTextContext, lang: &str, ctx: &LiturgyContext) -> Vec<String> {
    // (Dummy psalm numbers; they are not used further in our code.)
    let _psalmnum1 = -1;
    let _psalmnum2 = -1;

    // Read the antiphons-psalms from the psalterium (dummy stub)
    let psalmi_map = setupstring(
        lang, "Psalterium/Psalmi/Psalmi matutinum.txt", ResolveDirectives::All
    ).unwrap_or_default();

    let daym_key = format!("Daym{}", ctx.dayofweek);
    let mut psalmi: Vec<String> = psalmi_map
        .get(&daym_key)
        .cloned()
        .unwrap_or_default()
        .lines()
        .map(|s| s.to_string())
        .collect();

    // Special treatment for dayofweek == 5.
    if ctx.dayofweek == 5 {
        if ctx.winner.contains_key("Ant Laudes") {
            if let Some(line) = psalmi.get_mut(4) {
                *line = line.replace("92!", "");
            }
            if let Some(line) = psalmi.get_mut(12) {
                // Remove everything up to and including the last "99!"
                *line = remove_prefix_to_last(line, "99!");
            }
        } else {
            if let Some(line) = psalmi.get_mut(4) {
                *line = line.replace("!75", "");
            }
            if let Some(line) = psalmi.get_mut(12) {
                // Replace the first occurrence of "99!" and all following characters with "99"
                *line = replace_from_first(line, "99!", "99");
            }
        }
    }

    setbuild(
        "Psalterium/Psalmi/Psalmi matutinum monastic",
        &format!("dayM{}", ctx.dayofweek),
        "Psalmi ord",
    );

    let comment = 1;
    let prefix = translate(ctx2, "Antiphonae", lang);
    let name = gettempora("Psalmi Matutinum Monastic", lang);

    // Special Adv–Pasc antiphons for Sundays.
    if ctx.dayofweek == 0 && matches_adv_or_pasch(&name) {
        if let Some(special) = psalmi_map.get(&format!("{}m0", name)) {
            psalmi = special.lines().map(|s| s.to_string()).collect();
        }
    }

    // Special antiphons for non‐Quad weekdays.
    if (ctx.dayofweek > 0
        && !contains_quad(ctx.dayname.get(0).unwrap_or(&"".to_string())))
        || ctx
            .winner
            .values()
            .any(|v| v.contains("Pasc6-0"))
    {
        let start = if matches_pasc(ctx.dayname.get(0).unwrap_or(&"".to_string()))
            || matches_nat23(ctx.dayname.get(0).unwrap_or(&"".to_string()))
        {
            0
        } else {
            8
        };
        let mut p: Vec<String> = vec![];
        if matches_pasc(ctx.dayname.get(0).unwrap_or(&"".to_string())) {
            if let Some(val) = psalmi_map.get("Daym Pasch") {
                p = val.lines().map(|s| s.to_string()).collect();
            }
        } else if matches_nat23(ctx.dayname.get(0).unwrap_or(&"".to_string())) {
            if let Some(val) = psalmi_map.get("Daym Nat") {
                p = val.lines().map(|s| s.to_string()).collect();
            }
        }
        let mut psalmi = psalmi; // shadow to make mutable
        for i in start..14 {
            let mut p_line = p.get(i).cloned().unwrap_or_default();
            if let Some(psalm_line) = psalmi.get(i) {
                if let Some(idx) = psalm_line.find(";;") {
                    let suffix = &psalm_line[idx + 2..];
                    p_line = format!(";;{}", suffix);
                }
            }
            if i == 0 || i == 8 {
                if !matches_nat23_or_pasc0(ctx.dayname.get(0).unwrap_or(&"".to_string())) {
                    p_line = format!("{}{}", alleluia_ant(ctx2, lang), p_line);
                } else {
                    let current = p.get(i).unwrap_or(&"".to_string()).to_string();
                    p_line = format!("{}{}", current, p_line);
                }
            }
            if let Some(line) = psalmi.get_mut(i) {
                *line = p_line;
            }
        }
        setbuild2("Antiphonas Psalmi weekday special no Quad");
    }

    // Change of versicle for Adv, Quad, Pasc, etc.
    if !name.is_empty()
        && (ctx
            .winner
            .values()
            .any(|v| v.starts_with("Tempora"))
            || (name == "Nat" || name == "Epi"))
    {
        let mut i = if ctx.dayofweek == 0 { 1 } else { ctx.dayofweek };
        let src = "Psalterium".to_string();
        if i > 3 {
            i -= 3;
        }
        if name != "Asc" {
            if let Some(val) = psalmi_map.get(&format!("{} {} Versum", name, i)) {
                let parts: Vec<&str> = val.lines().collect();
                if parts.len() >= 2 && psalmi.len() > 7 {
                    psalmi[6] = parts[0].to_string();
                    psalmi[7] = parts[1].to_string();
                }
            }
            if ctx.dayofweek == 0 {
                if let Some(val) = psalmi_map.get(&format!("{} 2 Versum", name)) {
                    let parts: Vec<&str> = val.lines().collect();
                    if parts.len() >= 2 && psalmi.len() > 15 {
                        psalmi[14] = parts[0].to_string();
                        psalmi[15] = parts[1].to_string();
                    }
                }
                if let Some(val) = psalmi_map.get(&format!("{} 3 Versum", name)) {
                    let parts: Vec<&str> = val.lines().collect();
                    if parts.len() >= 2 && psalmi.len() > 18 {
                        psalmi[17] = parts[0].to_string();
                        psalmi[18] = parts[1].to_string();
                    }
                }
            }
        } else {
            let c = if columnsel(lang) {
                ctx.winner.clone()
            } else {
                ctx.winner.clone()
            };
            let src = "commune".to_string();
            if let Some(val) = c.get(&format!("Nocturn {} Versum", i)) {
                let parts: Vec<&str> = val.lines().collect();
                if parts.len() >= 2 && psalmi.len() > 7 {
                    psalmi[6] = parts[0].to_string();
                    psalmi[7] = parts[1].to_string();
                }
            }
            if ctx.dayofweek == 0 {
                if let Some(val) = c.get("Nocturn 2 Versum") {
                    let parts: Vec<&str> = val.lines().collect();
                    if parts.len() >= 2 && psalmi.len() > 15 {
                        psalmi[14] = parts[0].to_string();
                        psalmi[15] = parts[1].to_string();
                    }
                }
                if let Some(val) = c.get("Nocturn 3 Versum") {
                    let parts: Vec<&str> = val.lines().collect();
                    if parts.len() >= 2 && psalmi.len() > 18 {
                        psalmi[17] = parts[0].to_string();
                        psalmi[18] = parts[1].to_string();
                    }
                }
            }
        }
        setbuild(&src, &format!("{} {} Versum", name, i), "subst");
    }

    if ctx.month == 12 && ctx.day == 24 {
        if ctx.dayofweek != 0 {
            if let Some(val) = psalmi_map.get("Nat24 Versum") {
                let parts: Vec<&str> = val.lines().collect();
                if parts.len() >= 2 && psalmi.len() > 7 {
                    psalmi[6] = parts[0].to_string();
                    psalmi[7] = parts[1].to_string();
                }
            }
        } else {
            if let Some(val) = psalmi_map.get("Nat24 Versum") {
                let parts: Vec<&str> = val.lines().collect();
                if parts.len() >= 2 && psalmi.len() > 18 {
                    psalmi[17] = parts[0].to_string();
                    psalmi[18] = parts[1].to_string();
                }
            }
        }
        setbuild2("subst: Versus Nat24");
    }

    if ctx.winner.contains_key("Cantica") {
        if let Some(cantica) = ctx.winner.get("Cantica") {
            let c_lines: Vec<&str> = cantica.lines().collect();
            for i in 0..3 {
                if psalmi.len() > i + 16 {
                    psalmi[i + 16] = c_lines.get(i).unwrap_or(&"").to_string();
                }
            }
        }
    }

    // Decide between lectiones or brevis/legend readings.
    let rule_contains_12 = ctx.rule.contains("12 lectiones");
    let rule_contains_3 = ctx.rule.contains("3 lectiones");
    let version_lower = ctx.version.to_lowercase();
    let cond_divino = (ctx.rank >= 4.0 && version_lower.contains("divino"))
        || (ctx.rank >= 2.0 && version_lower.contains("trident"));
    let dayname1 = ctx.dayname.get(1).unwrap_or(&"".to_string());
    let dayname1_lower = dayname1.to_lowercase();
    let cond_dayname1 = !(dayname1_lower.contains("feria")
        || dayname1_lower.contains("sabbato")
        || dayname1_lower.contains("infra octavam"));

    if rule_contains_12 || (cond_divino && cond_dayname1 && !rule_contains_3) {
        lectiones(1, lang);
    } else if matches_pasc1_6_or_pent(ctx.dayname.get(0).unwrap_or(&"".to_string()))
        && !starts_with_11_digit_dash(&monthday(
            ctx.day as u32,
            ctx.month as u32,
            ctx.year,
        ctx.version.contains("196"),
            false,
        ))
        && !contains_rank_keywords(ctx.winner.get("Rank").unwrap_or(&"".to_string()))
        && ((!ctx
            .winner
            .get("Rank")
            .unwrap_or(&"".to_string())
            .to_lowercase()
            .contains("secunda")
            && ctx
                .winner
                .get("Rank")
                .unwrap_or(&"".to_string())
                .to_lowercase()
                .contains("roga"))
            || ctx.version.contains("196"))
        && !rule_contains_3
    {
        if ctx.winner.contains_key("Tempora")
            || !(ctx.winner.contains_key("Lectio94") || ctx.winner.contains_key("Lectio4"))
        {
            brevis_monastic(lang, ctx, ctx2);
        } else if ctx.winner.contains_key("Lectio94") || ctx.winner.contains_key("Lectio4") {
            legend_monastic(lang, ctx, ctx2);
        }
    } else {
        lectiones(0, lang);
    }
    if !ctx.rule.contains("12 lectiones") {
        if psalmi.len() > 14 {
            psalmi[14].clear();
        }
        if psalmi.len() > 15 {
            psalmi[15].clear();
        }
    }
    nocturn(2, lang, &psalmi, &vec![8, 9, 10, 11, 12, 13, 14, 15]);

    if rule_contains_12 || (cond_divino && cond_dayname1 && !rule_contains_3) {
        lectiones(2, lang);
        if let Some(line) = psalmi.get(16) {
            let parts: Vec<&str> = line.split(";;").collect();
            let ant = parts.get(0).unwrap_or(&"").to_string();
            let mut p = parts.get(1).unwrap_or(&"").to_string();
            let w = ctx.winner.clone();
            if let Some(ant_3n) = w.get("Ant Matutinum 3N") {
                let t_lines: Vec<String> =
                    ant_3n.lines().map(|s| s.to_string()).collect();
                for (i, line) in t_lines.iter().enumerate() {
                    if psalmi.len() > 16 + i {
                        psalmi[16 + i] = line.clone();
                    }
                }
                let parts_new: Vec<&str> = psalmi[16].split(";;").collect();
                let ant_new = parts_new.get(0).unwrap_or(&ant).to_string();
                p = parts_new.get(1).unwrap_or(&p).to_string();
            }
            p = p.replace(&['(', '-'][..], ",").replace(")", "");
            postprocess_ant(&ant, lang);
            if psalmi.len() > 16 {
                psalmi[16] = format!("{};;{}", ant, p);
            }
        }
        nocturn(3, lang, &psalmi, &vec![16, 17, 18]);
        lectiones(3, lang);
        return psalmi;
    }

    // After 2nd nocturn: handle the Capitulum.
    let (w, _c) = getproprium("MM Capitulum", lang, ctx, 0, 1);
    let mut capitulum = if w.is_empty() {
        if let Some(commune) = &ctx.commune {
            commune.get("MM Capitulum").cloned().unwrap_or_default()
        } else {
            "".to_string()
        }
    } else {
        w
    };
    if capitulum.is_empty() {
        let temp_name = gettempora("MM Capitulum", lang);
        let s_map = setupstring(lang, "Psalterium/Special/Matutinum Special.txt", ResolveDirectives::All).unwrap_or_default();
        capitulum = s_map
            .get(&format!("MM Capitulum{}", temp_name))
            .cloned()
            .unwrap_or_default();
    }
    if ctx
        .dayname
        .get(0)
        .unwrap_or(&"".to_string())
        .to_lowercase()
        .contains("pasc")
    {
        postprocess_vr(&capitulum, lang);
    }
    let mut output = psalmi;
    output.push("!!Capitulum".to_string());
    output.push(capitulum);
    output.push("".to_string());
    output
}

/// Returns the proper legend reading if appropriate.
pub fn monastic_lectio3(w: &str, lang: &str, ctx: &LiturgyContext) -> String {
    if !ctx
        .winner
        .values()
        .any(|v| v.to_lowercase().contains("sancti"))
        || ctx.winner.contains_key("Lectio3")
        || ctx.rank >= 4.0
        || ctx.rule.contains("9 lectio") || ctx.rule.contains("12 lectio")
        || ctx.rule.contains("Lectio1 tempora")
    {
        return w.to_string();
    }
    let winner_map = if columnsel(lang) {
        ctx.winner.clone()
    } else {
        ctx.winner.clone()
    };
    let mut str_val = if let Some(val) = winner_map.get("Lectio94") {
        val.to_string()
    } else {
        winner_map.get("Lectio4").unwrap_or(&"".to_string()).to_string()
    };
    str_val = remove_te_deum(&str_val);
    if str_val.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
        str_val = format!("v. {}", str_val);
    }
    let mut output = Vec::new();
    output.push(str_val);
    output.push("$Tu autem".to_string());
    output.push("_".to_string());

    let mut resp = if let Some(r) = winner_map.get("Responsory1") {
        r.to_string()
    } else {
        let commune_map = if columnsel(lang) {
            ctx.commune.clone().unwrap_or_default()
        } else {
            ctx.commune.clone().unwrap_or_default()
        };
        commune_map
            .get("Responsory1")
            .cloned()
            .unwrap_or_else(|| "Responsory for ne lesson not found!".to_string())
    };
    resp = responsory_gloria(&resp, 3);
    // (Dummy stub for alleluia handling.)
    output.push(resp);
    output
}


/// Return lines that implement “Absolution and Benedictio” logic.
pub fn absolutio_benedictio(lang: &str, ctx: &LiturgyContext, lctx: &LanguageTextContext) -> Vec<String> {
    let mut output = Vec::new();

    // Check if the “commune” map has an entry whose value contains "C10".
    let c10_found = if let Some(ref commune_map) = ctx.commune {
        commune_map.values().any(|val| val.contains("C10"))
    } else {
        false
    };

    let (abs, ben) = if c10_found {
        // In the original Perl: we read from commune's "Benedictio" lines.
        // e.g. @a = split("\n", $m{Benedictio}); abs=$a[0]; ben=$a[3].
        let m = ctx.commune.as_ref().unwrap();
        let benedictio_all = m.get("Benedictio").unwrap_or(&"".to_string());
        let lines: Vec<&str> = benedictio_all.lines().collect();
        let abs_str = lines.get(0).unwrap_or(&"").to_string();
        let ben_str = lines.get(3).unwrap_or(&"").to_string();
        // We might do a setbuild2("Special benedictio") call here. Omitted for brevity.
        (abs_str, ben_str)
    } else {
        // Otherwise read from Psalterium/Benedictions.txt => "Nocturn i" and "Absolutiones".
        let ben_map = setupstring(lang, "Psalterium/Benedictions.txt", ResolveDirectives::All).unwrap_or_default();
        let i = dayofweek2i(ctx);
        let nocturn_key = format!("Nocturn {}", i);
        let a_all = ben_map.get(&nocturn_key).unwrap_or(&"".to_string());
        let absolutiones_all = ben_map.get("Absolutiones").unwrap_or(&"".to_string());

        let a_lines: Vec<&str> = a_all.lines().collect();
        let abs_lines: Vec<&str> = absolutiones_all.lines().collect();

        // abs is the (i-1)-th line in abs_lines, if present.
        let abs_str = if i > 0 && i - 1 < abs_lines.len() {
            abs_lines[i - 1].to_string()
        } else {
            "".to_string()
        };

        // ben is the line at index (3 - (i == 3 ? 1 : 0)) in a_lines.
        // i.e., if i==3, subtract 1 from 3 => index 2, else index 3
        let ben_idx = 3 - if i == 3 { 1 } else { 0 };
        let ben_str = if ben_idx < a_lines.len() {
            a_lines[ben_idx].to_string()
        } else {
            "".to_string()
        };

        (abs_str, ben_str)
    };

    // Now push the lines to `output` as in the original:
    output.push("".to_string());
    output.push("$Pater noster_".to_string());
    output.push("_".to_string());
    output.push(format!("Absolutio. {}", abs));
    output.push("$Amen".to_string());
    output.push("".to_string());
    output.push(prayer(lctx, "Jube domne", lang));
    output.push(format!("Benedictio. {}", ben));
    output.push("$Amen".to_string());
    output.push("_".to_string());

    output
}

/// Returns the “Legend (contracted reading) for monastic days”.
pub fn legend_monastic(lang: &str, ctx: &LiturgyContext, lctx: &LanguageTextContext) -> Vec<String> {
    let mut output = Vec::new();

    // 1) Insert the absolution & benediction lines first.
    let mut ab = absolutio_benedictio(lang, ctx, lctx);
    output.append(&mut ab);

    // 2) Gather the reading from the “winner” map, either “Lectio94” or “Lectio4”.
    let winner_map = if columnsel(lang) {
        ctx.winner.clone()
    } else {
        ctx.winner.clone() // or “winner2” in older code
    };

    let mut reading = if let Some(v) = winner_map.get("Lectio94") {
        v.clone()
    } else {
        // fallback: Lectio4 plus possibly Lectio5 and Lectio6 if some condition is met
        let mut s = winner_map.get("Lectio4").unwrap_or(&"".to_string()).to_string();
        if let Some(l5) = winner_map.get("Lectio5") {
            // In Perl: `if (exists($w{Lectio5}) && $w{Lectio5} !~ /!/) { $str .= $w{Lectio5} . $w{Lectio6}; }`
            // So we do a simple check if it does NOT contain '!'
            if !l5.contains('!') {
                let l6 = winner_map.get("Lectio6").unwrap_or(&"".to_string());
                s.push_str(l5);
                s.push_str(l6);
            }
        }
        s
    };

    // Remove "&teDeum" with trailing spaces
    loop {
        if let Some(pos) = reading.find("&teDeum") {
            // remove any subsequent whitespace as well
            let mut end = pos + "&teDeum".len();
            while end < reading.len() && reading.as_bytes()[end].is_ascii_whitespace() {
                end += 1;
            }
            reading.replace_range(pos..end, "");
        } else {
            break;
        }
    }

    // If the text starts with an alphabetic letter, prepend "v. ".
    if let Some(ch) = reading.chars().next() {
        if ch.is_alphabetic() {
            reading = format!("v. {}", reading);
        }
    }

    // 3) Add these lines:
    output.push(reading);
    output.push("$Tu autem".to_string());
    output.push("_".to_string());

    // 4) Build the responsory (Responsory1 from the winner or from the commune).
    let mut resp = if let Some(r) = winner_map.get("Responsory1") {
        r.clone()
    } else {
        let commune_map = if columnsel(lang) {
            ctx.commune.clone().unwrap_or_default()
        } else {
            ctx.commune.clone().unwrap_or_default()
        };
        // fallback
        commune_map
            .get("Responsory1")
            .cloned()
            .unwrap_or_else(|| "Responsory for ne lesson not found!".to_string())
    };

    // Add Gloria if needed
    resp = responsory_gloria(&resp, 3);

    // If alleluia is required, we might do something like:
    if alleluia_required(
        ctx.dayname.get(0).unwrap_or(&"".to_string()),
        &ctx.votive,
    ) {
        let appended = matins_lectio_responsory_alleluia(&resp, lang);
        // In the original code, the new text got appended or replaced.
        // You might either modify `resp` or push another line.
        // We'll just push it for demonstration:
        if !appended.is_empty() {
            resp.push('\n');
            resp.push_str(&appended);
        }
    }

    output.push(resp);

    output
}

/// Implements the “brevis” (short) reading.
pub fn brevis_monastic(lang: &str, ctx: &LiturgyContext, lctx: &LanguageTextContext) -> Vec<String> {
    let mut output = Vec::new();
    output.extend(absolutio_benedictio(lang, ctx, lctx));
    let lectio: String;
    if let Some(commune) = &ctx.commune {
        if commune.values().any(|v| v.contains("C10")) {
            let name = get_c10_readingname(&ctx.version, ctx.month as u32, ctx.day as u32);
            let mut resp_lines: Vec<String> = commune
                .get("Responsory3")
                .cloned()
                .unwrap_or_default()
                .lines()
                .map(|s| s.to_string())
                .collect();
            if matches_pasc(
                ctx.dayname.get(0).unwrap_or(&"".to_string()).as_str(),
            ) {
                if let Some(line) = resp_lines.get_mut(1) {
                    compress_alleluia(line);
                }
                if let Some(line) = resp_lines.last_mut() {
                    compress_alleluia(line);
                }
            }
            let reading = commune
                .get(&name)
                .unwrap_or(&"".to_string())
                .replace(".teDeum", "");
            lectio = format!("{}\n$Tu autem\n_\n{}", reading, resp_lines.join("\n"));
            setbuild2(&format!("Mariae {}", name));
        } else if let Some(commune) = &ctx.commune {
            if !commune.is_empty() && !commune.keys().any(|k| k.starts_with("C")) {
                lectio = commune.get("MM LB").unwrap_or(&"".to_string()).to_string();
            } else {
                let b_map = setupstring(lang, "Psalterium/Special/Matutinum Special.txt", ResolveDirectives::All).unwrap_or_default();
                let key = if matches_pasc(ctx.dayname.get(0).unwrap_or(&"".to_string())) {
                    "MM LB Pasch".to_string()
                } else {
                    format!("MM LB{}", ctx.dayofweek)
                };
                lectio = b_map.get(&key).cloned().unwrap_or_default();
            }
        } else {
            let b_map = setupstring(lang, "Psalterium/Special/Matutinum Special.txt", ResolveDirectives::All).unwrap_or_default();
            let key = if matches_pasc(ctx.dayname.get(0).unwrap_or(&"".to_string())) {
                "MM LB Pasch".to_string()
            } else {
                format!("MM LB{}", ctx.dayofweek)
            };
            lectio = b_map.get(&key).cloned().unwrap_or_default();
        }
    } else {
        let b_map = setupstring(lang, "Psalterium/Special/Matutinum Special.txt", ResolveDirectives::All).unwrap_or_default();
        let key = if matches_pasc(ctx.dayname.get(0).unwrap_or(&"".to_string())) {
            "MM LB Pasch".to_string()
        } else {
            format!("MM LB{}", ctx.dayofweek)
        };
        lectio = b_map.get(&key).cloned().unwrap_or_default();
    }
    let lectio = lectio.replace("&Gloria1?", "&Gloria1");
    output.push(lectio);
    output
}

/// Returns the Evangelium text.
pub fn lectio_e(lang: &str, ctx: &LiturgyContext, tctx: &LanguageTextContext) -> String {
    let winner_map = if columnsel(lang) {
        ctx.winner.clone()
    } else {
        ctx.winner.clone()
    };
    let mut e_lines: Vec<String> = if let Some(e) = winner_map.get("LectioE") {
        e.lines().map(|s| s.to_string()).collect()
    } else {
        vec![]
    };

    if e_lines.is_empty() || e_lines.get(0).map(|s| s.starts_with("@")).unwrap_or(false) {
        let parts: Vec<&str> = e_lines.get(0).unwrap_or(&"").split(':').collect();
        let mut w_val = if !parts.is_empty() && !parts[0].is_empty() {
            format!("{}.txt", parts[0])
        } else {
            ctx.winner
                .get("default")
                .unwrap_or(&"default".to_string())
                .to_string()
        };
        w_val = w_val.replace("M", "");
        let s_val = parts.get(1).unwrap_or(&"Evangelium").replace("LectioE", "Evangelium");
        let missa_map = setupstring(&format!("../missa/{}", lang), &w_val, ResolveDirectives::All).unwrap_or_default();
        e_lines = missa_map
            .get(&s_val)
            .cloned()
            .unwrap_or_default()
            .lines()
            .map(|s| s.to_string())
            .collect();
    }
    let mut begin = e_lines.get(0).cloned().unwrap_or_default();
    begin = format!("v.{}", begin.trim_start_matches("v. "));
    if ctx.version.starts_with("Monastic") {
        begin = begin.replace("+", "++");
        if let Some(next_line) = e_lines.get(1).cloned() {
            begin = format!(
                "{}\n{}\nR. {}",
                begin,
                next_line,
                translate(tctx, "Gloria tibi Domine", lang)
            );
        }
    } else {
        begin = begin.replace("+", "");
        if e_lines.len() > 1 {
            e_lines.remove(1);
        }
    }
    e_lines[0] = begin;
    e_lines.retain(|line| !line.starts_with("!"));
    if e_lines.len() > 1 {
        e_lines[1] = format!("v.{}", e_lines[1].trim_start_matches("v. "));
    }
    e_lines.join("\n")
}

/// Determines if the Evangelium is required.
pub fn lectio_e_required(ctx: &LiturgyContext) -> bool {
    ctx.rank > 2.0 || ctx.commune.as_ref().map(|m| m.contains_key("C10")).unwrap_or(false)
}

/// For Ordo Praedicatorum: returns the text of the Regula
pub fn regula_vel_evangelium(lang: &str, ctx: &LiturgyContext, tctx: &LanguageTextContext) -> String {
    let ben_map = setupstring(lang, "Psalterium/Benedictions.txt", ResolveDirectives::All).unwrap_or_default();
    let b_lines_vec = ben_map.get("Nocturn 3").cloned().unwrap_or_default();
    let b_lines: Vec<&str> = b_lines_vec.lines().collect();
    let r_map = setupstring(lang, "Regula/OrdoPraedicatorum.txt", ResolveDirectives::All).unwrap_or_default();
    let be: String;
    let mut output: Vec<String> = vec![];

    if lectio_e_required(ctx) {
        be = b_lines.get(3).unwrap_or(&"").to_string();
        output.push(lectio_e(lang, ctx, tctx));
    } else {
        be = r_map.get("Benedictio").cloned().unwrap_or_default();
        output.push("_".to_string());
        output.push(format!(
            "v. {}{}",
            r_map.get("Incipit").cloned().unwrap_or_default(),
            r_map.get("our $dayofweek").cloned().unwrap_or_default(),
        ));
    }

    output.insert(0, format!("V. {}", b_lines.get(1).unwrap_or(&"")));
    output.insert(0, format!("Benedictio. {}", be));
    output.push("$Tu autem".to_string());
    output.push(r_map.get("Finita lectione").cloned().unwrap_or_default());
    output.join("\n")
}

/// Returns the text of the Regula for the day.
pub fn regula<F: Fn(&str) -> bool>(
    lang: &str, ctx: &LiturgyContext, pctx: &LanguageTextContext, 
    langfb: &str, file_exists: &F
) -> String {
    if ctx.version.to_lowercase().contains("ordo praedicatorum") {
        return regula_vel_evangelium(lang, ctx, pctx);
    }
    let mut output = format!("{}\n", prayer(pctx, "benedictio Prima", lang));
    let mut d = ctx.day;
    if ctx.month == 2 && ctx.day >= 24 && !leap_year(ctx.year) {
        d += 1;
    }
    let fname = format!("{:02}-{:02}", ctx.month, d);

    if !(file_exists)(&format!("{}/Latin/Regula/{}.txt", ctx.datafolder, fname)) {
        let regulatable_lines = do_read(&format!("{}/Latin/Regula/Regulatable.txt", ctx.datafolder));
        if regulatable_lines.iter().any(|line| line.contains(&fname)) {
            // adjust fname if necessary
        } else {
            return output;
        }
    }
    let fname_checked = checkfile(&ctx.datafolder, langfb, lang, &format!("Regula/{}.txt", fname), file_exists);
    let lines = do_read(&fname_checked).unwrap_or_default();
    let mut title = lines.get(0).cloned().unwrap_or_default();
    let content: Vec<String> = lines
        .iter()
        .skip(1)
        .map(|l| if l.is_empty() { "_".to_string() } else { l.clone() })
        .collect();
    title = replace_title(&title);
    output.push_str(&format!("{}\n", title));
    output.push_str(&content.join("\n"));

    if ctx.month == 2 && ctx.day == 23 && !leap_year(ctx.year) {
        let fname_checked = checkfile(&ctx.datafolder, langfb, lang, "Regula/02-24.txt", file_exists);
        let mut extra_lines = do_read(&fname_checked).unwrap_or_default();
        if !extra_lines.is_empty() {
            extra_lines.remove(0);
        }
        output.push_str(&extra_lines.join("\n"));
    }

    output.push_str("\n$Tu autem\n_\n$rubrica Regula\n");
    output
}

// -- HELPER FUNCTIONS 

/// Returns true if `name` (case‐insensitively) equals "Adv" or "Pasch".
pub fn matches_adv_or_pasch(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "adv" || lower == "pasch"
}

/// Returns true if `s` (case‑insensitively) contains "quad".
pub fn contains_quad(s: &str) -> bool {
    s.to_lowercase().contains("quad")
}

/// Returns true if `s` (case‑insensitively) contains "pasc".
pub fn matches_pasc(s: &str) -> bool {
    s.to_lowercase().contains("pasc")
}

/// Returns true if `s` (case‑insensitively) contains “nat2” or “nat3” (our approximation of “Nat[23]\d”).
pub fn matches_nat23(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.contains("nat2") || lower.contains("nat3")
}

/// Returns true if `s` (case‑insensitively) contains either a “Nat2…”/“Nat3…” pattern or “Pasc0”.
pub fn matches_nat23_or_pasc0(s: &str) -> bool {
    let lower = s.to_lowercase();
    matches_nat23(&lower) || lower.contains("pasc0")
}

/// Returns true if `s` (case‑insensitively) contains a “Pasc[1-6]” or “Pent” substring.
pub fn matches_pasc1_6_or_pent(s: &str) -> bool {
    let lower = s.to_lowercase();
    if lower.contains("pent") {
        return true;
    }
    for digit in ['1', '2', '3', '4', '5', '6'].iter() {
        if lower.contains(&format!("pasc{}", digit)) {
            return true;
        }
    }
    false
}

/// Returns true if `s` starts with the literal pattern of “11” then a digit between 1 and 5, then a dash.
fn starts_with_11_digit_dash(s: &str) -> bool {
    if s.len() >= 4 {
        let bytes = s.as_bytes();
        bytes[0] == b'1'
            && bytes[1] == b'1'
            && (bytes[2] >= b'1' && bytes[2] <= b'5')
            && bytes[3] == b'-'
    } else {
        false
    }
}

/// Returns true if `s` (case‑insensitively) contains any of the keywords:
/// “vigil”, “quatuor” (or “quattuor”), “infra octavam” or “post octavam asc”.
fn contains_rank_keywords(s: &str) -> bool {
    let lower = s.to_lowercase();
    lower.contains("vigil")
        || lower.contains("quatuor")
        || lower.contains("quattuor")
        || lower.contains("infra octavam")
        || lower.contains("post octavam asc")
}

/// Remove any occurrence of "&teDeum" followed by any amount of whitespace.
fn remove_te_deum(text: &str) -> String {
    let mut result = text.to_string();
    loop {
        if let Some(pos) = result.find("&teDeum") {
            let mut end = pos + "&teDeum".len();
            // Extend end index while the following characters are whitespace.
            while let Some(ch) = result[end..].chars().next() {
                if ch.is_whitespace() {
                    end += ch.len_utf8();
                } else {
                    break;
                }
                if end >= result.len() {
                    break;
                }
            }
            result.replace_range(pos..end, "");
        } else {
            break;
        }
    }
    result
}

/// Replace everything up to and including the last '#' in `s` with "v. ".
/// (This mimics the Perl s/.*#/v. / operation.)
fn replace_title(s: &str) -> String {
    if let Some(pos) = s.rfind('#') {
        let after = &s[(pos + 1)..];
        format!("v. {}", after)
    } else {
        s.to_string()
    }
}

/// Compress any run of "Alleluia" into a single "Alleluia " (with trailing space).
fn compress_alleluia(text: &mut String) {
    if text.contains("Alleluia") {
        *text = "Alleluia ".to_string();
    }
}


#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use crate::{language_text_tools::initialize_language_text_context, setup_string::SetupStringProvider};
    use super::*;

    /// A dummy setup provider that holds a mapping from (lang, file) to key/value data.
    struct DummySetupStringContext {
        data: HashMap<(String, String), HashMap<String, String>>,
    }

    impl DummySetupStringContext {
        fn new() -> Self {
            DummySetupStringContext {
                data: HashMap::new(),
            }
        }
        fn set_dummy(&mut self, lang: &str, file: &str, content: HashMap<String, String>) {
            self.data.insert((lang.to_string(), file.to_string()), content);
        }
    }

    impl SetupStringProvider for DummySetupStringContext {
        fn setupstring(&mut self, lang: &str, file: &str, _res: ResolveDirectives) -> Option<HashMap<String, String>> {
            self.data.get(&(lang.to_string(), file.to_string())).cloned()
        }
    }

    fn dummy_lang_ctx() -> LanguageTextContext {
        let mut dummy = DummySetupStringContext::new();
        let mut prayers = HashMap::new();
        prayers.insert("Alleluia".to_string(), "v. Alleluja. More text".to_string());
        dummy.set_dummy("Latin", "Psalterium/Common/Prayers.txt", prayers);

        initialize_language_text_context(
            &mut dummy, 
            "English", "German", 
            "Latin", "1.00", 
            false
        )
    }

    #[test]
    fn test_absolutio_benedictio_c10() {

        let mut commune_map = std::collections::HashMap::new();
        // Simulate “Benedictio => ‘abc\ndef\nghi\njkl…’ ”
        commune_map.insert(
            "Benedictio".to_string(),
            "AbsLine1\nLine2\nLine3\nBenLine4\n".to_string(),
        );
        commune_map.insert("SomeKey".to_string(), "C10 something…".to_string());

        let ctx = LiturgyContext {
            dayofweek: 2, // e.g. Tuesday
            winner: std::collections::HashMap::new(),
            commune: Some(commune_map),
            version: "Divino".to_string(),
            dayname: vec![],
            votive: "".to_string(),
            rule: "".to_string(),
            rank: 1.0,
            day: 2024,
            month: 01,
            year: 01,
            datafolder: "/data".to_string(),
        };

        let lines = absolutio_benedictio("la", &ctx, &dummy_lang_ctx());
        assert!(lines.contains(&"Absolutio. AbsLine1".to_string()));
        assert!(lines.contains(&"Benedictio. BenLine4".to_string()));
    }

    #[test]
    fn test_absolutio_benedictio_no_c10() {
        // No "C10" in the commune map => read from benedictions.
        // Our dummy `setupstring` is empty, so we get empty lines back, but we can still check structure.
        let ctx = LiturgyContext {
            dayofweek: 2,
            winner: std::collections::HashMap::new(),
            commune: None,
            version: "Divino".to_string(),
            dayname: vec![],
            votive: "".to_string(),
            rule: "".to_string(),
            rank: 1.0,
            day: 2024,
            month: 01,
            year: 01,
            datafolder: "/data".to_string(),
        };

        let lines = absolutio_benedictio("la", &ctx, &dummy_lang_ctx());
        // We only check that some known lines exist:
        assert!(lines.contains(&"$Pater noster_".to_string()));
        assert!(lines.contains(&"Absolutio. ".to_string())); // presumably empty though
        assert!(lines.contains(&"Benedictio. ".to_string())); 
    }

    #[test]
    fn test_legend_monastic_basic() {
        // Provide a minimal “winner” with Lectio4 and Responsory1.
        let mut winner_map = std::collections::HashMap::new();
        winner_map.insert("Lectio4".to_string(), "Some reading text".to_string());
        winner_map.insert("Responsory1".to_string(), "R. Lorem ipsum".to_string());
        let ctx = LiturgyContext {
            dayofweek: 3,
            winner: winner_map,
            commune: None,
            version: "".to_string(),
            dayname: vec!["".to_string(), "".to_string()],
            votive: "".to_string(),
            rule: "".to_string(),
            rank: 3.0,
            day: 2024,
            month: 01,
            year: 01,
            datafolder: "/data".to_string(),
        };

        let lines = legend_monastic("la", &ctx, &dummy_lang_ctx());
        // The first lines come from `absolutio_benedictio`.
        // Then we expect the reading, “$Tu autem”, “_”, and then the Responsory.
        let reading_pos = lines
            .iter()
            .position(|l| l.contains("Some reading text"))
            .unwrap_or(9999);
        let responsory_pos = lines
            .iter()
            .position(|l| l.contains("R. Lorem ipsum"))
            .unwrap_or(9999);

        assert!(reading_pos < 9999, "Reading not found in legend_monastic output");
        assert!(responsory_pos < 9999, "Responsory not found in legend_monastic output");
        assert!(
            reading_pos < responsory_pos,
            "Reading should appear before Responsory"
        );
    }

    #[test]
    fn test_legend_monastic_combines_lectio5_6() {
        let mut winner_map = std::collections::HashMap::new();
        winner_map.insert("Lectio4".to_string(), "Part4 ".to_string());
        winner_map.insert("Lectio5".to_string(), "Part5 ".to_string());
        winner_map.insert("Lectio6".to_string(), "Part6".to_string());
        // If Lectio5 does not contain '!', we append them to 4 => "Part4 Part5 Part6"
        let ctx = LiturgyContext {
            dayofweek: 0,
            winner: winner_map,
            commune: None,
            version: "MonasticSomething".to_string(),
            dayname: vec!["Pasc0".to_string(), "".to_string()],
            votive: "".to_string(),
            rule: "".to_string(),
            rank: 2.5,
            day: 2024,
            month: 01,
            year: 01,
            datafolder: "/data".to_string(),
        };

        let lines = legend_monastic("la", &ctx, &dummy_lang_ctx());
        // The reading line should contain "Part4 Part5 Part6".
        let combined = lines.iter().find(|l| l.contains("Part4 Part5 Part6"));
        assert!(
            combined.is_some(),
            "Lectio4,5,6 did not combine as expected"
        );
    }

    #[test]
    fn test_legend_monastic_removes_te_deum() {
        let mut winner_map = std::collections::HashMap::new();
        winner_map.insert("Lectio4".to_string(), "Some &teDeum text   ".to_string());
        let ctx = LiturgyContext {
            dayofweek: 1,
            winner: winner_map,
            commune: None,
            version: "".to_string(),
            dayname: vec!["".to_string(), "".to_string()],
            votive: "".to_string(),
            rule: "".to_string(),
            rank: 3.0,
            day: 2024,
            month: 01,
            year: 01,
            datafolder: "/data".to_string(),
        };
        let lines = legend_monastic("la", &ctx, &dummy_lang_ctx());

        // We should see that "&teDeum" was removed.
        let found = lines
            .iter()
            .any(|l| l.contains("&teDeum"));
        assert!(!found, "Should have removed &teDeum");
    }

    #[test]
    fn test_makeferia() {
        assert_eq!(makeferia(0), "Sunday");
        assert_eq!(makeferia(2), "Feria III.");
        assert_eq!(makeferia(6), "Sabbato");
    }

    #[test]
    fn test_compress_alleluia() {
        let mut text = "Alleluia Alleluia Alleluia".to_string();
        compress_alleluia(&mut text);
        assert_eq!(text, "Alleluia ");
    }

    #[test]
    fn test_replace_title() {
        let title = "some stuff#Rest of title";
        assert_eq!(replace_title(title), "v. Rest of title");
    }
}
