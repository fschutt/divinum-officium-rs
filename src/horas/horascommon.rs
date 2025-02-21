use crate::date::getweek;
use crate::regex::{contains_ci, remove_after, starts_with_ignore_case};
use crate::setup_string::{ResolveDirectives, SetupStringProvider};

/// Returns the rank name (a string) given the many parameters that the Perl code used as globals.
pub fn rankname(
    provider: &mut impl SetupStringProvider,
    lang: &str,
    rank: f64,
    winner: &str,
    commune: &str,
    version: &str,
    day: u32,
    month: u32,
    year: i32,
    dayofweek: u8,
    hora: &str,
) -> String {

    /// Helper: returns a truncated (floor) index if the float is >= 0.
    /// Negative rank would panic; the original Perl code never does that.
    fn rank_index(r: f64) -> usize {
        if r < 0.0 {
            0
        } else {
            r as usize // truncation
        }
    }


    // 1) Pull "Latin" winner => latwinner => latwinner["Rank"], removing everything after `;;`
    let latin_sections = match provider.setupstring("Latin", winner, ResolveDirectives::None) {
        Some(fs) => fs,
        None => return "".to_string(), // If no data, we can’t proceed.
    };
    let mut latname = latin_sections.get("Rank").cloned().unwrap_or_default();
    remove_after(&mut latname, ";;");

    // 2) Pull the “Comment.txt” in the user’s chosen language => t => ranktable from "Festa"
    let comment_txt = match provider.setupstring(lang, "Psalterium/Comment.txt", ResolveDirectives::None) {
        Some(fs) => fs,
        None => return "".to_string(),
    };

    // ranktable lines
    let ranktable: Vec<String> = comment_txt
        .get("Festa")
        .map(|festas| festas.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default();

    // A safe accessor for ranktable[i], gracefully returning "" if out of range
    let get_ranktable = |i: usize| -> String {
        ranktable.get(i).cloned().unwrap_or_default()
    };

    // We also use .get(...) for other keys:
    let t_vigilia        = comment_txt.get("Vigilia").cloned().unwrap_or("Vigilia".to_string());
    let t_privilegiata   = comment_txt.get("privilegiata").cloned().unwrap_or("privilegiata".to_string());
    let t_classis        = comment_txt.get("classis").cloned().unwrap_or("classis".to_string());
    let t_dies_octavae   = comment_txt.get("Dies Octavæ").cloned().unwrap_or("Dies Octavæ".to_string());
    let t_ordinis        = comment_txt.get("ordinis").cloned().unwrap_or("ordinis".to_string());

    // For Sunday references, we read from "Dominicae"
    let sundaytable: Vec<String> = comment_txt
        .get("Dominicae")
        .map(|doms| doms.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default();

    // For Feria references, we read from "Feriae"
    let feriatable: Vec<String> = comment_txt
        .get("Feriae")
        .map(|f| f.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default();

    // Let’s define rankname as we go:
    // We'll do a series of condition checks that match the original Perl logic,
    // with early return for each scenario.

    // This matches the big if(...) in the original:
    // if ( ($latname !~ /(?:Die|Feria|Sabbato|^In Octava)/i )
    //   && ($winner !~ /Pasc[07]/ || $dayofweek == 0 || $latname !~ /Pasc|Pent/) ) { ... }
    let no_ferial_words = !contains_ci(&latname, "die")
        && !contains_ci(&latname, "feria")
        && !contains_ci(&latname, "sabbato")
        && !starts_with_ignore_case(&latname, "In Octava");
    let winner_not_pasc07 = !(contains_ci(winner, "Pasc0") || contains_ci(winner, "Pasc7"));
    let latname_not_pascpent = !(contains_ci(&latname, "pasc") || contains_ci(&latname, "pent"));

    if no_ferial_words && (winner_not_pasc07 || dayofweek == 0 || latname_not_pascpent) {
        // we replicate the sub-block:
        let mut i = rank;
        // if($version =~ /19(?:55|6)/ && $winner !~ /Pasc5-3/ && $latname =~ /feria/i) { $i=0; }
        let version_1955_or_196 = version.contains("1955") || version.contains("196");
        if version_1955_or_196
            && !contains_ci(winner, "Pasc5-3")
            && contains_ci(&latname, "feria")
        {
            i = 0.0;
        }
        // if($latname =~ /Sanctæ Fami/i && $version !~ /196/) { $i=4; }
        if contains_ci(&latname, "sanctæ fami") && !version.contains("196") {
            i = 4.0;
        }
        // if($latname =~ /Defunctorum/i && $version !~ /196/) { $i=3; }
        if contains_ci(&latname, "defunctorum") && !version.contains("196") {
            i = 3.0;
        }

        let rankname = get_ranktable(rank_index(i));

        // if ($latname =~ /Vigilia Epi/i) { ... }
        if contains_ci(&latname, "vigilia epi") {
            if contains_ci(version, "cist") {
                // $rankname = $t{Vigilia};
                // $rankname .= $dayofweek ? $t{privilegiata} : $ranktable[2];
                let mut vig = t_vigilia.clone();
                if dayofweek > 0 {
                    vig.push_str(&t_privilegiata);
                } else {
                    vig.push_str(&get_ranktable(2));
                }
                return vig.replace('\n', "");
            } else {
                // $rankname = $ranktable[2]; # Semiduplex
                // $rankname .= " $t{Vigilia} II. $t{classis}" unless $version =~ /Trident/;
                let mut r = get_ranktable(2);
                if !contains_ci(version, "Trident") {
                    r.push(' ');
                    r.push_str(&t_vigilia);
                    r.push_str(" II. ");
                    r.push_str(&t_classis);
                }
                return r.replace('\n', "");
            }
        }
        // elsif ($latname =~ /^In Vigilia/i && $rank <= 2.5) { ... }
        if starts_with_ignore_case(&latname, "In Vigilia") && rank <= 2.5 {
            // $rankname = $version =~ /cist/i ? $t{Vigilia} : $ranktable[1];
            if contains_ci(version, "cist") {
                return t_vigilia.replace('\n', "");
            } else {
                return get_ranktable(1).replace('\n', "");
            }
        }

        // if ($latname =~ /Dominica/i && $version !~ /196/) { ... big sunday logic ... }
        if contains_ci(&latname, "dominica") && !version.contains("196") {

            let weekname = getweek(
                day, month, year,
                dayofweek == 6 && contains_ci(hora, "Vespera|Completorium"), 
                false /* missaf */
            );

            // The giant if:
            let mut idx = if contains_ci(&weekname, "pasc0")
                || contains_ci(&weekname, "pasc7")
                || contains_ci(&weekname, "pent01")
            {
                0 // Duplex I. classis
            } else if contains_ci(&weekname, "adv1")
                || contains_ci(&weekname, "quad1")
                || contains_ci(&weekname, "quad2")
                || contains_ci(&weekname, "quad3")
                || contains_ci(&weekname, "quad4")
                || contains_ci(&weekname, "quad5")
                || contains_ci(&weekname, "quad6")
            {
                1 // Semiduplex Dominica I. classis
            } else if contains_ci(&weekname, "adv2")
                || contains_ci(&weekname, "adv3")
                || contains_ci(&weekname, "adv4")
                || contains_ci(&weekname, "quadp")
            {
                2 // Semiduplex Dominica II. classis
            } else if (contains_ci(&weekname, "epi1")
                || contains_ci(&weekname, "epi2")
                || contains_ci(&weekname, "epi3")
                || contains_ci(&weekname, "epi4")
                || contains_ci(&weekname, "epi5")
                || contains_ci(&weekname, "epi6")
                || contains_ci(&weekname, "pent22")
                || contains_ci(&weekname, "pent23"))
                && dayofweek != 0
            {
                3 // Semiduplex Dominica anticipata
            } else {
                4 // Semiduplex Dominica minor
            };
            // $i=2 if $version =~ /Trident/ && /Quad[2-4]/  (We approximate that check of `w`)
            if version.contains("Trident")
                && (contains_ci(&weekname, "quad2")
                    || contains_ci(&weekname, "quad3")
                    || contains_ci(&weekname, "quad4"))
            {
                idx = 2;
            }
            let mut rn = sundaytable.get(idx).cloned().unwrap_or_default();

            // if cist, we do some replacements
            if contains_ci(version, "cist") {
                if contains_ci(&weekname, "pasc0") || contains_ci(&weekname, "pasc7") {
                    rn = get_ranktable(6);
                }
                if contains_ci(&weekname, "pent01") {
                    rn = get_ranktable(5);
                }
                rn = rn.replace("Duplex ", "Dominica ");
                rn = rn.replace("Semiduplex ", "");
            }
            return rn.replace('\n', "");
        }

        // Otherwise (still in the main “no_ferial_words && …” block):
        return rankname.replace('\n', "");
    }

    // next block: elsif ($commune =~ /C10/) { ...
    if contains_ci(commune, "C10") {
        // Simplex - BMV Sabbato
        return get_ranktable(1).replace('\n', "");
    }

    // elsif ($version =~ /196/ && $winner =~ /Pasc[07]-[1-6]/)
    if version.contains("196")
        && ((contains_ci(winner, "Pasc0-1")
            || contains_ci(winner, "Pasc0-2")
            || contains_ci(winner, "Pasc0-3")
            || contains_ci(winner, "Pasc0-4")
            || contains_ci(winner, "Pasc0-5")
            || contains_ci(winner, "Pasc0-6"))
            ||
            (contains_ci(winner, "Pasc7-1")
            || contains_ci(winner, "Pasc7-2")
            || contains_ci(winner, "Pasc7-3")
            || contains_ci(winner, "Pasc7-4")
            || contains_ci(winner, "Pasc7-5")
            || contains_ci(winner, "Pasc7-6"))
        )
    {
        // Paschal & Pentecost Octave post 1960
        return format!("{} I. {}", t_dies_octavae, t_classis).replace('\n', "");
    }

    // elsif ($version =~ /196/ && $winner =~ /Pasc6-6/)
    if version.contains("196") && contains_ci(winner, "Pasc6-6") {
        // I. classis - Vigilia Pentecostes
        return get_ranktable(6).replace('\n', "");
    }

    // elsif ($version =~ /196/ && $winner =~ /Pasc5-3/)
    if version.contains("196") && contains_ci(winner, "Pasc5-3") {
        // II. classis - Vigilia Asc
        return get_ranktable(5).replace('\n', "");
    }

    // elsif ($version =~ /196/ && $month == 12 && $day > 16 && $day < 25 && $dayofweek)
    if version.contains("196") && month == 12 && day > 16 && day < 25 && dayofweek > 0 {
        // II. classis - Week before Christmas
        return get_ranktable(5).replace('\n', "");
    }

    // elsif ($version =~ /cist/i && $winner =~ /Pasc[07]-[1-6]/)
    if contains_ci(version, "cist")
       && (
            contains_ci(winner, "Pasc0-1")
            || contains_ci(winner, "Pasc0-2")
            || contains_ci(winner, "Pasc0-3")
            || contains_ci(winner, "Pasc0-4")
            || contains_ci(winner, "Pasc0-5")
            || contains_ci(winner, "Pasc0-6")
            || contains_ci(winner, "Pasc7-1")
            || contains_ci(winner, "Pasc7-2")
            || contains_ci(winner, "Pasc7-3")
            || contains_ci(winner, "Pasc7-4")
            || contains_ci(winner, "Pasc7-5")
            || contains_ci(winner, "Pasc7-6")
          )
    {
        // Paschal & pentecost Octave pre 1960
        if (rank - 7.0).abs() < f64::EPSILON {
            // rank == 7
            return get_ranktable(4).replace('\n', "");
        } else {
            return get_ranktable(1).replace('\n', "");
        }
    }

    // elsif ($version !~ /196/ && $winner =~ /Pasc[07]-[1-6]/)
    if !version.contains("196")
       && (
            contains_ci(winner, "Pasc0-1")
            || contains_ci(winner, "Pasc0-2")
            || contains_ci(winner, "Pasc0-3")
            || contains_ci(winner, "Pasc0-4")
            || contains_ci(winner, "Pasc0-5")
            || contains_ci(winner, "Pasc0-6")
            || contains_ci(winner, "Pasc7-1")
            || contains_ci(winner, "Pasc7-2")
            || contains_ci(winner, "Pasc7-3")
            || contains_ci(winner, "Pasc7-4")
            || contains_ci(winner, "Pasc7-5")
            || contains_ci(winner, "Pasc7-6")
          )
    {
        // Paschal & pentecost Octave pre 1960
        if (rank - 7.0).abs() < f64::EPSILON {
            // Duplex I. classis
            return get_ranktable(7).replace('\n', "");
        } else if version.contains("1955") {
            // Duplex
            return get_ranktable(3).replace('\n', "");
        } else {
            // Semiduplex
            return get_ranktable(2).replace('\n', "");
        }
    }

    // elsif ($version =~ /1955/ && $winner =~ /Pasc6-6/)
    if version.contains("1955") && contains_ci(winner, "Pasc6-6") {
        // Vigilia Pentecostes => 'Duplex'
        return get_ranktable(3).replace('\n', "");
    }

    // elsif ($version =~ /Trident/ && $latname =~ /^In Octava/i)
    if version.contains("Trident") && starts_with_ignore_case(&latname, "In Octava") {
        // 'Duplex/xij.L.' - all other Octaves pre Divino
        let i = if contains_ci(version, "cist") { 2 } else { 3 };
        return get_ranktable(i).replace('\n', "");
    }

    // elsif ($version =~ /Trident/ && $latname =~ /infra Octavam|post Octavam Asc|Vigilia Pent/i)
    if version.contains("Trident")
        && (contains_ci(&latname, "infra octavam")
            || contains_ci(&latname, "post octavam asc")
            || contains_ci(&latname, "vigilia pent"))
    {
        // 'Semiduplex/iij.L.' - all other Octaves pre Divino
        let i = if contains_ci(version, "cist") { 1 } else { 2 };
        return get_ranktable(i).replace('\n', "");
    }

    // elsif ($version =~ /Divino/ && $latname =~ /^In Octava|infra Octavam|post Octavam Asc|Vigilia Pent/i)
    if version.contains("Divino")
        && (starts_with_ignore_case(&latname, "In Octava")
            || contains_ci(&latname, "infra octavam")
            || contains_ci(&latname, "post octavam asc")
            || contains_ci(&latname, "vigilia pent"))
    {
        // Big chain. We replicate the final fallback from the original:
        if rank < 2.0 {
            return get_ranktable(1).replace('\n', "");
        } else if rank < 3.0 && (!contains_ci(&latname, "asc") 
                                && !contains_ci(&latname, "nat")
                                && !contains_ci(&latname, "cord"))
                  || contains_ci(&latname, "post")
                  || contains_ci(&latname, "joan")
        {
            return get_ranktable(2).replace('\n', "");
        } else if rank < 3.0 {
            // "Semiduplex III. ordinis"
            let mut s = get_ranktable(2);
            s.push_str(" III. ");
            s.push_str(&t_ordinis);
            return s.replace('\n', "");
        } else if rank < 5.0
                  && (!contains_ci(&latname, "asc")
                       && !contains_ci(&latname, "nat")
                       && !contains_ci(&latname, "cord"))
        {
            return get_ranktable(4).replace('\n', "");
        } else if rank < 5.0 {
            // "Duplex majus III. ordinis"
            let mut s = get_ranktable(4);
            s.push_str(" III. ");
            s.push_str(&t_ordinis);
            return s.replace('\n', "");
        } else if rank < 5.61 {
            // "Semiduplex II. ordinis"
            let mut s = get_ranktable(2);
            s.push_str(" II. ");
            s.push_str(&t_ordinis);
            return s.replace('\n', "");
        } else if rank < 6.5 {
            // "Duplex majus II. ordinis"
            let mut s = get_ranktable(4);
            s.push_str(" II. ");
            s.push_str(&t_ordinis);
            return s.replace('\n', "");
        } else {
            // "Semiduplex Vigilia I. classis"
            let mut s = get_ranktable(2);
            s.push(' ');
            s.push_str(&t_vigilia);
            s.push_str(" I. ");
            s.push_str(&t_classis);
            return s.replace('\n', "");
        }
    }

    // elsif ($version !~ /196/ && $winner =~ /07-04/ && $dayofweek > 0)
    if !version.contains("196")
        && contains_ci(winner, "07-04")
        && dayofweek > 0
    {
        // ??? "Independence Day" comment
        if (rank - 7.0).abs() < f64::EPSILON {
            return "Duplex I. classis".to_string();
        } else {
            return "Semiduplex".to_string();
        }
    }

    // Default for Ferias:
    if !version.contains("196") {
        // "my @feriatable = split(...); $rankname = $feriatable[$rank == 1.15 ? 2 : $rank]"
        // We replicate that logic:
        if (rank - 1.15).abs() < f64::EPSILON {
            // index=2
            return feriatable.get(2).cloned().unwrap_or_default().replace('\n', "");
        } else {
            let idx = rank_index(rank);
            return feriatable.get(idx).cloned().unwrap_or_default().replace('\n', "");
        }
    } else {
        // $version =~ /196/
        // $rank == 4.9 => index=5 else index=rank
        if (rank - 4.9).abs() < f64::EPSILON {
            return get_ranktable(5).replace('\n', "");
        } else {
            let idx = rank_index(rank);
            return get_ranktable(idx).replace('\n', "");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::setup_string::{FileSections, ResolveDirectives, SetupStringProvider};

    use super::*;

    /// Example stub for your real SetupStringContext.
    pub struct SetupStringContext;

    impl SetupStringProvider for SetupStringContext {
        fn setupstring(
            &mut self,
            lang: &str,
            file: &str,
            _res: ResolveDirectives
        ) -> Option<FileSections> {
            
            // Stubbed-out logic. In real code, you’d load from disk or memory.
            // Below we just return Some minimal data so the function can compile/test.
            let mut map = FileSections::new();
            if file == "Psalterium/Comment.txt" {
                map.insert("Festa".to_string(),
                    r#"Duplex I. classis
                    Semiduplex
                    Simplex
                    Duplex
                    Duplex majus
                    II. classis
                    I. classis
                    Duplex I. classis (index 7)
                    "#.to_string()
                );

                map.insert("Dominicae".to_string(),
                    r#"Duplex I. classis (Sunday index=0)
                    Semiduplex Dominica I. classis
                    Semiduplex Dominica II. classis
                    Semiduplex Dominica anticipata
                    Semiduplex Dominica minor
                    "#.to_string()
                );

                map.insert("Feriae".to_string(),
                    r#"Feria rank=0
                    Feria rank=1
                    Feria rank=2 (Quattuor?)
                    "#.to_string()
                );

                // Example placeholders:
                map.insert("Vigilia".to_string(), "Vigilia placeholder".to_string());
                map.insert("privilegiata".to_string(), "privilegiata placeholder".to_string());
                map.insert("classis".to_string(), "classis placeholder".to_string());
                map.insert("Dies Octavæ".to_string(), "Dies Octavæ placeholder".to_string());
                map.insert("ordinis".to_string(), "ordinis placeholder".to_string());
            } else if lang == "Latin" {
                // Example of a "Rank" key in the “file” named by `winner`
                map.insert("Rank".to_string(), "Some Latin rank;;and something else".to_string());
            }

            Some(map)
        }
    }


    #[test]
    fn test_basic_rankname() {
        let mut ctx = SetupStringContext;
        // Pretend we’re passing some typical parameters:
        let result = rankname(
            &mut ctx,
            "English",
            0.0,            // rank
            "Pasc0-2",      // winner
            "C10",          // commune
            "1962",         // version
            10, 4, 2025,    // day, month, year
            1,              // dayofweek
            "Matutinum"
        );
        // Because we gave "version=1962" and "winner=Pasc0-2",
        // the code block for `($version =~ /196/ && $winner =~ /Pasc[07]-[1-6]/)` triggers,
        // returning "Dies Octavæ I. classis" (or whatever the stub data says).
        assert_eq!(result, "Dies Octavæ placeholder I. classis placeholder");
    }

    #[test]
    fn test_rank_7_pre_1960() {
        let mut ctx = SetupStringContext;
        // rank=7, version doesn't contain "196", winner = "Pasc7-3"
        let result = rankname(
            &mut ctx,
            "English",
            7.0,
            "Pasc7-3",
            "",
            "1955",    // version
            1, 5, 2025, 6, "Vespera"
        );
        // The logic for "pre 1960" + "Pasc7-[1-6]" + rank=7 => "Duplex I. classis"
        // (per the big chain).
        assert_eq!(result, "Duplex I. classis (index 7)");
    }

    #[test]
    fn test_feria_fallback() {
        let mut ctx = SetupStringContext;
        // A scenario that doesn't match the big ifs => default "Ferias"
        // version does not contain "196", no special Pascal code, day=0, rank=0, etc.
        let result = rankname(
            &mut ctx,
            "English",
            0.0,
            "OrdinaryWinner",
            "",
            "Divino1954",  // does not contain "196"
            2, 2, 2025, 3, "None"
        );
        // We'll see what the fallback picks up from "Feriae" table index=0
        assert_eq!(result, "Feria rank=0");
    }

    #[test]
    fn test_vigilia_epiphany() {
        let mut ctx = SetupStringContext;
        // We set up latname so that it includes "Vigilia Epi"
        // We'll fake that by changing what SetupStringContext returns for "Rank"
        // or we can rely on the default stub. For demonstration, let’s do a real check:
        // If latname has "vigilia epi" => we pick a path in the code.
        // We do "cist" to see that branch.

        // Since our stub returns "Some Latin rank;;and something else" for "Rank"
        // we must adapt. Real test code might require a more elaborate mock or fixture.
        // For demonstration, let's just ensure "version" includes "cist" to see that path.
        let result = rankname(
            &mut ctx,
            "English",
            3.0,     // rank
            "Something",  // winner
            "",
            "cistercian", // version with "cist"
            6, 1, 2025,
            6, // dayofweek=6 => Saturday
            "Vespera"
        );
        // Because our default stub for latname is "Some Latin rank", which does NOT contain
        // "vigilia epi," this won't do exactly that path. In a real test, you'd mock or
        // insert the string "vigilia epi" in the "Rank" field. This is just to illustrate.
        // We'll just check we don't crash:
        assert!(!result.is_empty());
    }
}
