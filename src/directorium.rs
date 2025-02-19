//! directorium.rs
//!
//! This module is a Rust translation of the Divinum Officium Directorium.pm module.
//!
//! It provides functions for obtaining liturgical data from cached data files.
//!
//! Internally, the module loads data from a file (DATA_FOLDER/data.txt) into a global
//! cache (representing version‐specific data) and uses additional caches for other files.
//!
//! **Caveat:** Global mutable caches are used here (via Mutex and once_cell) to mimic the
//! Perl behavior. In a larger Rust project, a more modular design may be preferable.

use crate::date;
use crate::fileio;
use std::collections::HashMap;
use std::io;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// The base folder for data files, relative to the binary location.
const DATA_FOLDER: &str = "../../www/Tabulae";

/// Struct holding per‐version data loaded from data.txt.
#[derive(Debug, Clone)]
struct Data {
    kalendar: String,
    transfer: String,
    stransfer: String,
    base: String,
    tbase: String,
}

/// Global cache for version data.
/// Once loaded, keys are version names mapping to their Data.
static DATA: Lazy<Mutex<HashMap<String, Data>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Global cache for various computed data.
/// Keys are strings such as "loaded", "kalendar:<version>", "Transfer:<version>:<year>",
/// "Stransfer:<version>:<year>", "Tempora:<version>".
static DCACHE: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Loads the version data from DATA_FOLDER/data.txt into the global DATA cache.
/// The file is assumed to be a CSV with a header (which is skipped) and lines of the form:
///     version,kalendar,transfer,stransfer,base,tbase
fn load_data_data() -> io::Result<()> {
    let path = format!("{}/data.txt", DATA_FOLDER);
    let lines = fileio::do_read(&path)?;
    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Can't open {}", path),
        ));
    }
    let mut data_lock = DATA.lock().unwrap();
    // Skip the header line.
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 6 {
            continue;
        }
        let ver = parts[0].to_string();
        data_lock.insert(
            ver,
            Data {
                kalendar: parts[1].to_string(),
                transfer: parts[2].to_string(),
                stransfer: parts[3].to_string(),
                base: parts[4].to_string(),
                tbase: parts[5].to_string(),
            },
        );
    }
    // Mark that data is loaded by inserting a special key in DCACHE.
    DCACHE
        .lock()
        .unwrap()
        .insert("loaded".to_string(), HashMap::new());
    Ok(())
}

/// Checks whether a given cache key is present in DCACHE.
/// If the global DATA has not yet been loaded, it is loaded first.
fn is_cached(key: &str) -> bool {
    {
        let dcache = DCACHE.lock().unwrap();
        if dcache.contains_key("loaded") {
            // already loaded
        } else {
            drop(dcache);
            let _ = load_data_data();
        }
    }
    DCACHE.lock().unwrap().contains_key(key)
}

/// --- Helper functions replacing regexes --- ///

/// Remove an optional prefix "Hy" or "seant" from the beginning of a string.
fn remove_optional_prefix(s: &str) -> &str {
    if s.starts_with("Hy") {
        &s[2..]
    } else if s.starts_with("seant") {
        &s[5..]
    } else {
        s
    }
}

/// Returns true if the line matches the pattern equivalent to:
///   ^(?:Hy|seant)?(?:01|02-[01]|02-2[01239]|dirge1)
fn matches_pattern1(line: &str) -> bool {
    let s = remove_optional_prefix(line);
    if s.starts_with("01") {
        return true;
    }
    if s.starts_with("02-0") || s.starts_with("02-1") {
        return true;
    }
    if s.starts_with("02-2") {
        // "02-2" is 4 characters; check the next character if present
        if let Some(c) = s.chars().nth(4) {
            if "01239".contains(c) {
                return true;
            }
        }
    }
    if s.starts_with("dirge1") {
        return true;
    }
    false
}

/// Returns true if the line matches the pattern equivalent to:
///   ^(?:Hy|seant)?(?:01|02-[01]|02-2[01239]|.*=(01|02-[01]|02-2[0123])|dirge1)
fn matches_pattern2(line: &str) -> bool {
    let s = remove_optional_prefix(line);
    if s.starts_with("01") || s.starts_with("02-0") || s.starts_with("02-1") {
        return true;
    }
    if s.starts_with("02-2") {
        if let Some(c) = s.chars().nth(4) {
            if "01239".contains(c) {
                return true;
            }
        }
    }
    if s.starts_with("dirge1") {
        return true;
    }
    // Check for alternative: any text, then '=' then one of (01|02-[01]|02-2[0123])
    if let Some(eq_pos) = s.find('=') {
        let after = &s[eq_pos + 1..];
        if after.starts_with("01") || after.starts_with("02-0") || after.starts_with("02-1") {
            return true;
        }
        if after.starts_with("02-2") {
            if let Some(c) = after.chars().nth(4) {
                if "0123".contains(c) {
                    return true;
                }
            }
        }
    }
    false
}

/// Reads a transfer file from DATA_FOLDER/{type}/{name}.txt and filters its lines according to `filter`.
///
/// - If `filter == 1`: returns lines that do **not** match pattern2.
/// - If `filter == 2`: returns lines that **do** match pattern1.
/// - Otherwise, returns all lines.
fn load_transfer_file(name: &str, filter: i32, type_: &str) -> io::Result<Vec<String>> {
    let path = format!("{}/{}/{}.txt", DATA_FOLDER, type_, name);
    let lines = fileio::do_read(&path)?;
    let filtered = if filter == 1 {
        lines.into_iter().filter(|line| !matches_pattern2(line)).collect()
    } else if filter == 2 {
        lines.into_iter().filter(|line| matches_pattern1(line)).collect()
    } else {
        lines
    };
    Ok(filtered)
}

/// Loads the kalendar file for a given version into the global cache.
/// The file is located at: DATA_FOLDER/Kalendaria/{kalendar}.txt,
/// where {kalendar} is obtained from the version’s Data.
fn load_kalendar(version: &str) -> io::Result<()> {
    let data_lock = DATA.lock().unwrap();
    let ver_data = data_lock.get(version).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Unknown version {}", version),
        )
    }).cloned()?;
    drop(data_lock);
    let cache_key = format!("kalendar:{}", version);
    let path = format!("{}/Kalendaria/{}.txt", DATA_FOLDER, ver_data.kalendar);
    let lines = fileio::do_read(&path)?;
    let mut map = HashMap::new();
    for line in lines.into_iter().filter(|l| l.contains('=')) {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
        }
    }
    DCACHE.lock().unwrap().insert(cache_key, map);
    Ok(())
}

/// Loads the tempora file for a given version into the global cache.
/// Reads from: DATA_FOLDER/Tempora/{transfer}.txt, where {transfer} is from Data.
fn load_tempora(version: &str) -> io::Result<()> {
    let data_lock = DATA.lock().unwrap();
    let ver_data = data_lock.get(version).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Unknown version {}", version),
        )
    }).cloned()?;
    drop(data_lock);
    let cache_key = format!("Tempora:{}", version);
    let lines = load_transfer_file(&ver_data.transfer, 0, "Tempora")?;
    let mut map = HashMap::new();
    for line in lines {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            let val = parts[1]
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            map.insert(parts[0].trim().to_string(), val);
        }
    }
    DCACHE.lock().unwrap().insert(cache_key, map);
    Ok(())
}

/// Loads a transfer table (for Transfer or Stransfer) for a given year and version.
/// If `stransferf` is Some(_), then loads Stransfer; otherwise, loads Transfer.
fn load_transfer(
    year: i32,
    version: &str,
    stransferf: Option<&str>,
) -> io::Result<HashMap<String, String>> {
    let data_lock = DATA.lock().unwrap();
    let ver_data = data_lock.get(version).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Unknown version {}", version),
        )
    }).cloned()?;
    drop(data_lock);
    let type_str = if stransferf.is_some() {
        "Stransfer"
    } else {
        "Transfer"
    };
    let cache_key = format!("{}:{}:{}", type_str, version, year);
    {
        let dcache = DCACHE.lock().unwrap();
        if let Some(map) = dcache.get(&cache_key) {
            return Ok(map.clone());
        }
    }
    let isleap = date::leap_year(year);
    let (e_day, e_month, _) = date::geteaster(year);
    let e_value = e_month * 100 + e_day; // as in Perl: month*100 + day
    let letter_index = (e_value as i32 - 319 + if e_month == 4 { 1 } else { 0 }) % 7;
    let letters = ["a", "b", "c", "d", "e", "f", "g"];
    let mut lines = load_transfer_file(letters[letter_index as usize], if isleap { 1 } else { 0 }, type_str)?;
    lines.extend(load_transfer_file(&e_value.to_string(), if isleap { 0 } else { 0 }, type_str)?);
    if isleap {
        let mut e_adj = e_value + 1;
        if e_adj == 332 {
            e_adj = 401;
        }
        let letter2_index = ((letter_index - 6).rem_euclid(7)) as usize;
        lines.extend(load_transfer_file(letters[letter2_index], 2, type_str)?);
        lines.extend(load_transfer_file(&e_adj.to_string(), 2, type_str)?);
    }
    let mut transfer_map = HashMap::new();
    for line in lines {
        let parts: Vec<&str> = line.split(";;").collect();
        if parts.is_empty() {
            continue;
        }
        let entry = parts[0];
        let ver_pattern = parts.get(1).map(|s| s.trim());
        if ver_pattern.is_none() || ver_pattern.unwrap().is_empty() || ver_data.transfer.contains(ver_pattern.unwrap()) {
            let kv: Vec<&str> = entry.splitn(2, '=').collect();
            if kv.len() == 2 {
                transfer_map.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }
    }
    DCACHE
        .lock()
        .unwrap()
        .insert(cache_key.clone(), transfer_map.clone());
    Ok(transfer_map)
}

/// Returns the kalendar filename for a given version and day.
/// If not found, and if the version’s Data contains a fallback in `base`,
/// recursively looks it up.
pub fn get_kalendar(version: &str, day: &str) -> Option<String> {
    let cache_key = format!("kalendar:{}", version);
    if !is_cached(&cache_key) {
        let _ = load_kalendar(version);
    }
    let dcache = DCACHE.lock().unwrap();
    if let Some(map) = dcache.get(&cache_key) {
        if let Some(val) = map.get(day) {
            return Some(val.clone());
        }
    }
    let data_lock = DATA.lock().unwrap();
    if let Some(ver_data) = data_lock.get(version) {
        if !ver_data.base.is_empty() {
            return get_kalendar(&ver_data.base, day);
        }
    }
    None
}

/// Returns the transfer table value for a given key, year, and version (for Transfer).
/// If not found, attempts to fallback using the version’s `tbase` field.
pub fn get_transfer(year: i32, version: &str, key: &str) -> Option<String> {
    let cache_key = format!("Transfer:{}:{}", version, year);
    if !is_cached(&cache_key) {
        if let Ok(_) = load_transfer(year, version, None) {
            // loaded
        }
    }
    {
        let dcache = DCACHE.lock().unwrap();
        if let Some(map) = dcache.get(&cache_key) {
            if let Some(val) = map.get(key) {
                return Some(val.clone());
            }
        }
    }
    let data_lock = DATA.lock().unwrap();
    if let Some(ver_data) = data_lock.get(version) {
        if !ver_data.tbase.is_empty() {
            return get_transfer(year, &ver_data.tbase, key);
        }
    }
    None
}

/// Returns the stransfer table value for a given key, year, and version (for Stransfer).
/// Follows similar fallback logic to get_transfer.
pub fn get_stransfer(year: i32, version: &str, key: &str) -> Option<String> {
    let cache_key = format!("Stransfer:{}:{}", version, year);
    if !is_cached(&cache_key) {
        if let Ok(_) = load_transfer(year, version, Some("Stransfer")) {
            // loaded
        }
    }
    {
        let dcache = DCACHE.lock().unwrap();
        if let Some(map) = dcache.get(&cache_key) {
            if let Some(val) = map.get(key) {
                return Some(val.clone());
            }
        }
    }
    let data_lock = DATA.lock().unwrap();
    if let Some(ver_data) = data_lock.get(version) {
        if !ver_data.tbase.is_empty() {
            return get_stransfer(year, &ver_data.tbase, key);
        }
    }
    None
}

/// Returns the tempora table value for a given key and version.
/// If not found, uses fallback via tbase.
pub fn get_tempora(version: &str, key: &str) -> Option<String> {
    let cache_key = format!("Tempora:{}", version);
    if !is_cached(&cache_key) {
        let _ = load_tempora(version);
    }
    {
        let dcache = DCACHE.lock().unwrap();
        if let Some(map) = dcache.get(&cache_key) {
            if let Some(val) = map.get(key) {
                return Some(val.clone());
            }
        }
    }
    let data_lock = DATA.lock().unwrap();
    if let Some(ver_data) = data_lock.get(version) {
        if !ver_data.tbase.is_empty() {
            return get_tempora(&ver_data.tbase, key);
        }
    }
    None
}

/// Checks whether a given saint or season is transferred.
/// Returns Some(destination) if transferred, or None otherwise.
pub fn transfered(s: &str, year: i32, version: &str) -> Option<String> {
    let s = s.replacen("SanctiM/", "", 1).replacen("Sancti/", "", 1);
    if s.trim().is_empty() {
        return None;
    }
    let transfer_key = format!("Transfer:{}:{}", version, year);
    let temp_key = format!("Tempora:{}", version);
    let dcache = DCACHE.lock().unwrap();
    let transfer_map = dcache.get(&transfer_key).cloned();
    if let Some(ref tm) = transfer_map {
        for (key, val) in tm.iter() {
            if key.to_lowercase().contains("dirge") || key.to_lowercase().contains("hy") {
                continue;
            }
            if !val.is_empty() {
                if !val.starts_with(key)
                    && (s.to_lowercase().contains(&val.to_lowercase())
                        || val.to_lowercase().contains(&s.to_lowercase()))
                    && !tm.get(key).map_or(false, |v| v.trim_end().ends_with("v"))
                {
                    return Some(key.clone());
                }
            }
        }
    }
    let temp_map = dcache.get(&temp_key).cloned();
    if let Some(ref temp_map) = temp_map {
        for (key, val) in temp_map.iter() {
            if key.to_lowercase().contains("dirge") {
                continue;
            }
            if val.to_lowercase().contains(&s.to_lowercase()) {
                if let Some(t_val) = transfer_map.as_ref().and_then(|m| m.get(key)) {
                    if !t_val.trim_end().ends_with("v") {
                        return Some(key.clone());
                    }
                }
            }
        }
    }
    None
}

/// Checks for coronatio: if day==20 and month==3, returns "Votive/Coronatio", else None.
pub fn check_coronatio(day: u32, month: u32) -> Option<String> {
    if day == 20 && month == 3 {
        Some("Votive/Coronatio".to_string())
    } else {
        None
    }
}

/// Determines whether a “dirge” (defunctorum) should be said after a given hour.
/// Returns true if the current hour (or the next day’s for Vespera) matches the transfer rule.
pub fn dirge(version: &str, hora: &str, day: u32, month: u32, year: i32) -> bool {
    if !(hora.contains("Vespera") || hora.contains("Laudes")) {
        return false;
    }
    let sday = if hora.contains("Laudes") {
        date::get_sday(month, day, year)
    } else {
        date::nextday(month, day, year)
    };
    let part1 = get_transfer(year, version, "dirge1").unwrap_or_else(|| "".to_string());
    let part2 = get_transfer(year, version, "dirge2").unwrap_or_else(|| "".to_string());
    let dirgeline = format!("{} {}", part1, part2);
    dirgeline.contains(&sday)
}

/// Determines whether the Matutinum Hymn should be merged with Vesperas.
pub fn hymnmerge(version: &str, day: u32, month: u32, year: i32) -> bool {
    let key = format!("Hy{}", date::get_sday(month, day, year));
    get_transfer(year, version, &key).map_or(false, |v| v == "1")
}

/// Determines whether the Hymns should be shifted according to the transfer table.
pub fn hymnshift(version: &str, day: u32, month: u32, year: i32) -> bool {
    let key = format!("Hy{}", date::get_sday(month, day, year));
    get_transfer(year, version, &key).map_or(false, |v| v == "2")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // --- Tests for the helper functions (mimicking the original regex behavior) --- //

    #[test]
    fn test_matches_pattern1() {
        // These examples should match the pattern:
        assert!(matches_pattern1("Hy01Test"));
        assert!(matches_pattern1("seant02-1Example"));
        assert!(matches_pattern1("02-29Data")); // because "02-2" + '9'
        assert!(matches_pattern1("dirge1Something"));
        // These should not match:
        assert!(!matches_pattern1("03-01Data"));
        assert!(!matches_pattern1("SomeOtherText"));
    }

    #[test]
    fn test_matches_pattern2() {
        // Examples matching one of the first alternatives:
        assert!(matches_pattern2("Hy01Test"));
        assert!(matches_pattern2("seant02-1Example"));
        assert!(matches_pattern2("02-29Data"));
        assert!(matches_pattern2("dirge1Something"));
        // Test the alternative with '=':
        assert!(matches_pattern2("anything=01rest"));
        assert!(matches_pattern2("prefix=02-0more"));
        assert!(matches_pattern2("prefix=02-1more"));
        assert!(matches_pattern2("prefix=02-20more"));
        assert!(matches_pattern2("prefix=02-22more"));
        // Should not match if after '=' the pattern is not one of the allowed:
        assert!(!matches_pattern2("prefix=03-01more"));
    }

    // --- Tests for public functions --- //

    #[test]
    fn test_check_coronatio() {
        assert_eq!(check_coronatio(20, 3), Some("Votive/Coronatio".to_string()));
        assert_eq!(check_coronatio(19, 3), None);
        assert_eq!(check_coronatio(20, 4), None);
    }

    #[test]
    fn test_get_kalendar_with_cache() {
        // Simulate a version and preloaded kalendar data.
        let version = "test_version";
        {
            let mut data_lock = DATA.lock().unwrap();
            data_lock.insert(version.to_string(), Data {
                kalendar: "dummy_kal".to_string(),
                transfer: "".to_string(),
                stransfer: "".to_string(),
                base: "".to_string(),
                tbase: "".to_string(),
            });
        }
        // Insert a dummy kalendar cache for this version.
        let cache_key = format!("kalendar:{}", version);
        let mut kal_map = HashMap::new();
        kal_map.insert("Monday".to_string(), "file_monday.txt".to_string());
        {
            let mut dcache = DCACHE.lock().unwrap();
            dcache.insert(cache_key.clone(), kal_map);
        }
        assert_eq!(get_kalendar(version, "Monday"), Some("file_monday.txt".to_string()));
        assert_eq!(get_kalendar(version, "Tuesday"), None);
    }

    #[test]
    fn test_get_transfer_with_cache() {
        // Set up dummy data in DATA and DCACHE for a transfer table.
        let version = "test_version";
        let year = 2025;
        {
            let mut data_lock = DATA.lock().unwrap();
            data_lock.insert(version.to_string(), Data {
                kalendar: "".to_string(),
                transfer: "dummy_transfer".to_string(),
                stransfer: "".to_string(),
                base: "".to_string(),
                tbase: "".to_string(),
            });
        }
        let cache_key = format!("Transfer:{}:{}", version, year);
        let mut transfer_map = HashMap::new();
        transfer_map.insert("key1".to_string(), "value1".to_string());
        {
            let mut dcache = DCACHE.lock().unwrap();
            dcache.insert(cache_key.clone(), transfer_map);
        }
        assert_eq!(get_transfer(year, version, "key1"), Some("value1".to_string()));
        assert_eq!(get_transfer(year, version, "nonexistent"), None);
    }

    #[test]
    fn test_get_tempora_with_cache() {
        let version = "test_version";
        {
            let mut data_lock = DATA.lock().unwrap();
            data_lock.insert(version.to_string(), Data {
                kalendar: "".to_string(),
                transfer: "".to_string(),
                stransfer: "".to_string(),
                base: "".to_string(),
                tbase: "".to_string(),
            });
        }
        let cache_key = format!("Tempora:{}", version);
        let mut tempora_map = HashMap::new();
        tempora_map.insert("temp_key".to_string(), "temp_value".to_string());
        {
            let mut dcache = DCACHE.lock().unwrap();
            dcache.insert(cache_key.clone(), tempora_map);
        }
        assert_eq!(get_tempora(version, "temp_key"), Some("temp_value".to_string()));
        assert_eq!(get_tempora(version, "nonexistent"), None);
    }

    #[test]
    fn test_transfered() {
        // Set up dummy transfer table and tempora table in DCACHE.
        let version = "test_version";
        let year = 2025;
        let transfer_key = format!("Transfer:{}:{}", version, year);
        let mut transfer_map = HashMap::new();
        // Insert a key (not containing "dirge" or "hy") with a value that does not end with 'v'
        transfer_map.insert("dest".to_string(), "saint".to_string());
        {
            let mut dcache = DCACHE.lock().unwrap();
            dcache.insert(transfer_key.clone(), transfer_map);
            dcache.insert(format!("Tempora:{}", version), HashMap::new());
        }
        // "Sancti/Saint" should trigger a match.
        assert_eq!(transfered("Sancti/Saint", year, version), Some("dest".to_string()));
        // A non‐matching string returns None.
        assert_eq!(transfered("Sancti/Unknown", year, version), None);
    }

    #[test]
    fn test_dirge() {
        // For dirge, simulate get_transfer values for "dirge1" and "dirge2".
        let version = "test_version";
        let year = 2025;
        let cache_key = format!("Transfer:{}:{}", version, year);
        let mut transfer_map = HashMap::new();
        transfer_map.insert("dirge1".to_string(), "ABC".to_string());
        transfer_map.insert("dirge2".to_string(), "DEF".to_string());
        {
            let mut dcache = DCACHE.lock().unwrap();
            dcache.insert(cache_key.clone(), transfer_map);
        }
        // We assume that date::get_sday and nextday return a string that might match.
        // Here we simply check that the function returns a boolean.
        let result = dirge(version, "Laudes", 1, 1, year);
        assert!(result == true || result == false);
    }

    #[test]
    fn test_hymnmerge_and_hymnshift() {
        let version = "test_version";
        let year = 2025;
        // Assume that date::get_sday returns "X" for the given parameters.
        let sday = "X".to_string();
        let key = format!("Hy{}", sday);
        let cache_key = format!("Transfer:{}:{}", version, year);
        let mut transfer_map = HashMap::new();
        transfer_map.insert(key.clone(), "1".to_string());
        {
            let mut dcache = DCACHE.lock().unwrap();
            dcache.insert(cache_key.clone(), transfer_map);
        }
        // hymnmerge should be true if the transfer value equals "1".
        let merge = hymnmerge(version, 1, 1, year);
        assert_eq!(merge, true);
        // Now change the transfer value to "2" for hymnshift.
        {
            let mut dcache = DCACHE.lock().unwrap();
            if let Some(map) = dcache.get_mut(&cache_key) {
                map.insert(key.clone(), "2".to_string());
            }
        }
        let shift = hymnshift(version, 1, 1, year);
        assert_eq!(shift, true);
    }
}

