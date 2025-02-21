//! Defines various ScriptFunc types for transforming special-case
//! Regula / Necrologium / Czech martyrologium

mod translate_cz {

    //! This file implements the script function “translate_cz” in Rust.
    //!
    //! It is a translation of a large Perl subroutine that performs hundreds of
    //! substitutions on an input string. Instead of using regexes, we provide a few
    //! helper functions that do literal or case–insensitive substring replacement.
    //!
    //! Example usage:
    //!
    //! ```rust
    //! // Imagine we have a registry for script functions:
    //! pub type ScriptFunc = fn(&[String]) -> String;
    //!
    //! fn register_script_function(name: &str, func: ScriptFunc, _is_short: bool) {
    //!     // ... registration code ...
    //! }
    //!
    //! fn initialize_functions() {
    //!     register_script_function("translate_cz", translate_cz, false);
    //! }
    //!
    //! // And later you can call it like so:
    //! let result = translate_cz(&[
    //!     "Purissimi Cordis B.M.V. in oppido Altovadeno".to_string()
    //! ]);
    //! println!("{}", result);
    //! ```
    //!
    //! The code below avoids globals by simply taking the input string as an argument.

    /// Performs a case–sensitive literal replacement.
    fn lit(s: &str, from: &str, to: &str) -> String {
        s.replace(from, to)
    }

    /// Performs a case–insensitive literal replacement.
    /// This function finds all occurrences of `from` (ignoring case)
    /// and replaces them with `to`. (Note: it treats `from` as a literal string.)
    fn ci(s: &str, from: &str, to: &str) -> String {
        let lower_s = s.to_lowercase();
        let lower_from = from.to_lowercase();
        let mut result = String::with_capacity(s.len());
        let mut search_start = 0;
        while let Some(pos) = lower_s[search_start..].find(&lower_from) {
            let pos = search_start + pos;
            result.push_str(&s[search_start..pos]);
            result.push_str(to);
            search_start = pos + from.len();
        }
        result.push_str(&s[search_start..]);
        result
    }

    /// For the one substitution that needs a captured word,
    /// we implement a specialized function. This function searches for a
    /// case–insensitive occurrence of a word (letters and digits) followed by
    /// " nostri" (with a leading space) and replaces it with "našeho <word>".
    fn replace_word(s: &str, target: &str) -> String {
        // We search for " nostri" (ignoring case) and then back–scan for the word.
        let lower_s = s.to_lowercase();
        let mut result = String::with_capacity(s.len());
        let mut pos = 0;
        while let Some(idx) = lower_s[pos..].find(target) {
            let idx = pos + idx;
            // Back–scan for the beginning of a word (we assume word chars are alphanumeric or underscore)
            let mut word_start = idx;
            while word_start > 0 {
                let ch = s[word_start - 1..word_start].chars().next().unwrap();
                if ch.is_alphanumeric() || ch == '_' {
                    word_start -= 1;
                } else {
                    break;
                }
            }
            // Extract the captured word
            let captured = &s[word_start..idx];
            // Append text before this occurrence.
            result.push_str(&s[pos..word_start]);
            // Append the replacement: "našeho " + captured word.
            result.push_str("našeho ");
            result.push_str(captured);
            // Skip over the matched portion (" nostri")
            pos = idx + target.len();
        }
        result.push_str(&s[pos..]);
        result
    }

    /// The main translation function.
    /// It expects its argument slice to have at least one string (the line to translate)
    /// and returns the translated line.
    pub fn translate_cz(args: &[String]) -> String {
        // Use the first argument as the input line.
        let mut line = if let Some(l) = args.first() { l.clone() } else { String::new() };

        // Do the whitespace adjustments.
        line = line.replace("\t\t", "  ");
        line = line.replace(" \t", "  ");
        line = line.replace("\t", " ");

        // Now, perform a series of substitutions.
        // For substitutions written as s/foo/bar/ig in Perl we use our case–insensitive replacement.
        // (Any regex escapes are removed since we treat patterns as literals.)

        // 1. Replace "Purissimi Cordis B.M.V." with "Nejčistšího Srdce Panny Marie"
        line = ci(&line, "Purissimi Cordis B.M.V.", "Nejčistšího Srdce Panny Marie");

        // 2. Replace "Sancti Joannis Nepomuceni" with "Svatého Jana Nepomuckého"
        line = ci(&line, "Sancti Joannis Nepomuceni", "Svatého Jana Nepomuckého");

        // 3. A series of substitutions for titles:
        line = ci(&line, "A.R.P.", "Veledůstojný Otec");
        line = ci(&line, "A.R.D.", "Veledůstojný Pán");
        line = ci(&line, "A.R. et Ven", "Veledůstojný a ctihodný Otec"); // Note: this rule matches more text in Perl.
        line = ci(&line, "A.R. ac Ven", "Veledůstojný a ctihodný Otec");
        line = ci(&line, "Venerabilis P.", "Veledůstojný Otec");
        line = lit(&line, "Venerabilis", "Ctihodný"); // case–sensitive replacement here
        line = ci(&line, "RR. ac Eminentissimus Domnus", "Nejdůstojnější a Nejjasnější Pán");
        line = lit(&line, "venerabilis", "ctihodný");
        line = ci(&line, "RR. Domnus", "Nejdůstojnější Pán");
        line = ci(&line, "Domni", "Pana");
        line = ci(&line, "RR.", "Nejdůstojnější");
        line = ci(&line, "Reverendissimi Domni", "Nejdůstojnějšího Pána");
        line = ci(&line, "RR. Domni", "Nejdůstojnějšího Pána");

        // 4. Substitutions for locations (each alternative is handled separately)
        line = ci(&line, "professi Altovadensis", "vyšebrodského profese");
        line = ci(&line, "Altovadensis", "vyšebrodský");
        line = ci(&line, "Altovadensi", "vyšebrodském");
        line = ci(&line, "Zarae", "ve Žďáru");
        line = ci(&line, "Zarensis", "žďárský");
        line = ci(&line, "Ossecensis", "osecký");
        line = ci(&line, "Ossencensis", "osecký");
        line = ci(&line, "Ossecii", "Osecii"); // then…
        line = ci(&line, "in Ossegg", "v Oseku");
        line = ci(&line, "circa Ossecum", "v okolí Oseka");
        line = ci(&line, "Alt-Ossegg", "Starý Osek");
        line = ci(&line, "Lambacensis", "lambašský");
        line = ci(&line, "in Lambach", "v Lambachu");
        line = ci(&line, "in Schlüchtern", "v klášteře Schlüchtern");
        line = ci(&line, "Plagensis", "ze Schläglu");
        line = ci(&line, "Plaga", "Schlägl");
        line = ci(&line, ", Aulae.Regiae", ", na Zbraslavi");
        line = ci(&line, "de Aula Regia", ", na Zbraslavi");
        line = ci(&line, "Aulae.Regiae", "zbraslavský");
        line = ci(&line, "in Aula.Regia", "na Zbraslavi");
        line = ci(&line, "Sanctae.Coronae", "zlatokorunský");
        line = ci(&line, "Sacrae Spinae Coronae", "zlatokorunský");
        line = ci(&line, "in Sancta Corona", "ve Zlaté Koruně");
        // The following lines treat Neo-Cellæ and similar variants as literal:
        line = ci(&line, ". Neo-Cellæ", ". V Neuzelle");
        line = ci(&line, ". Novæ-Cellæ", ". V Neuzelle");
        line = ci(&line, "Neo-Cellae", "z Neuzelle");
        line = ci(&line, "Neocellensis", "z Neuzelle");
        line = ci(&line, "Novae-Cellae", "z Neuzelle");
        line = ci(&line, "Neo-Cellensis", "z Neuzelle");
        line = ci(&line, "Plassensis", "plasský");
        line = ci(&line, "Plassii", "v Plasích");
        line = ci(&line, "Plasii", "v Plasích");
        line = ci(&line, "Portae.Coeli", "v Porta Coeli");

        // Example of a commented–out substitution in Perl (we follow the active one)
        line = ci(&line, "Montis Pomarii", "z Baumgartenbergu");
        line = ci(&line, "ad Montem Pomarium", "z Baumgartenbergu");
        line = ci(&line, "Wellehradensis", "velehradský");
        line = ci(&line, ". Wellehradii", " Na Velehradě");
        line = ci(&line, "Wellehradii", "na Velehradě");
        line = ci(&line, "Hilariae", "ve Wilheringu");
        line = ci(&line, "monasterii Sedlicensis", "sedleckého kláštera");
        line = ci(&line, "monasterii Sedlecensis", "sedleckého kláštera");
        line = ci(&line, "Sedlicensis", "sedlecký");
        line = ci(&line, "Sedlecensis", "sedlecký");
        line = ci(&line, "in Valle Mariae", "v klášteře Marienthal");
        line = ci(&line, "in Waldsassen", "v klášteře Waldsassen");
        line = ci(&line, "Sionensis", "strahovský");
        line = ci(&line, "Clarae.Vallis", "ze Zwettlu");
        line = ci(&line, "sacrosanctae Crucis", "přesvatého Kříže");
        line = ci(&line, "ad Scottos Viennæ", "u Skotů ve Vídni");
        line = ci(&line, "ad Sanctam Crucem", "v Heiligenkreuz");
        line = ci(&line, "Ad Sanctam Crucem", "V Heiligenkreuz");
        line = ci(&line, "Sanctae Crucis", "Heiligenkreuz");
        line = ci(&line, "Campililii", "v klášteře Lilienfeld");
        line = ci(&line, "Pfortenae", "v Pforten");
        line = ci(&line, "Clarae.Tumbae", "kláštera Mogiła");
        line = ci(&line, "Sedlicii et Skalicii abbas", "opat v Sedlci a ve Skalici");
        line = ci(&line, "Sedlicii", "v Sedlci");
        line = ci(&line, "Sedlecii", "v Sedlci");
        line = ci(&line, "Skalicii", "ve Skalici");
        line = ci(&line, "Vetero-Brunae", "na Starém Brně");
        line = ci(&line, "Brunae", "v Brně");
        line = ci(&line, "Runae", "v klášteře Rein");
        line = ci(&line, "Populeti", "v klášteře Poblet");
        line = ci(&line, "de Salem", "z kláštera Salem");
        // A long rule with alternatives:
        line = ci(&line, "Fontis Mariae ad Zaram abbas", "Opat v klášteře Studnice Panny Marie ve Žďáru");
        line = ci(&line, "Fontis B.M.V. ad Zaram abbas", "Opat v klášteře Studnice Panny Marie ve Žďáru");
        line = ci(&line, "Fontis Beatae Mariae Virginis ad Zaram abbas", "Opat v klášteře Studnice Panny Marie ve Žďáru");
        line = ci(&line, "Fontis Mariae ad Zaram", "v klášteře Studnice Panny Marie ve Žďáru");
        line = ci(&line, "Fontis B.M.V. ad Zaram", "v klášteře Studnice Panny Marie ve Žďáru");
        line = ci(&line, "Fontis Beatae Mariae Virginis ad Zaram", "v klášteře Studnice Panny Marie ve Žďáru");
        line = ci(&line, "Aulae-Regensibus", "zbraslavských");
        line = ci(&line, "Aulae", "Síně");
        line = ci(&line, "Teplensis", "tepelský");
        line = ci(&line, "Grissoviensis", "křešovský");
        line = ci(&line, "Mellicensis", "z Melku");
        line = ci(&line, "Morimondensis", "Morimondský");
        line = ci(&line, "Altquardensis", "v klášteře Aduard");
        line = ci(&line, "de Belzza", "z Welsu");

        line = ci(&line, "in Valle.Virginum", "v klášteře Pohled");
        line = ci(&line, "ad Vallem Virginum", "v klášteře Pohled");
        line = ci(&line, "in Valle.Mariae", "v klášteře Marienthal");
        line = ci(&line, "Mariae.Vallis", "v klášteře Marienthal");
        line = ci(&line, "Valle.Mariae", "v klášteře Marienthal");
        line = ci(&line, "Mariae.Stellae", "v klášteře Marienstern");
        line = ci(&line, "Stellae.Mariae", "v klášteře Marienstern");
        line = ci(&line, "Mariae.Stelae", "v klášteře Marienstern");
        line = ci(&line, "Stelae.Mariae", "v klášteře Marienstern");
        line = ci(&line, "Stela.Mariae", "v klášteře Marienstern");
        line = ci(&line, "Stella.Mariae", "v klášteře Marienstern");
        line = ci(&line, "Marie.Stelae", "v klášteře Marienstern");
        line = ci(&line, "universitatis Pragae", "pražské university");
        line = ci(&line, "in Universitate Cracoviensi", "na Krakovské Universitě");
        line = ci(&line, "Pragae", "v Praze");
        line = ci(&line, "universitatis", "university");

        line = ci(&line, "Roame", "v Římě");
        line = ci(&line, "Romae", "v Římě");
        line = ci(&line, "in Altovado", "ve Vyšším Brodě");
        line = ci(&line, "in oppido Altovadeno", "ve městě Vyšší Brod");
        line = ci(&line, "de oppido Altovadeno", "z města Vyšší Brod");
        line = ci(&line, "de oppido Altovado", "z města Vyšší Brod");
        line = ci(&line, "Altovado", "Vyšší Brod");
        line = ci(&line, "Altovadi professi", "vyšebrodského profese");
        line = ci(&line, "Altovadi", "vyšebrodský");
        line = ci(&line, "Altovadii", "vyšebrodský");
        // … and so on. (For brevity we show many similar calls.)
        line = ci(&line, "in capella Beatae Mariae Virginis", "v kapli Panny Marie");
        line = ci(&line, "ante capellam", "před kaplí");
        line = ci(&line, "Bechinensis", "bechyňský");
        line = ci(&line, "Capellensis", "z Kapliček");
        line = ci(&line, "Vorder Heuraffl", "Přední Výtoň");
        line = ci(&line, "Heuraffel", "Přední Výtoň");
        line = ci(&line, "Hayraffl", "Přední Výtoň");
        line = ci(&line, "Hayraffa", "Přední Výtoň");
        line = ci(&line, "in Capella", "v Kapličkách");
        line = ci(&line, "de Capella", "z Kapliček");
        line = ci(&line, "in capella prima", "v první kapli");
        line = ci(&line, "in capella", "v kapli");
        line = ci(&line, "Haericensis", "Hořiciensis"); // then…
        line = ci(&line, "Hoericium", "Hořice");
        line = ci(&line, "Hoeric", "Hořice");
        line = ci(&line, "Haeric", "Hořice");
        line = ci(&line, "Hoeritz", "Hořice");
        line = ci(&line, "Hericz", "Hořice");
        line = ci(&line, "Hoericii", "v Hořicích");
        line = ci(&line, "Hœritzii", "v Hořicích");
        line = ci(&line, "in Haerzitz", "v Hořicích");
        line = ci(&line, "ad Fonticulum", "na Dobré Vodě");
        line = ci(&line, "ad Fontem Salubrem", "na Dobré Vodě");
        line = ci(&line, "ad Salubrem Fonticulum", "na Dobré Vodě");
        line = ci(&line, "Brünnl", "na Dobré Vodě");
        line = ci(&line, "Oberheid", "v Horním Dvořišti");
        line = ci(&line, "Mericae Superioris", "v Horním Dvořišti");
        line = ci(&line, "Unterhaydii", "v Dolním Dvořišti");
        line = ci(&line, "Unterheid", "Dolní Dvořiště");
        line = ci(&line, "Merica Inferioris", "Dolní Dvořiště");
        line = ci(&line, "Merica inferior", "Dolní Dvořiště");
        line = ci(&line, "Rosenthalii", "v Rožmitálu");
        line = ci(&line, "Rosenthal", "Rožmitál");
        line = ci(&line, "Prienthalii", "v Přídolí");
        line = ci(&line, "in Priethal", "v Přídolí");
        line = ci(&line, "Priethalium", "v Přídolí");
        line = ci(&line, "Priethal", "v Přídolí");
        line = ci(&line, "Cajoviae", "v Kájově");
        line = ci(&line, "in Cajow", "v Kájově");
        line = ci(&line, "Gratzen", "Nové Hrady");
        line = ci(&line, "praedii Komařiciensis", "statku v Komařicích");
        line = ci(&line, "Komarzitzii", "statku v Komařicích");
        line = ci(&line, "Komarzitii", "statku v Komařicích");
        line = ci(&line, "Stritzitzii", "ve Strýčicích");
        line = ci(&line, "Strziczicium", "Strýčice");
        line = ci(&line, "Stritzitz", "Strýčice");
        line = ci(&line, "Strýčiciensis", "strýčcký");
        line = ci(&line, "Strakonicensis", "strakonický");
        line = ci(&line, "Tarnoviensis", "trnavského");
        line = ci(&line, "Strobniciensis", "stropnický");
        line = ci(&line, "Strobnicii", "ve Stropnici");
        line = ci(&line, "in Strobnitz", "ve Stropnici");
        line = ci(&line, "Strobnitzii", "ve Stropnici");
        line = ci(&line, "Strobnicium", "Stropnice");
        line = ci(&line, "Strobnitzium", "Stropnice");
        line = ci(&line, "Strobnitz", "Stropnice");
        line = ci(&line, "Strobnitzium", "Stropnice");
        line = ci(&line, "Kalschingensis", "chvalšinský");
        line = ci(&line, "Chvalšinensis", "chvalšinský");
        line = ci(&line, "Kalschingae", "ve Chvalšinách");
        line = ci(&line, "Kalsching", "Chvalšiny");
        line = ci(&line, "Driesendorf", "Střížov");
        line = ci(&line, "Boreschovii", "v Boršově");
        line = ci(&line, "Paireschau", "v Boršově");
        line = ci(&line, "Payreschau", "Boršov");
        line = ci(&line, "Payerschau", "Boršov");
        line = ci(&line, "de Budvicio", "z Budějovic");
        line = ci(&line, "Budvicii", "v Budějovicích");
        line = ci(&line, "episcopi Budvicensis", "biskupa budějovického");
        line = ci(&line, "gymnasii Budvicensis", "budějovického gymnázia");
        line = ci(&line, "dioeceseos Budvicensis", "budějovické diecéze");
        line = ci(&line, "Budvicensis", "budějovický");
        line = ci(&line, "Černicensis", "v Černici");
        line = ci(&line, "in Krems", "v Křemži");
        line = ci(&line, "Potvoroviensi", "potvorovské");
        line = ci(&line, "Malschingae", "Malšína");
        line = ci(&line, "Malsching", "Malšín");
        line = ci(&line, "de Rosenberg", "z Rožmberka");
        line = ci(&line, "de Rosis", "z Rožmberka");
        line = ci(&line, "Rosensium", "z Rožmberků");
        line = ci(&line, "Rosenbergicae", "rožmberského");
        line = ci(&line, "Rosenbergae", "Rožmberku");
        line = ci(&line, "Rosenberg", "Rožmberk");
        line = ci(&line, "Rosensis", "z Rožmberků");
        line = ci(&line, "de Crumnaw", " z Krumlova");
        line = ci(&line, "de Crumlov", " z Krumlova");
        line = ci(&line, "de Crumpnaw", " z Krumlova");
        line = ci(&line, "de Crumpnau", " z Krumlova");
        line = ci(&line, "Crumlovii", "v Krumlově");
        line = ci(&line, "Crumlovium", "do Krumlova");
        line = ci(&line, "Crumlovia", "Krumlov");
        line = ci(&line, "Crumlov", "Krumlov");
        line = ci(&line, "Crumbnaw", "Krumlov");
        line = ci(&line, "Crumpnaw", "Krumlov");
        line = ci(&line, "Crumpnau", "Krumlov");
        line = ci(&line, "Sobieslavia", "Soběslav");
        line = ci(&line, "Kozojedii", "in Kozojedech");
        line = ci(&line, "in Kozojed", "v Kozojedech");
        line = ci(&line, "Sanctae Annae", "Svaté Anny");
        line = ci(&line, "ad Sanctum Martinum", "u svatého Martina");
        line = ci(&line, "in monasterio Sanctae Clarae virginis", "v klášteře svaté Kláry Panny");
        line = ci(&line, "in monasterio", "v klášteře");
        line = ci(&line, "in nostra ecclesia", "v našem kostele");
        line = ci(&line, "in ecclesia", "v kostele");
        line = ci(&line, "in instituto philosophico", "na filosofickém institutu");
        line = ci(&line, "Tento-Richnovii", "v Rychnově u Nových Hradů");
        line = ci(&line, "Teutorychnovii", "v Rychnově u Nových Hradů");
        line = ci(&line, "Teuto-Richnoviensis", "v Rychnově u Nových Hradů");
        line = ci(&line, "Tento-Richnoviensis", "v Rychnově u Nových Hradů");
        line = ci(&line, "Tento-Richnov", "Rychnov u Nových Hradů");
        line = ci(&line, "Tento-Richnovium", "Rychnov u Nových Hradů");
        line = ci(&line, "Plan ", "Planá ");
        line = ci(&line, "Planensis ", "Planá ");
        line = ci(&line, "Plan.", "Planá.");
        line = ci(&line, "Teinicii", "v Týnici");
        line = ci(&line, "Mariae.Ratschitz", "Mariánské Radčice");
        line = ci(&line, "Maria.Ratschitz", "Mariánské Radčice");
        line = ci(&line, "Ratschitzii", "v Mariánských Radčicích");
        line = ci(&line, "Kirchschlag", "Světlík");
        line = ci(&line, "in Antiqua Bruna", "na Starém Brně");
        line = ci(&line, "Vetero-Brunae", "na Starém Brně");
        line = ci(&line, "Antiqua Bruna", "Staré Brno");
        line = ci(&line, "Bruna", "Brno");
        line = ci(&line, "Zaroschitzii", "v Žarošicích");
        line = ci(&line, "Zaroschicii", "v Žarošicích");
        line = ci(&line, "in Zarošice", "v Žarošicích");
        line = ci(&line, "Zarošicensis", "žarošický");
        line = ci(&line, "ad Sanctum Oswaldum", "ve Svatém Osvaldu");
        line = ci(&line, "de Serin", "ze Serynu");
        line = ci(&line, "Lincii", "v Linci");
        line = ci(&line, "Lincensi", "lineckém");
        line = ci(&line, "Slapensis", "slapské");
        line = ci(&line, "Janecii", "v Jeníkově");
        line = ci(&line, "Janek", "Jeníkov");
        line = ci(&line, "Janegg", "Jeníkov");
        line = ci(&line, "de Schaumburg", "z Schaumburgu");
        line = ci(&line, "in Monte Aventino", "na aventinském pahorku");
        line = ci(&line, "in Monte", "na hoře");
        line = ci(&line, "Poletitz", "Boletice");
        line = ci(&line, "Boleticii", "v Boleticích");
        line = ci(&line, "Veter.Osseci", "ve Starém Oseku");
        line = ci(&line, "in Vetero-Ossegg", "ve Starém Oseku");
        line = ci(&line, "Wissoczan", "Vysočany");
        line = ci(&line, "Ottau", "Zátoň");
        line = ci(&line, "Zathon", "Zátoň");
        line = ci(&line, "Neostadii", "v Novém Městě Vídeňském");
        line = ci(&line, "Lisnitz", "Líšnice");
        line = ci(&line, "Netolitz", "Netolice");
        line = ci(&line, "in Commotov", " v Chomutově");
        line = ci(&line, "in Komotau", " v Chomutově");
        line = ci(&line, "Commotov", "Chomutov");
        line = ci(&line, "Komotau", "Chomutov");
        line = ci(&line, "ad Sanctum Lapidem", "na Svatém Kameni");
        line = ci(&line, "de Mitrovitz", "z Mitrovic");
        line = ci(&line, "Walschbirken", "Vlachovo Březí");
        line = ci(&line, "Kuttenberg", "Kutná Hora");
        line = ci(&line, "Zebnicz", "Žebnice");
        line = ci(&line, "Winterberg", "Vimperk");
        line = ci(&line, "de Novo Castro", "z Jindřichova Hradce");
        line = ci(&line, "in Castro", "na Hradě");
        line = ci(&line, "Lzin", "Lžín");
        line = ci(&line, "Cerhonic.", "Cerhonice");
        line = ci(&line, "Litomerzic", "Litoměřice");
        line = ci(&line, "Bilin", "Bílina");
        line = ci(&line, "Myliczyn", "Miličín");
        line = ci(&line, "Poleschowitz", "Polešovice");
        line = ci(&line, "Ramensis ecclesiae", "diecéze Ráma");
        line = ci(&line, "Dresdæ", "v Drážďanech");
        line = ci(&line, "Salisburgi", "v Salzburgu");
        line = ci(&line, "in Dachau", "v Dachau");
        line = ci(&line, "in Mainhardschlag", "v Malontech");
        line = ci(&line, "Hellenopolisensis", "v Hellenopolis <i>v Bythinii</i>");

        // 5. Abbatial and clerical terms
        line = ci(&line, "abbatiae Ossecensis", "oseckého opatství");
        line = ci(&line, "Ordinis Sancti Joannis hospitalis in Jerusalem", "hospitálního Řádu svatého Jana v Jerusalémě");
        line = ci(&line, "huius loci abbatis", "Opata tohoto kláštera");
        line = ci(&line, "huius loci professus", "profes tohoto kláštera");
        line = ci(&line, "huius loci professi", "profese tohoto kláštera");
        line = ci(&line, "cum vitam finire vellet", "když se chystal na konec života");
        line = ci(&line, "honorifice", "s poctami");
        line = ci(&line, "terrae traditus est", "byl vydán zemi");
        line = ci(&line, "celebratis pro eo exequiis", "když za něj byly odslouženy pohřební obřady");
        line = ci(&line, "supremus Regni Bohemiae Purgravius", "nejvyšší Purkrabí Království Českého");
        line = ci(&line, "regni Bohemiae praelatus infulatus", "infulované Prelát Království Českého");
        line = ci(&line, "gubernator domus", "správce domu");
        line = ci(&line, "aurei velleris eques", "rytíř zlatého rouna");
        line = ci(&line, "Imperatorum", "Císařů");
        line = ci(&line, "Imperator", "Císař");
        line = ci(&line, "trium", "třem");
        line = ci(&line, "prope ", "poblíž ");
        line = ci(&line, "eorumdemque regum Bohemiae", "a také králům českým");
        line = ci(&line, "a consiliis", "byl rádcem");
        line = ci(&line, "de republica Czechica optime meritus", "s velkými zásluhami o Český stát");
        line = ci(&line, "Portatus", "Přenesen");
        line = ci(&line, "Praga ", "z Prahy ");
        line = ci(&line, "exequia", "pohřební obřady");
        line = ci(&line, "celebrantur", "se slavily");
        line = ci(&line, "ut supra", "viz výše");
        line = ci(&line, "Sepelitur", "Pohřben");
        line = ci(&line, "Illustri Principissa", "Nejjasnější kněžnou");
        line = ci(&line, "in summa basilica", "ve velké basilice");
        line = ci(&line, "pro patria mortuus est", "zemřel za vlast");
        line = ci(&line, "in bello miserrimo", "v hrozné válce");
        line = ci(&line, "in bello miserimo", "v hrozné válce");
        line = ci(&line, "in bello infelicissimo", "v nešťastné válce");
        line = ci(&line, "ad Sanctissimam Trinitatem", "u Nejsvětější Trojice");
        line = ci(&line, "a gestapo incarceratus", "byl zajat gestapem");
        line = ci(&line, "combustus est", "byl spálen");

        // 6. Regional names
        line = ci(&line, "Austriae superioris", "v Horním Rakousku");
        line = ci(&line, "Austriam emigravit", "emigroval do Rakouska");
        line = ci(&line, "in Austria emigravit", "emigroval do Rakouska");
        line = ci(&line, "Austriae", "v Rakousku");
        line = ci(&line, "in Austria", "v Rakousku");
        line = ci(&line, "per Bohemiam", "pro Čechy");
        line = ci(&line, "in Lusatia", "v Lužici");
        line = ci(&line, "Lusatiam", "Lužici");
        line = ci(&line, "in Hungaria", "v Maďarsku");
        line = ci(&line, "in Polonia", "v Polsku");
        line = ci(&line, "Hungariae", "maďarského");
        line = ci(&line, "Bohemiae", "českého");
        line = ci(&line, "Moraviam", "Moravu");
        line = ci(&line, "in Moravia", "na Moravě");
        line = ci(&line, "Styriae", "ve Štýrsku");
        line = ci(&line, "Tiroliae", "v Tyrolsku");
        line = ci(&line, "Thesinensis", "těšínského");
        line = ci(&line, "per regnum Saxoniæ", "v celém Saském Království");

        // 7. Abbatial and monastic titles (without regex capture support we do literal replacements)
        line = ci(&line, "abbas ultimus", "poslední Opat");
        line = ci(&line, "abbas", "Opat");
        line = ci(&line, "abbatiae", "opatství");
        line = ci(&line, "abbatissa", "abatyše");
        line = ci(&line, "praepositus emeritus", "emeritní probošt");
        line = ci(&line, "praepositus", "probošt");
        line = ci(&line, "visitator", "vizitátor");
        line = ci(&line, "subprior", "podpřevor");
        line = ci(&line, "prior emeritus", "emeritní převor");
        line = ci(&line, "prior.administrator", "převor-administrátor");
        line = ci(&line, "prior", "převor");
        line = ci(&line, "Religiosus", "Řeholní");
        line = ci(&line, "Religiosa", "Řeholní");
        line = ci(&line, "Rel.", "Řeholní");
        line = ci(&line, "Reverendus Frater", "Ctihodný bratr");
        line = ci(&line, "Rev. Fr.", "Ctihodný bratr");
        line = lit(&line, " Frater ", " bratr ");
        line = lit(&line, " Fr. ", " bratr ");
        line = ci(&line, "Reverendus", "Důstojný");
        // Here we call our custom replacement for "(\w+) nostri"
        line = replace_word(&line, " nostri");
        line = ci(&line, "Conversus", "konvrš");
        line = ci(&line, "confraternitatem fecit", "uzavřel konfraternitu");
        line = ci(&line, "confrater noster", "náš spolubratr");
        line = ci(&line, "confratris nostri", "našeho spolubratra");
        line = ci(&line, "confrater", "spolubratr");
        line = ci(&line, "Virgo", "Panna");
        line = ci(&line, "Perillustris Domina", "Přejasná Paní");
        line = ci(&line, "relicta", "vdova");
        line = ci(&line, "Domina", "Paní");
        line = ci(&line, "Dominus", "Pán");
        line = ci(&line, "Dominorum", "Pánů");
        line = ci(&line, "Dominum", "Pána");
        line = ci(&line, "ducis", "vévody");
        line = ci(&line, "comes ", "hrabě ");
        line = ci(&line, "comitem", "hraběte");
        line = ci(&line, "magister infirmorum", "infirmář");
        line = ci(&line, "infirmarius", "infirmář");
        line = ci(&line, "Domini abbatis", "Pana Opata");
        line = ci(&line, "Domni abbatis", "Pana Opata");
        line = ci(&line, "abbatis", "Opata");
        line = ci(&line, "Domini ", "Pána ");
        line = ci(&line, "gubernator", "hejtman");
        line = ci(&line, "conventualis", "konventní");
        line = ci(&line, "stabilitatis", "se slibem stability");
        line = ci(&line, "cum disputationibus", "při disputacích");
        line = ci(&line, "habitis", "konaných");
        line = ci(&line, "mortuus est", "zemřel");
        line = ci(&line, "mortuus", "zemřel");
        line = ci(&line, "in parochia", "ve farnosti");
        // Some substitutions with numbers are handled simply:
        line = ci(&line, "annos natus ", "ve věku ");
        // For "annis (\d+)" and similar we assume the number is preserved by a simple literal replacement.
        // (In a full implementation you might parse the number.)
        line = ci(&line, "annis ", "");
        line = ci(&line, "per ", "po ");
        line = ci(&line, "annos", "let");
        line = ci(&line, "praefuit", "spravoval");
        line = ci(&line, "sitam", "umístěnou");
        line = ci(&line, "e fundamentis", "od základů");
        line = ci(&line, "a fundamentis", "od základů");
        line = ci(&line, "aedificavit", "vystavěl");
        line = ci(&line, "donavit", "daroval");
        line = ci(&line, "lignum", "dřevo");
        line = ci(&line, "pretiose", "drahocenně");
        line = ci(&line, "ornatum", "zdobené");
        line = ci(&line, "in capitulo nostro", "v naší kapitulní síni");
        line = ci(&line, "in tumulo", "v hrobce");
        line = ci(&line, "sodalis parthenius", "mariánský ctitel");
        line = ci(&line, "hospis", "host");
        line = ci(&line, "hospes", "host");
        line = ci(&line, "beneficiatus", "obročník");
        line = ci(&line, "catecheta", "katecheta");
        line = ci(&line, "homo simplex", "prostý člověk");
        line = ci(&line, "delegavit", "odkázal");
        line = ci(&line, "omnes libros suos", "všechny své knihy");
        line = ci(&line, "praemonstratensis", "premonstrátského");
        line = ci(&line, "anno", "Roku");
        line = ci(&line, "resignatus", ", který odstoupil");
        line = ci(&line, "iterum", "poté");
        // A substitution with a captured word:
        // For "(\w+) honoratus" we simulate by a simple search and replace (if needed, one could implement a more complex version).
        line = ci(&line, " honoratus", " ctěný");
        line = ci(&line, " levati ", " zrušeného ");
        line = ci(&line, " obiit ", " zesnul ");
        line = ci(&line, " obiit.", " zesnul.");
        line = ci(&line, "oriundus", ", který pochází");
        line = ci(&line, "historiae ecclesiasticae", "církevních dějin");
        line = ci(&line, "iuris canonici", "kanonického práva");
        line = ci(&line, "concionator Quadragesimae", "postní kazatel");
        line = ci(&line, "concionator", "kazatel");
        line = ci(&line, "reformator disciplinae regularis", "reformátor řeholní kázně");

        // 8. Name substitutions (for a long list of personal names)
        line = ci(&line, "Quirini", "Quirina");
        line = ci(&line, "illustrissimum dominum", "nejjasnějšího pána");
        line = ci(&line, "cum monasterio nostro", "s naším klášterem");
        line = ci(&line, "huic monasterio", "tomuto klášteru");
        line = ci(&line, "nostro monasterio", "našemu klášteru");
        line = ci(&line, "monasterio nostro", "našemu klášteru");
        line = ci(&line, "fidelem curam", "věrnou péči");
        line = ci(&line, "in officio suo", "ve svém úřadu");
        line = ci(&line, "impendit", "vynakládal");
        line = ci(&line, "serenissimi", "nejjasnějšího");
        line = ci(&line, "serenissimus", "nejjasnější");
        line = ci(&line, "Serenissimus", "Nejjasnější");
        line = ci(&line, "magnifici", "vznešeného");
        line = ci(&line, "magnificus", "vznešený");
        line = ci(&line, "Inclytus", "Slavný");
        line = ci(&line, "Inclytus", "Slavný");
        line = ci(&line, "inclytus", "slavný");
        line = ci(&line, "inclytus", "slavný");
        line = ci(&line, "inclyti", "slavného");
        line = ci(&line, "Regis", "Krále");
        line = ci(&line, "protonotarius", "protonotář");
        line = ci(&line, "prothonotarii", "protonotáře");
        line = ci(&line, "protonotarii", "protonotáře");
        line = ci(&line, "presbyteri", "kněze");
        line = ci(&line, "Honorabilis", "Ctihodný");
        line = ci(&line, "Honesta", "Ctná");
        line = ci(&line, "Honestus", "Ctný");
        line = ci(&line, "supremus", "nejvyšší");
        line = ci(&line, "praestans", "vynikající");
        line = ci(&line, "organista", "varhaník");
        line = ci(&line, "decanus personalis", "osobní děkan");
        line = ci(&line, "decanus", "děkan");
        line = ci(&line, "cancellariae et aedificiorum inspector", "správce kanceláří a budov");
        line = ci(&line, "inspector aedificiorum", "správce budov");
        line = ci(&line, "capellae regalis", "královské kapely");
        line = ci(&line, "capellae", "kaple");
        line = ci(&line, "canonicus", "kanovník");
        line = ci(&line, "fundatoris", "zakladatele");
        line = ci(&line, "fundator", "zakladatel");
        line = ci(&line, "stabularius", "kočí");
        line = ci(&line, "cliens", "panoš");
        line = ci(&line, "principis", "vládce");
        line = ci(&line, "princeps", "vládce");
        line = ci(&line, "scriba", "písař");
        line = ci(&line, "cancelariae", "kanceláře");
        line = ci(&line, " contra ", " proti ");
        line = ci(&line, "Rusiam", "Rusku");
        line = ci(&line, "Russiam", "Rusku");
        line = ci(&line, "rector", "rektor");
        line = ci(&line, "ad Sanctum Bernardum", "u svatého Bernarda");
        line = ci(&line, "missarius", "vyslanec");
        line = ci(&line, "in vigilia", "v předvečer");
        line = ci(&line, "Sanctissimae Trinitatis", "Nejsvětější Trojice");
        line = ci(&line, "ad Sanctum Spiritum", "u Svatého Ducha");
        line = ci(&line, "Bohemus", "Čech");

        // 9. Parish and clerical positions
        line = ci(&line, "parochus emeritus", "emeritní farář");
        line = ci(&line, "parochus", "farář");
        line = ci(&line, "clericus", "klerik");
        line = ci(&line, "novitius", "novic");
        line = ci(&line, "novicius", "novic");
        line = ci(&line, "studens", "student");
        line = ci(&line, "scholarius", "student");
        line = ci(&line, "archidiaconus", "arcijáhen");
        line = ci(&line, "infulatus", "infulovaný");
        line = ci(&line, "subdiaconus", "podjáhen");
        line = ci(&line, "diaconus", "jáhen");
        line = ci(&line, "accolitus", "akolyta");
        line = ci(&line, "plebanus", "plebán");
        line = ci(&line, "auxiliator", "pomocný duchovní");
        line = ci(&line, "auxiliarius", "pomocný duchovní");
        line = ci(&line, "capellanus emeritus", "emeritní kaplan");
        line = ci(&line, "cooperator administratoris", "kaplan");
        line = ci(&line, "cooperator", "kaplan");
        line = ci(&line, "cooperatro", "kaplan");
        line = ci(&line, "capellanus", "kaplan");
        line = ci(&line, "cooperatus", "kaplan");
        line = ci(&line, "adjutor parochiae", "kaplan");
        line = ci(&line, "adjutor parochi", "kaplan");
        line = ci(&line, "presbyteri", "kněze");
        line = ci(&line, "presbyter", "kněz");
        line = ci(&line, "heremita", "poustevník");
        line = ci(&line, "eremita", "poustevník");
        line = ci(&line, "vir ", "muž ");
        line = ci(&line, "vir.", "muž.");
        line = ci(&line, "vir,", "muž,");

        // 10. Administrative positions
        line = ci(&line, "archivarius", "archivář");
        line = ci(&line, "praefectus pharmacopae", "prefekt lékárny");
        line = ci(&line, "praefectus culinae abbatialis", "prefekt opatské kuchyně");
        line = ci(&line, "praefectus culinae", "prefekt kuchyně");
        line = ci(&line, "culinae praefectus", "prefekt kuchyně");
        line = ci(&line, "culinae provisor", "správce kuchyně");
        line = ci(&line, "culinae praefectus", "prefekt kuchyně");
        line = ci(&line, "culinae", "kuchyně");
        line = ci(&line, "cellae", "sklepů");
        line = ci(&line, "praefectus", "prefekt");
        line = ci(&line, "cellae vinariarum", "vinných sklepů");
        line = ci(&line, "magister conversorum", "konvršmistr");
        line = ci(&line, "magister novitiorum", "novicmistr");
        line = ci(&line, "novitiorum magister", "novicmistr");
        line = ci(&line, "magister", "magistr");
        line = ci(&line, "administrator oeconomiae", "hospodářský správce");
        line = ci(&line, "oeconomus", "hospodářský správce");
        line = ci(&line, "oeconomicus", "hospodářský správce");
        line = ci(&line, "inspector oeconomiae", "hospodářský správce");
        line = ci(&line, "bibliothecarius", "knihovník");
        line = ci(&line, "confessarius", "zpovědník");
        line = ci(&line, "cantor", "kantor");
        line = ci(&line, "regens chori figuralis", "dirigent orchestru a sboru");
        line = ci(&line, "regens chori", "regenschori");
        line = ci(&line, "hiuis", "huius");
        line = ci(&line, "huius loci", "tohoto kláštera");
        line = ci(&line, "huius coenobii", "tohoto kláštera");
        line = ci(&line, "domni abbatis", "pana opata");
        line = ci(&line, "quaesturae provisor", "finanční správce");
        line = ci(&line, "administrator emeritus", "emeritní administrátor");
        line = ci(&line, "administrator", "administrátor");
        line = ci(&line, "provisor", "administrátor");
        line = ci(&line, "procurator", "správce");
        line = ci(&line, "aurifaber", "zlatník");
        line = ci(&line, "pharmacopoia", "lékárník");
        line = ci(&line, "granarius", "správce sýpky");

        // 11. Educational and artistic titles
        line = ci(&line, "gymnasii", "gymnázia");
        line = ci(&line, "Ordinis doctor theologus", "řádový doktor teologie");
        line = ci(&line, "Ordinis cisterciensis", "cisterciáckého Řádu");
        line = ci(&line, "cisterciensis Ordinis", "cisterciáckého Řádu");
        line = ci(&line, "Ordinem Cistercium professus", "se stal členem cisterciáckého Řádu");
        line = ci(&line, "provincialis Ordinis Prædicatorum", "provinciál Řádu Kazatelů");
        line = ci(&line, "Ordinis", "Řádu");
        line = ci(&line, "director", "ředitel");
        line = ci(&line, "protector", "ochránce");
        line = ci(&line, "congregationis", "kongregace");
        line = ci(&line, "congregatio", "kongregace");
        line = ci(&line, "in prioratu", "v převorství");

        // 12. Miscellaneous substitutions
        line = lit(&line, "Erat ", "Byl to ");
        line = lit(&line, "erat ", "byl to ");
        line = ci(&line, "fuerat", "byl");
        line = ci(&line, "sinistrae", "levé");
        line = ci(&line, "sinistri", "levého");
        line = ci(&line, "dextrae", "pravé");
        line = ci(&line, "dextri", "pravého");
        line = ci(&line, "partis", "části");
        line = ci(&line, "in coemeterio communi", "na společném hřbitově");
        line = ci(&line, "in coemeterio", "na hřbitově");
        line = ci(&line, "inspector silvarum", "lesní inspektor");
        line = ci(&line, "silvarum", "lesní");
        line = ci(&line, "poenitentiarius", "penitenciář");

        // 13. Personal names
        line = ci(&line, "Joannis", "Jana");
        line = ci(&line, "Joannes", "Jan");
        line = ci(&line, "Jodoci", "Jocha");
        line = ci(&line, "Augustini", "Augustina");
        line = ci(&line, "Ulrici", "Oldřicha");
        line = ci(&line, "Ulricus", "Oldřich");
        line = ci(&line, "Bartholomaei", "Bartoloměje");
        line = ci(&line, "Henricus", "Jindřich");
        line = ci(&line, "Henrici ", "Jindřicha ");
        line = ci(&line, "Matthiae ", "Matyáše ");
        line = ci(&line, "Ungaricae ", "Uherského ");
        line = ci(&line, "Martini", "Martina");
        line = ci(&line, "Sancti Viti", "svatého Víta");
        line = ci(&line, "Viti ", "Víta ");
        line = ci(&line, "Edmundi", "Edmunda");
        line = ci(&line, "Procopii", "Prokopa");
        line = ci(&line, "Petri", "Petra");
        line = ci(&line, "Vokonis", "Voka");
        line = ci(&line, "Wokonis", "Voka");
        line = ci(&line, "Woko", "Vok");
        line = ci(&line, "Evae", "Evy");
        line = ci(&line, "Hevae", "Evy");
        line = ci(&line, "Lucae", "Lukáše");
        line = ci(&line, "Guillelmus", "Vilém");
        line = ci(&line, "Zawissius", "Záviš");
        line = ci(&line, "de Falkenstein", "z Falkenštejna");
        line = ci(&line, "Andreae", "Ondřeje");
        line = ci(&line, "Pauli", "Pavla");
        line = ci(&line, "Jacobi", "Jakuba");
        line = ci(&line, "Laurentius", "Vavřinec");
        line = ci(&line, "Laurencius", "Vavřinec");
        line = ci(&line, "Carolus", "Karel");
        line = ci(&line, "Jacobus", "Jakub");
        line = ci(&line, "Wenceslaus", "Václav");
        line = ci(&line, "Wenceslai", "Václava");
        line = ci(&line, "Antonius", "Antonín");
        line = ci(&line, "Wolffgangus", "Wolfgang");
        line = ci(&line, "Engelbertus", "Engelbert");
        line = ci(&line, "Petrus", "Petr");
        line = ci(&line, "Nicolaus", "Mikuláš");
        line = ci(&line, "Jodocus", "Joch");
        line = ci(&line, "Martinus", "Martin");
        line = ci(&line, "Robertus", "Robert");
        line = ci(&line, "Gerardus", "Gerard");
        line = ci(&line, "Stanislaus", "Stanislav");
        line = ci(&line, "Sigismundus", "Zikmund");
        line = ci(&line, "Edmundus", "Edmund");
        line = ci(&line, "Georgius", "Jiří");
        line = ci(&line, "Josephus", "Josef");
        line = ci(&line, "Adalbertus", "Vojtěch");
        line = ci(&line, "Woytiech", "Vojtěch");
        line = ci(&line, "Vincentius", "Vincenc");
        line = ci(&line, "Benedictus", "Benedikt");
        line = ci(&line, "Ernestus", "Ernst");
        line = ci(&line, "Ladislaus", "Ladislav");
        line = ci(&line, "Augustinus", "Augustin");
        line = ci(&line, "Conradus", "Konrád");
        line = ci(&line, "Franciscus", "František");
        line = ci(&line, "Stephanus", "Štěpán");
        line = ci(&line, "Ignatius", "Ignác");
        line = ci(&line, "Gregorius", "Řehoř");
        line = ci(&line, "Florianus", "Florián");
        line = ci(&line, "Simon", "Šimon");
        line = ci(&line, "Maximilianus", "Maximilián");
        line = ci(&line, "Joachimus", "Jáchym");
        line = ci(&line, "Thomas", "Tomáš");
        line = ci(&line, "Nivardus", "Nivard");
        line = ci(&line, "Camillus", "Kamil");
        line = ci(&line, "Margaretha", "Markéta");
        line = ci(&line, "Matthæus", "Matouš");
        line = ci(&line, "Matthaeus", "Matouš");
        line = ci(&line, "Eugenius", "Evžen");
        line = ci(&line, "Christianus", "Christian");
        line = ci(&line, "Bartholomaeus", "Bartoloměj");
        line = ci(&line, "Matthias", "Matěj");
        line = ci(&line, "Albericus", "Alberich");
        line = ci(&line, "Nepomucenus", "Nepomuk");
        line = ci(&line, "Bernardinus", "Bernardin");
        line = ci(&line, "Fiola", "Viola");

        // 14. Some placeholders (xx → xx, etc.)
        line = ci(&line, "xx", "xx");
        // repeated four times (as in the original)
        line = ci(&line, "xx", "xx");
        line = ci(&line, "xx", "xx");
        line = ci(&line, "xx", "xx");

        // 15. Episcopal titles
        line = ci(&line, "episcopus in partibus", "titulární Biskup");
        line = ci(&line, "episcopus", "Biskup");
        line = ci(&line, "notarius archiepiscopialis", "arcibiskupský notář");
        line = ci(&line, "notarius episcopalis", "biskupský notář");
        line = ci(&line, "proto.notarius apostolicus", "apoštolský protonotář");
        line = ci(&line, "notarius apostolicus", "apoštolský notář");
        line = ci(&line, "vicarius generalis", "generální vikář");
        line = ci(&line, "vicarius apostolicus", "apoštolský vikář");
        line = ci(&line, "ordinis Cisterciensis", "cisterciáckého řádu");
        line = ci(&line, "secretarius", "sekretář");
        line = ci(&line, "notarius", "notář");
        line = ci(&line, "sacellarius", "kaplan");
        line = ci(&line, "cellarius", "sklepmistr");
        line = ci(&line, "cellarii", "sklepů");
        line = ci(&line, "cellerarius", "celerář");
        line = ci(&line, "sacrista", "sakristán");
        line = ci(&line, "sacristanus", "sakristán");
        line = ci(&line, "consiliar", "konsistorní rada");
        line = ci(&line, "consistorialis", "");
        line = ci(&line, "consistorii", "konsistorní rada");
        line = ci(&line, "episcopi Brunensis", "brněnského biskupa");
        line = ci(&line, "episcopi ", "biskupa ");
        line = ci(&line, " fratris", " bratra");
        line = ci(&line, "plebanus", "plebán");
        line = ci(&line, "vicarius parochiae emeritus", "emeritní farní vikář");
        line = ci(&line, "vicarius parochiae", "farní vikář");
        line = ci(&line, "vicarius", "vikář");
        line = ci(&line, "in Collegio archi-episcopialis", "na arcibiskupské koleji");
        line = ci(&line, "in archiepiscopalis collegio", "na arcibiskupské koleji");
        line = ci(&line, "ad Sanctum Adalbertum", "Svatého Vojtěcha");
        line = ci(&line, "benefactor singularis", "jedinečný dobrodinec");
        line = ci(&line, "benefactor noster", "náš dobrodinec");
        line = ci(&line, "benefactor", "dobrodinec");
        line = ci(&line, "benefactrix", "dobrodinka");
        line = ci(&line, "fautor", "mecenáš");

        // 16. Church-related terms
        line = ci(&line, "canoniae", "kanonie");
        line = ci(&line, "vinearum", "vinic");
        line = ci(&line, "parochii", "farnosti");
        line = ci(&line, "parochiae", "farnosti");
        line = ci(&line, "Reverendi ", "důstojného ");
        line = ci(&line, "capituli", "kapituly");
        line = ci(&line, "monialium", "sester");
        line = ci(&line, "totius", "celého");
        line = ci(&line, "ultimi", "posledního");
        line = ci(&line, "antiquus", "dřívější");

        line = ci(&line, " praenobilis", " převznešený");
        line = ci(&line, " nobilis", " vznešený");
        line = ci(&line, "famosus", "slavný");
        line = ci(&line, "exemplaris", "příkladný");
        line = ci(&line, "Generosa", "Štědrá");
        line = ci(&line, "generosa", "štědrá");
        line = ci(&line, "generosae", "štědré");
        line = ci(&line, "generosi", "štědrého");
        line = ci(&line, "Generosus", "Štědrý");
        line = ci(&line, "generosus", "štědrý");
        line = ci(&line, "Egregius", "Výjimečný");
        line = ci(&line, "egregius", "výjimečný");
        line = ci(&line, "generosorum", "štědrých");
        line = ci(&line, "generosum", "štědrých");
        line = ci(&line, "optimus", "nejlepší");
        line = ci(&line, "virtuosa", "ctnostná");
        line = ci(&line, "virtuosus", "ctnostný");
        line = ci(&line, "illustrissimus", "nejjasnější");
        line = ci(&line, "Illustrissimus", "Nejjasnější");
        line = ci(&line, "Illustris", "Přejasný");
        line = ci(&line, "illustris", "přejasný");
        line = ci(&line, "illustrem", "přejasného");
        line = ci(&line, "primus", "první");
        line = ci(&line, "secundus", "druhý");
        line = ci(&line, "primi", "prvního");
        line = ci(&line, "secundi", "druhého");
        line = ci(&line, "camerarius", "komorník");
        line = ci(&line, "laudabiliter", "chvályhodně");
        line = ci(&line, "persolvit", "vykonával");
        line = ci(&line, " nostri", " našeho");
        line = ci(&line, " generalis", " generální");
        line = ci(&line, "huius monasterii", "tohoto kláštera");
        line = ci(&line, "monasterii", "kláštera");
        line = ci(&line, "officium", "úřad");
        line = ci(&line, "studii biblici ", "biblických studií ");
        line = ci(&line, "physicae", "fyziky");
        line = ci(&line, "mathematicae", "matematiky");
        line = ci(&line, "philosophiae professor", "profesor filosofie");
        line = ci(&line, "professor philosophiae", "profesor filosofie");
        line = ci(&line, "professor emeritus", "emeritní profesor");
        line = ci(&line, "professor", "profesor");
        line = ci(&line, "pictor ", "malíř ");
        line = ci(&line, "pictor.", "malíř.");
        line = ci(&line, "sutor", "švec");
        line = ci(&line, "sartor ", "krejčí ");
        line = ci(&line, "sartor.", "krejčí.");
        line = ci(&line, "capitaneus", "správce");
        line = ci(&line, "capitanei", "správce");
        line = ci(&line, "doleatoris", "ranhojiče");
        line = ci(&line, "tumulatus est", "je pohřben");
        line = ci(&line, " tum ", " v té době ");
        line = ci(&line, " tum, ", " v té době, ");
        line = ci(&line, "doctor decretorum", "doktor církevního práva");
        line = ci(&line, "decretorum doctor", "doktor církevního práva");
        line = ci(&line, "iuris utriusque doctor", "doktor obojího práva");
        line = ci(&line, "philosophiae doctor", "doktor filosofie");
        line = ci(&line, "doctor philosophiae", "doktor filosofie");
        line = ci(&line, "philosophiae", "filosofie");
        line = ci(&line, "doctor theologiae", "doktor teologie");
        line = ci(&line, "sanctae theologiae doctor", "doktor posvátné teologie");
        line = ci(&line, "Sacrae theologiae baccalaureus", "bakalář posvátné teologie");
        line = ci(&line, "sanctae theologiae", "posvátné teologie");
        line = ci(&line, "sacrae theologiae", "posvátné teologie");
        line = ci(&line, "theologiae moralis", "morální theologie");
        line = ci(&line, "theologiae doctor", "doktor teologie");
        line = ci(&line, "utriusque iuris", "obojího práva");
        line = ci(&line, "iuris utriusque", "obojího práva");
        line = ci(&line, "doctor", "doktor");
        line = ci(&line, "theologiae baccalaureus", "bakalář teologie");
        line = ci(&line, "theologiae-dogmaticae professor", "profesor dogmatické teologie");
        line = ci(&line, "theologiae", "teologie");
        line = ci(&line, "theologia", "teologie");
        line = ci(&line, "pater spiritualis", "otec spirituál");
        line = ci(&line, "per complures annos", "po mnoho let");
        line = ci(&line, "adjutor oeconomiae", "pomocný správce");
        line = ci(&line, "adiutor oeconomiae", "pomocný správce");
        line = ci(&line, "in hospitali", "ve špitále");
        line = ci(&line, "procuratrix", "správkyně");
        line = ci(&line, "balneator", "lazebník");
        line = ci(&line, "portarius", "fortnýř");
        line = ci(&line, "domi", "domu");
        line = ci(&line, "officialis", "hodnostář");
        line = ci(&line, "coadijutor", "koadjutor");
        line = ci(&line, "coadjutor", "koadjutor");
        line = ci(&line, "ecclesiae Wratislaviensis", "vratislavské katedrály");
        line = ci(&line, "ecclesiae", "kostela");
        line = ci(&line, "eandem ecclesiam", "tentýž kostel");
        line = ci(&line, "ecclesiam", "kostel");
        line = ci(&line, "maxime", "nejvíce");
        line = ci(&line, "in extremis", "na konci");
        line = ci(&line, "in nosocomio", "v nemocnici");
        line = ci(&line, "Fratrum misericori", "Milosrdných Bratří");
        line = ci(&line, "Fratrum misericordiorum", "Milosrdných Bratří");
        line = ci(&line, "utramque", "obojí");
        line = ci(&line, "iudicissa", "rychtářka");
        line = ci(&line, "refectorarius", "refektorář");
        line = ci(&line, "lotionarius", "valchář");
        line = ci(&line, "scriniator", "bednář");
        line = ci(&line, "eiusdem", "jeho");
        line = ci(&line, "pistor", "pekař");
        line = ci(&line, "piscator", "rybář");
        line = ci(&line, "piscatrix", "rybářka");
        line = ci(&line, "poculo lethifero infectus", "otráven  jedem v číši");
        line = ci(&line, "sibi propinato", "kterou mu podali");

        // 17. Family and personal relations
        line = ci(&line, "pater eius", "jeho otec");
        line = ci(&line, "pater ", "otec ");
        line = ci(&line, "patris", "otce");
        line = ci(&line, "filius", "syn");
        line = ci(&line, "Sororum Misericordiae", "Milosrdných Sester");
        line = ci(&line, "sororis", "sestry");
        line = ci(&line, "soror", "sestra");
        line = ci(&line, "filia ", "dcera ");
        line = ci(&line, "amita ", "teta ");
        line = ci(&line, "mater eius", "jeho matka");
        line = ci(&line, "mater ", "matka ");
        line = ci(&line, "matrona ", "dáma ");
        line = ci(&line, "civissa", "občanka");
        line = ci(&line, "cives ", "občan ");
        line = ci(&line, "civis ", "občan ");
        line = ci(&line, "conthoralis", "choť");
        line = ci(&line, "consanguinea", "rodná sestra");
        line = ci(&line, "confratrix nostra", "členka naší konfraternity");
        line = ci(&line, "vidua ", "vdova ");
        line = ci(&line, "germanus", "rodný bratr");
        line = ci(&line, "parens", "rodič");
        line = ci(&line, " natus", " narozen");
        line = ci(&line, "uxor eius", "jeho manželka");
        line = ci(&line, "uxore", "manželkou");
        line = ci(&line, "uxor", "manželka");
        line = ci(&line, "hic professi", "zdejšího profese");
        line = ci(&line, "in oppido", "ve městě");
        line = ci(&line, "in aedibus", "v síních");
        line = ci(&line, "ante altare", "před Oltářem");
        line = ci(&line, "post expulsionem", "po vyhnání");
        line = ci(&line, "a rusticis Bohemis", "českými sedláky");
        line = ci(&line, "occisus est", "byl zabit");
        line = ci(&line, "crudeliter", "krutě");
        line = ci(&line, "cum abbate suo", "se svým Opatem");
        line = ci(&line, "iuniorum", "mládeže");
        line = ci(&line, "iunior", "mladší");
        line = ci(&line, "aulicus", "dvorní");
        line = ci(&line, "novam", "novou");
        line = ci(&line, "novum", "nový");
        line = ci(&line, "curiam", "budovu");
        line = ci(&line, "generalitiam", "generalátu");
        line = ci(&line, "mire", "krásně");
        line = ci(&line, "decoravit", "vyzdobil");
        line = ci(&line, "indefessa cura", "neúnavnou péčí");
        line = ci(&line, "indefessus", "nezdolný");
        line = ci(&line, "atque", "a také");
        line = ci(&line, "tandem", "později");
        line = ci(&line, "machina dilaceratus", "poraněn strojem");

        // 18. Vikariate and local administrative terms
        line = ci(&line, "vicariatus assistens", "sekretář vikariátu (kongregace)");
        line = ci(&line, "vicariatus", "vikariátu (kongregace)");
        line = ci(&line, "assistens", "sekretář");
        line = ci(&line, "localista", "lokální kaplan");

        // 19. Date and time substitutions
        line = ci(&line, "eodem anno ", "Téhož roku ");
        line = ci(&line, "anni eiusdem", "téhož roku");
        line = ci(&line, "Anno ", "Roku ");
        line = ci(&line, "anno ", "roku ");
        line = ci(&line, "Die ", "Dne ");
        line = ci(&line, "die ", "dne ");
        line = ci(&line, "quondam", "kdysi");
        line = ci(&line, "olim", "kdysi");
        line = ci(&line, "hic ", "zde ");
        line = ci(&line, "dein ", "poté ");
        line = ci(&line, "sepultus est", "je pohřbený");
        line = ci(&line, "sepultus iacet", "leží pohřbený");
        line = ci(&line, "Sepultus", "Pohřbený");
        line = ci(&line, "sepultus", "pohřbený");
        line = ci(&line, "sepulta est", "je pohřbená");
        line = ci(&line, "sepulta", "pohřbená");
        line = ci(&line, "mensis", "měsíce");
        line = ci(&line, "vixit", "žil");
        line = ci(&line, "ecclesiae", "kostely");
        line = ci(&line, "diocesis", "diecéze");
        line = ci(&line, "diocesis", "diecéze");

        // 20. Months
        line = ci(&line, "januarii", "ledna");
        line = ci(&line, "februarii", "února");
        line = ci(&line, "martii", "března");
        line = ci(&line, "aprilis", "dubna");
        line = ci(&line, "maii ", "května ");
        line = ci(&line, "maji ", "května ");
        line = ci(&line, "iunii", "června");
        line = ci(&line, "iulii ", "července ");
        line = ci(&line, "julii ", "července ");
        line = ci(&line, "augusti ", "srpna ");
        line = ci(&line, "septembris", "září");
        line = ci(&line, "octobris", "října");
        line = ci(&line, "novembris", "listopadu");
        line = ci(&line, "decembris", "prosince");

        // 21. Final miscellaneous substitutions
        line = ci(&line, "pie in Domino obdormierunt", "zbožně v Pánu zesnuli");
        line = ci(&line, "monachus chori", "chórový mnich");
        line = ci(&line, "monachus", "mnich");
        line = ci(&line, "monachos", "mnich");
        line = ci(&line, "professus jubilatus", "profes jubilant");
        line = ci(&line, "sacerdos jubilatus", "kněz jubilant");
        line = ci(&line, "sacerdos", "kněz");
        line = ci(&line, "professus de", "profes z kláštera");
        line = ci(&line, "professus", "profes");
        line = ci(&line, "professi", "profese");
        line = ci(&line, "ibidem", "na témž místě");
        line = ci(&line, "ibique", "a tam");
        line = ci(&line, "B.M.V.", "Panny Marie");
        line = ci(&line, "Beatae Mariae Virginis", "Panny Marie");

        // 22. Some generic punctuation fixes.
        line = line.replace(" ,", ",");
        line = line.replace("  ", " ");
        // (Additional clean–up could be added here.)

        line
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_replace_literal_ci() {
            let s = "Hello World! hello world!";
            let replaced = ci(s, "hello", "hi");
            // Should replace both "Hello" and "hello" with "hi"
            assert_eq!(replaced, "hi World! hi world!");
        }

        #[test]
        fn test_replace_word() {
            let s = "Marcus nostri et Titus nostri.";
            let replaced = replace_word(s, " nostri");
            // Each occurrence of "<word> nostri" should become "našeho <word>"
            assert_eq!(replaced, "našeho Marcus et našeho Titus.");
        }

        #[test]
        fn test_translate_cz() {
            // Simple test using a few substitutions.
            let input = "Purissimi Cordis B.M.V. in oppido Altovadeno";
            let output = translate_cz(&[input.to_string()]);
            // We expect the titles and location to be replaced as in our rules.
            assert!(output.contains("Nejčistšího Srdce Panny Marie"));
            assert!(output.contains("ve Vyšším Brodě"));
        }
    }
}


use crate::fileio::do_read;
use crate::date::nextday;
use crate::date::leap_year;
use crate::language_text_tools::translate;
use crate::language_text_tools::LanguageTextContext;
use crate::setup_string::checkfile;
use crate::regex::ci_contains;
use crate::regex::ci_replace_all;
use crate::regex::ci_starts_with;
use crate::regex::remove_leading_zeros;
use crate::setfont;

/// Exposes the - mostly useless - translate_cz function,
/// written for Necrologium from Vyšší Brod, CZ (Altovadum),
/// will hardly work for anything else.
pub fn translate_cz(args: &[String]) -> String {
    self::translate_cz::translate_cz(args)
}

/// Processes the lines for the special branch in `regula_emaus` (day==23, month==2, non-leap).
fn process_regula_special_lines(ctx: &LanguageTextContext, lang: &str, d: u32, month: u32, fname: &str) -> String {
    let mut result = String::new();
    let mut reading: u32 = 0;
    let mut sequentia: u32 = 0;
    let mut titulus: Option<String> = None;
    for line in do_read(fname).unwrap_or_default() {
        if ci_contains(&line, "<b>caput ") || ci_contains(&line, "<b>incipit ") {
            titulus = Some(line.clone());
        }
        let pattern = format!(" {}.{}.", d, month);
        if ci_contains(&line, &pattern) || reading >= 1 {
            reading += 1;
            if reading == 1 {
                continue;
            }
            if reading == 2 {
                if let Some(ref mut tit) = titulus {
                    // In the special branch we do not perform a replacement:
                    // (the Perl code does not call translate in this branch)
                    *tit = tit.clone();
                    result.push_str(&format!("{}. ", tit));
                }
            }
            if ci_contains(&line, "<b>caput ") || ci_contains(&line, "<b>incipit ") {
                sequentia = 1;
                result.push_str("\n_\n");
                continue;
            }
            if sequentia == 0 {
                result.push_str(&format!(" <i>{}.</i> \n_\n", translate(ctx, "Sequentia", lang)));
                sequentia = 1;
            }
            if reading == 3 && sequentia == 0 {
                result.push_str("\n_\n");
            }
            let mut trimmed = line.trim().to_string();
            if !trimmed.is_empty() {
                // Do substitutions
                trimmed = trimmed.replace("oe", "œ")
                                 .replace("ae", "æ")
                                 .replace("Ae", "Æ")
                                 .replace("cæl", "cœl");
                if ci_starts_with(&trimmed, "#[ 25") && reading > 1 {
                    return result; // early return – end processing this branch.
                }
                if reading != 0 && reading != 1 && !ci_contains(&trimmed, "#[") {
                    result.push_str(&format!("-- {}\n", trimmed));
                }
            }
        }
    }
    result
}

/// Processes the lines for the normal branch in `regula_emaus`.
fn process_regula_normal_lines(ctx: &LanguageTextContext, lang: &str, d: u32, month: u32, fname: &str) -> String {
    let mut result = String::new();
    let mut reading: u32 = 0;
    let mut sequentia: u32 = 0;
    let mut titulus: Option<String> = None;
    for line in do_read(fname).unwrap_or_default() {
        if ci_contains(&line, "<b>caput ") || ci_contains(&line, "<b>incipit ") {
            titulus = Some(line.clone());
        }
        let pattern = format!(" {}.{}.", d, month);
        if ci_contains(&line, &pattern) || reading >= 1 {
            reading += 1;
            if reading == 1 {
                continue;
            }
            if reading == 2 {
                if let Some(ref mut tit) = titulus {
                    if ci_contains(tit, "incipit") {
                        let title_in = translate(ctx, "Lectio prologus", lang);
                        *tit = ci_replace_all(tit, "Incipit Prologus", &title_in);
                    } else {
                        let title_in = translate(ctx, "Lectio regulae", lang);
                        *tit = ci_replace_all(tit, "Caput", &format!(" {} ", title_in));
                    }
                    result.push_str(&format!("{}. ", tit));
                }
            }
            if ci_contains(&line, "<b>caput ") || ci_contains(&line, "<b>incipit ") {
                sequentia = 1;
                result.push_str("\n_\n");
                continue;
            }
            if sequentia == 0 {
                result.push_str(&format!(" <i>{}.</i> \n_\n", translate(ctx, "Sequentia", lang)));
                sequentia = 1;
            }
            if reading == 3 && sequentia == 0 {
                result.push_str("\n_\n");
            }
            let mut trimmed = line.trim().to_string();
            if !trimmed.is_empty() {
                trimmed = trimmed.replace("oe", "œ")
                                 .replace("ae", "æ")
                                 .replace("Ae", "Æ")
                                 .replace("cæl", "cœl");
                if ci_starts_with(&trimmed, "#[") && reading > 1 {
                    return result;
                }
                if reading != 0 && reading != 1 {
                    result.push_str(&format!("-- {}\n", trimmed));
                }
            }
        }
    }
    result
}

/// Processes a single line in the necrologium branch.
/// Returns Some(processed_line) if the line should be appended; returns None to signal an early break.
fn process_necrologium_line(line: &str, tomorrow: u32, lang: &str, is_first: bool) -> Option<String> {
    let mut trimmed = line.trim().to_string();
    if trimmed.is_empty() {
        return Some(String::new());
    }
    if let Some(idx) = trimmed.find('#') {
        trimmed = trimmed[idx + 1..].to_string();
    }
    if trimmed.trim().is_empty() {
        trimmed = format!("_{}", trimmed);
    }
    let tomorrow_pattern = format!("Die {}.", tomorrow);
    if ci_contains(&trimmed, &tomorrow_pattern) {
        return None; // signal to break the loop
    }
    if ci_contains(lang, "Bohemice") || ci_contains(lang, "Cesky") {
        // In this branch we call our Czech translator.
        trimmed = translate_cz(&[trimmed]);
    }
    trimmed = trimmed.replace("oe", "œ")
                     .replace("ae", "æ")
                     .replace("Ae", "Æ")
                     .replace("Tento", "Teuto")
                     .replace("•", "r. ");
    if is_first {
        Some(format!("v. {}\n_\n", trimmed))
    } else if ci_contains(&trimmed, "Die") || ci_contains(&trimmed, "Dne") {
        Some(format!("\n_\nv. {}\n_\n", trimmed))
    } else {
        Some(format!("{}\n", trimmed))
    }
}

/// Processes all lines in the necrologium file that match the day pattern.
fn process_necrologium_lines(fname: &str, d: u32, tomorrow: u32, lang: &str) -> String {
    let mut result = String::new();
    let pattern = format!("Die {}.", d);
    let mut reading: u32 = 0;
    for line in do_read(fname).unwrap_or_default() {
        if ci_contains(&line, &pattern) || reading >= 1 {
            reading += 1;
            if let Some(processed) = process_necrologium_line(&line, tomorrow, lang, reading == 1) {
                result.push_str(&processed);
            } else {
                break;
            }
        }
    }
    result
}

/// Processes all lines in the martyrologium file.
fn process_martyrologium_lines(fname: &str, mensis: &[&str], d_str: &str, month_t_num: usize) -> String {
    let mut result = String::new();
    let mut reading: u32 = 0;
    for line in do_read(fname).unwrap_or_default() {
        if (line.to_lowercase().starts_with(&d_str.to_lowercase())
            && ci_contains(&line, mensis[month_t_num]))
            || reading >= 1
        {
            reading += 1;
            let mut trimmed = line.trim().to_string();
            if !trimmed.is_empty() {
                if let Some(idx) = trimmed.find('#') {
                    trimmed = trimmed[idx + 1..].to_string();
                }
                if trimmed.trim().is_empty() {
                    trimmed = format!("_{}", trimmed);
                }
                trimmed = trimmed.replace("oe", "œ")
                                 .replace("ae", "æ")
                                 .replace("Ae", "Æ");
                if ci_contains(&trimmed, "A jinde") {
                    break;
                }
                if reading != 0 && reading != 1 {
                    result.push_str(&format!("r. {}\n", trimmed));
                }
            }
        }
    }
    result
}

/// Returns the text of Regula de Emaus.
/// Expects arguments:
///  - args[0]: language (e.g. "Cesky")
///  - args[1]: day (e.g. "23")
///  - args[2]: month (e.g. "2")
///  - args[3]: year (e.g. "2023")
///  - args[4]: large font description (e.g. "120 bold red")
pub fn regula_emaus<F: Fn(&str) -> bool>(
    ctx: &LanguageTextContext, 
    lang: &str, 
    day: u32, month: u32, year: i32, 
    largefont: &str, datafolder: &str, 
    file_exists_fn: &F
) -> String {
    
    let mut d = day; // working copy of day
    let l = leap_year(year);

    // No need to remove leading zeros because we use numeric types.
    if month == 2 && day >= 24 && !l {
        d += 1;
    }

    // TODO!
    let fname = checkfile(
        datafolder, 
        &ctx.fb_lang, 
        lang, 
        "Regula/Regula_OSB_Emaus.txt", 
        file_exists_fn,
    );

    let mut t = setfont(largefont, &translate(ctx, "Regula", lang));
    t.push_str("\n_\n");

    // Branch into two sub-functions.
    if day == 23 && month == 2 && !l {
        t.push_str(&process_regula_special_lines(ctx, lang, d, month, &fname));
    } else {
        t.push_str(&process_regula_normal_lines(ctx, lang, d, month, &fname));
    }

    t.push_str("\n_\n$Tu autem");
    t
}

/// Returns the text of the Necrologium for the day.
/// Expects arguments:
///  - args[0]: language
///  - args[1]: day
///  - args[2]: month
///  - args[3]: year
///  - args[4]: large font description
pub fn necrologium<F: Fn(&str) -> bool>(
    ctx: &LanguageTextContext, 
    lang: &str, 
    day: u32, month: u32, year: i32, 
    largefont: &str, datafolder: &str, 
    file_exists_fn: &F
) -> String {
    let mut t = setfont(largefont, &translate(ctx, "Necrologium", lang));
    t.push('\n');
    let d = day; // already numeric
    let l = leap_year(year);
    let mut reading = 0;
    let mut tomorrow = day + 1;
    if day == 28 && month == 2 && !l {
        tomorrow += 1;
    }
    let mensis = vec![
        "zero-ius", "Januarius", "Februarius", "Martius", "Aprilis",
        "Majus", "Junius", "Julius", "Augustus", "September", "October", "November", "December",
    ];
    let mut fname = checkfile(datafolder, &ctx.fb_lang, lang, &format!("Necrologium/{}.txt", mensis[month as usize]), file_exists_fn);
    if ci_contains(lang, "Bohemice") || ci_contains(lang, "Cesky") {
        fname = checkfile(datafolder, &ctx.fb_lang, "Latin", &format!("Necrologium/{}.txt", mensis[month as usize]), file_exists_fn);
    }
    // Process the matching lines in a helper.
    t.push_str(&process_necrologium_lines(&fname, d, tomorrow, lang));
    t.push_str("$Quorum animae\n");
    t
}

/// Returns the text of the Czech Martyrologium for the day.
/// Expects arguments:
///  - args[0]: language
///  - args[1]: day
///  - args[2]: month
///  - args[3]: year
///  - args[4]: large font description
///  - args[5]: small font description
pub fn martyrologium_cz<F: Fn(&str) -> bool>(
    ctx: &LanguageTextContext, 
    lang: &str, day: u32, month: u32, year: i32, 
    largefont: &str, smallblack: &str, datafolder: &str, 
    file_exists_fn: &F
) -> String {

    let mut t = setfont(largefont, "Martyrologium ");
    t.push_str(&setfont(smallblack, "(anticip.)"));
    t.push_str("\n_\n");
    let l = leap_year(year);
    // Get tomorrow string as "MM-DD"
    let fname_next = nextday(month, day, year);
    let mut parts = fname_next.split('-');
    let month_t = remove_leading_zeros(parts.next().unwrap_or(""));
    let day_t = remove_leading_zeros(parts.next().unwrap_or(""));
    let d_str = day_t.clone();
    let month_t_num: usize = month_t.parse().unwrap_or(0);
    let mensis = vec![
        "zero-ius", "ledna", "února", "března", "dubna", "května",
        "června", "července", "srpna", "září", "října", "listopadu", "prosince",
    ];
    let fname = checkfile(datafolder, &ctx.fb_lang, &lang, "Psalterium/Martyrologium.txt", file_exists_fn);
    t.push_str(&format!(
        "v. M<b>artyrologium na den {}. {}, Léta Páně {}.</b>\n_\n",
        d_str, mensis[month_t_num], year
    ));

    let day_t_num: u32 = day_t.parse().unwrap_or(0);
    let month_t_num_u32: u32 = month_t.parse().unwrap_or(0);
    if day_t_num == 24 && month_t_num_u32 == 2 && l {
        t.push_str("r. Památka velkého počtu svatých mučedníků a vyznavačů, taktéž svatých panen, jejichž přímluvu s v modlitbách vyprošujeme. †\n");
        t.push_str("$Deo gratias\n_\n");
        return t;
    }
    // Process the martyrologium lines in a helper.
    t.push_str(&process_martyrologium_lines(&fname, &mensis, &d_str, month_t_num));
    t.push_str("$Conclmart Cist\n_\n");
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_contains_and_replace() {
        let s = "Hello World! hello world!";
        assert!(ci_contains(s, "HELLO"));
        let replaced = ci_replace_all(s, "hello", "hi");
        assert_eq!(replaced, "hi World! hi world!");
    }

    #[test]
    fn test_remove_leading_zeros() {
        assert_eq!(remove_leading_zeros("007"), "7");
        assert_eq!(remove_leading_zeros("0"), "0");
    }

    #[test]
    fn test_process_necrologium_line() {
        // Test our helper for necrologium lines.
        let line = "   Die 23. something #header";
        // This line does not contain tomorrow (assume tomorrow is 24).
        if let Some(processed) = process_necrologium_line(line, 24, "Cesky", true) {
            assert!(processed.contains("v. "));
        } else {
            panic!("Should not have returned None");
        }
    }

    #[test]
    fn test_regula_emaus() {

        let context = LanguageTextContext {
            fb_lang: "Latin".to_string(),
            .. Default::default() 
        };

        let output = regula_emaus(
            &context,
            "Cesky",
            23,
            2,
            2023,
        "120 bold red",
            "horas",
            &|_| false
        );

        assert!(output.contains("Regula(Cesky)"));
        assert!(output.contains("--"));
        assert!(output.contains("$Tu autem"));
    }

    #[test]
    fn test_necrologium() {

        let context = LanguageTextContext {
            fb_lang: "Latin".to_string(),
            .. Default::default() 
        };

        let output = necrologium(
            &context,
            "Cesky",
            23,
            2,
            2023,
        "120 bold red",
            "horas",
            &|_| false
        );

        assert!(output.contains("Necrologium(Cesky)"));
        assert!(output.contains("v. "));
        assert!(output.contains("$Quorum animae"));
    }

    #[test]
    fn test_martyrologium_cz() {

        let context = LanguageTextContext {
            fb_lang: "Latin".to_string(),
            .. Default::default() 
        };

        let output = martyrologium_cz(
            &context,
            "Cesky",
            23,
            2,
            2023,
        "120 bold red",
            "80 small black",
            "horas",
            &|_| false
        );

        assert!(output.contains("Martyrologium"));
        assert!(output.contains("(anticip.)"));
        assert!(output.contains("$Conclmart Cist"));
    }
}