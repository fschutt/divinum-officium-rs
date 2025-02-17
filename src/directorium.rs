//! directorium.rs
//!
//! This module is a Rust translation of the Divinum Officium Directorium.pm module.
//!
//! It provides functions for obtaining liturgical data from cached data files.
//!
//! Exported functions include:
//! - `get_kalendar`
//! - `get_transfer`
//! - `get_stransfer`
//! - `get_tempora`
//! - `transfered`
//! - `check_coronatio`
//! - `dirge`
//! - `hymnmerge`
//! - `hymnshift`
//!
//! Internally, the module loads data from a file (DATA_FOLDER/data.txt) into a global
//! cache (representing version‐specific data) and uses additional caches for other files.
//!
//! **Caveat:** Global mutable caches are used here (via Mutex and once_cell) to mimic the
//! Perl behavior. In a larger Rust project, a more modular design may be preferable.

use crate::date;
use crate::fileio;
use regex::Regex;
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

/// Reads a transfer file from DATA_FOLDER/{type}/{name}.txt and filters its lines according to `filter`.
///
/// - If `filter == 1`: returns lines that do **not** match regexp2.
/// - If `filter == 2`: returns lines that **do** match regexp.
/// - Otherwise, returns all lines.
fn load_transfer_file(name: &str, filter: i32, type_: &str) -> io::Result<Vec<String>> {
    let path = format!("{}/{}/{}.txt", DATA_FOLDER, type_, name);
    let lines = fileio::do_read(&path)?;
    let re1 = Regex::new(r"^(?:Hy|seant)?(?:01|02-[01]|02-2[01239]|dirge1)").unwrap();
    let re2 = Regex::new(r"^(?:Hy|seant)?(?:01|02-[01]|02-2[01239]|.*=(01|02-[01]|02-2[0123])|dirge1)").unwrap();
    let filtered = if filter == 1 {
        lines
            .into_iter()
            .filter(|line| !re2.is_match(line))
            .collect()
    } else if filter == 2 {
        lines
            .into_iter()
            .filter(|line| re1.is_match(line))
            .collect()
    } else {
        // no filtering
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
            // Use substring of the value up to the first ';'
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
    // Not cached yet—compute it.
    let isleap = date::leap_year(year);
    let (e_day, e_month, _) = date::geteaster(year);
    let e_value = e_month * 100 + e_day; // as in Perl: month*100 + day
    let letter_index = (e_value as i32 - 319 + if e_month == 4 { 1 } else { 0 }) % 7;
    let letters = ["a", "b", "c", "d", "e", "f", "g"];
    let mut lines = load_transfer_file(letters[letter_index as usize], if isleap { 1 } else { 0 }, type_str)?;
    // Also add lines from the file named by e_value.
    lines.extend(load_transfer_file(&e_value.to_string(), if isleap { 0 } else { 0 }, type_str)?);
    if isleap {
        let mut e_adj = e_value + 1;
        if e_adj == 332 {
            e_adj = 401;
        }
        // Note: in Perl, letter index with offset -6.
        let letter2_index = ((letter_index - 6).rem_euclid(7)) as usize;
        lines.extend(load_transfer_file(letters[letter2_index], 2, type_str)?);
        lines.extend(load_transfer_file(&e_adj.to_string(), 2, type_str)?);
    }
    let mut transfer_map = HashMap::new();
    for line in lines {
        // Split on ";;" to get (the line, and an optional version pattern)
        let parts: Vec<&str> = line.split(";;").collect();
        if parts.is_empty() {
            continue;
        }
        let entry = parts[0];
        let ver_pattern = parts.get(1).map(|s| s.trim());
        // If no version pattern or if the version pattern matches ver_data.transfer, we accept this entry.
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
    // Fallback: use the base version if available.
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
    // Fallback using tbase
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
    // Fallback using tbase
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
    // Fallback using tbase
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
///
/// The function first removes any "Sancti" or "Sanctim" prefix from `s`,
/// then looks for a matching key in the Transfer table (and then the Tempora table)
/// where the value (if present) does not end with a trailing "v".
pub fn transfered(s: &str, year: i32, version: &str) -> Option<String> {
    // Remove "Sancti" or "Sanctim" prefix.
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
                    && !tm.get(key).map_or(false, |v| Regex::new(r"v\s*$").unwrap().is_match(v))
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
                    if !Regex::new(r"v\s*$").unwrap().is_match(t_val) {
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
/// - `hora` should match either "Vespera" or "Laudes"; otherwise, returns false.
pub fn dirge(version: &str, hora: &str, day: u32, month: u32, year: i32) -> bool {
    let re = Regex::new(r"(?i)Vespera|Laudes").unwrap();
    if !re.is_match(hora) {
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
    Regex::new(&regex::escape(&sday))
        .unwrap()
        .is_match(&dirgeline)
}

/// Determines whether the Matutinum Hymn should be merged with Vesperas.
/// Returns true if the transfer table value for the key "Hy<SanctiDay>" equals "1".
pub fn hymnmerge(version: &str, day: u32, month: u32, year: i32) -> bool {
    let key = format!("Hy{}", date::get_sday(month, day, year));
    get_transfer(year, version, &key).map_or(false, |v| v == "1")
}

/// Determines whether the Hymns should be shifted according to the transfer table.
/// Returns true if the transfer table value for "Hy<SanctiDay>" equals "2".
pub fn hymnshift(version: &str, day: u32, month: u32, year: i32) -> bool {
    let key = format!("Hy{}", date::get_sday(month, day, year));
    get_transfer(year, version, &key).map_or(false, |v| v == "2")
}
