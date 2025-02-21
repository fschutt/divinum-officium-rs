//! scripting.rs
//!
//! This module corresponds to `Scripting.pm` from Divinum Officium. It provides
//! a simple mechanism for defining, registering, and dispatching “script
//! functions” which can be called by name at runtime, with arguments parsed
//! from a string. In the original Perl, the module uses `Attribute::Handlers`
//! to automatically register functions decorated with `: ScriptFunc(func_name)`
//! or `: ScriptShortFunc(func_name)`, storing them in a global `%script_functions`
//! map. Then, calls to `dispatch_script_function("func_name", @args)` are routed
//! to the appropriate subroutine.
//!
//! # Overview
//!
//! - **`register_script_function(function_name, code, short: bool)`**: Stores a
//!   function closure in a global map. In Perl, this is called automatically
//!   when the function is declared with an attribute. In Rust, we call it
//!   manually or via macros.
//!
//! - **`dispatch_script_function(function_name, &args)`**: Looks up the function
//!   in the registry by name and calls it with the provided arguments. If the
//!   function does not exist, or only the “short” variant is present, we raise
//!   an error. If successful, returns whatever the function closure returns.
//!
//! - **`parse_script_arguments(list_str)`**: Splits a string into arguments
//!   based on commas that are not inside single quotes. Each argument can be
//!   either a numeric literal or a single-quoted string. This replicates the
//!   minimal argument parsing from `Scripting.pm`.
//!
//! Because Rust does not have `Attribute::Handlers` the same way Perl does,
//! the attribute-based logic (`sub UNIVERSAL::ScriptFunc : ATTR(CODE,BEGIN) {...}`)
//! is omitted. Instead, you can define your script functions and register
//! them with `register_script_function(...)` in your initialization code.
//!
//! Example usage or notes:
//!
//! ```no_run,ignore
//! fn initialize_functions() {
//!    // register a "psalm" function
//!    register_script_function(
//!        "psalm",
//!         Box::new(|args: &[String]| {
//!            // logic for psalm
//!            "some psalm text".to_string()
//!        }),
//!        false, // not short
//!    );
//! }
//! ```
//! 
//! Then calls to `dispatch_script_function("psalm", &["117".to_string()])` => returns "some psalm text".

use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::fmt;

/// A type alias for the function signature. In Perl, subroutines can have
/// variable arguments. In Rust, we unify them into `Vec<String>`. The script
/// function can return a `String`, though some might prefer `String` or an
/// enum for more complex usage.
pub type ScriptFunc = fn(&[String]) -> String;

/// A global registry of script function names to two possible handlers:
/// - `"func"`: The normal or “long” form
/// - `"shortfunc"`: The short form, if any
///
/// In Perl, it was `%script_functions{$function_name}{func} = code_ref`.
static SCRIPT_FUNCTIONS: Lazy<std::sync::Mutex<HashMap<String, HashMap<&'static str, ScriptFunc>>>> =
    Lazy::new(|| std::sync::Mutex::new(HashMap::new()));

/// An internal structure to store “deferred” functions. In Perl, this was
/// needed for older versions that triggered attribute handlers before the
/// sub was in the symbol table. In Rust, we typically do not replicate
/// that. We provide this struct only to mirror the original logic. You could
/// omit it if not needed.
#[derive(Clone)]
struct DeferredFunction {
    package: String,
    code: ScriptFunc,
    params: HashMap<&'static str, bool>,
}

impl fmt::Debug for DeferredFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DeferredFunction")
        .field("package", &self.package)
        .field("code", &"<fn>")
        .field("params", &self.params)
        .finish()
    }
}

/// In Perl, `@deferred_functions`. We rarely need that in Rust, but we include
/// it for completeness.
static DEFERRED_FUNCTIONS: Lazy<std::sync::Mutex<Vec<DeferredFunction>>> =
    Lazy::new(|| std::sync::Mutex::new(Vec::new()));

/// Registers a new script function, as in
/// `register_script_function($function_name, $code_ref, short => boolean)`.
///
/// # Arguments
/// * `function_name` - Name used to dispatch this function at runtime.
/// * `code` - A closure that takes a slice of `String` arguments and returns a `String`.
/// * `is_short` - If `true`, registers the function under `"shortfunc"`. Otherwise, `"func"`.
pub fn register_script_function(
    function_name: &str,
    code: ScriptFunc,
    is_short: bool,
) {
    let mut map = SCRIPT_FUNCTIONS.lock().unwrap();
    let entry = map
        .entry(function_name.to_string())
        .or_insert_with(HashMap::new);
    let slot = if is_short { "shortfunc" } else { "func" };
    entry.insert(slot, code);
}

/// This function in Perl was `register_deferred_functions`, which tries to
/// attach any leftover subroutines if we can. In Rust, this is typically
/// unnecessary. We provide it to replicate structure.
pub fn register_deferred_functions() -> usize {
    let mut defers = DEFERRED_FUNCTIONS.lock().unwrap();
    let mut still_deferred = Vec::new();
    let mut count = 0;

    for d in defers.iter() {
        // The original code tries to find the GLOB for the sub. In Rust,
        // there's no direct equivalent. We'll assume we can always register.
        // If we cannot, we'd push it back into still_deferred.
        let can_register = true;
        if can_register {
            register_script_function(&d.package, d.code.clone(), d.params.get("short").copied().unwrap_or(false));
            count += 1;
        } else {
            still_deferred.push(d.clone());
        }
    }
    // replace defers
    *defers = still_deferred;
    count
}

/// Dispatch a registered script function by name, passing `args`.
///
/// # Panics
///
/// * If the function doesn't exist in the registry (and we cannot salvage it by
///   calling `register_deferred_functions()`).
/// * If there's no "func" (long form) handler for that name.
pub fn dispatch_script_function(function_name: &str, args: &[String]) -> String {
    {
        let map = SCRIPT_FUNCTIONS.lock().unwrap();
        if !map.contains_key(function_name) {
            // Attempt to handle deferred
            drop(map); // release lock
            if register_deferred_functions() > 0 {
                // re-lock
                let map2 = SCRIPT_FUNCTIONS.lock().unwrap();
                if !map2.contains_key(function_name) {
                    panic!("Invalid script function {}", function_name);
                }
            } else {
                panic!("Invalid script function {}", function_name);
            }
        }
    }

    let map = SCRIPT_FUNCTIONS.lock().unwrap();
    let info = map
        .get(function_name)
        .expect("dispatch_script_function: function not found after checking deferred");
    let code_ref = info
        .get("func")
        .unwrap_or_else(|| panic!("No handler registered for {}", function_name));
    code_ref(args)
}

/// Parse a string of arguments in a simplistic style.
/// 
/// - Splits on commas that are not within single quotes,
/// - Each argument is either a number (matching `-?\d+`) or a single-quoted string.
/// 
/// Examples:
/// 
/// ```
/// let args = parse_script_arguments("123, 'hello', '12, 34', -5");
/// assert_eq!(args, vec!["123", "hello", "12, 34", "-5"]);
/// ```
pub fn parse_script_arguments(list_str: &str) -> Vec<String> {
    // Early return if the string is empty.
    if list_str.is_empty() {
        return Vec::new();
    }

    // Split the string on commas that are not within single quotes.
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut in_quote = false;

    // Iterate over the string by character index.
    for (i, ch) in list_str.char_indices() {
        if ch == '\'' {
            // Toggle our "in quote" flag.
            in_quote = !in_quote;
        } else if ch == ',' && !in_quote {
            // Found a comma outside a quoted string: slice out the piece.
            pieces.push(&list_str[start..i]);
            start = i + 1; // start next piece after the comma
        }
    }
    // Push the last piece.
    pieces.push(&list_str[start..]);

    let mut results = Vec::new();
    // For each piece, trim it and then extract the inner part if it is quoted.
    for piece in pieces {
        let trimmed = piece.trim();
        if trimmed.is_empty() {
            continue;
        }
        // If the argument is a quoted string, remove the surrounding quotes.
        if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
            results.push(trimmed[1..trimmed.len() - 1].to_string());
        } else {
            // Otherwise, just use the trimmed piece (e.g. a numeric literal).
            results.push(trimmed.to_string());
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(parse_script_arguments(""), Vec::<String>::new());
    }

    #[test]
    fn test_numeric_only() {
        assert_eq!(parse_script_arguments("123"), vec!["123"]);
        assert_eq!(parse_script_arguments("-456"), vec!["-456"]);
    }

    #[test]
    fn test_quoted_only() {
        assert_eq!(parse_script_arguments("'hello'"), vec!["hello"]);
        assert_eq!(parse_script_arguments("  'world'  "), vec!["world"]);
    }

    #[test]
    fn test_mixed_arguments() {
        let input = "123, 'hello', '12, 34', -5";
        let expected = vec!["123", "hello", "12, 34", "-5"];
        assert_eq!(parse_script_arguments(input), expected);
    }

    #[test]
    fn test_incomplete_quotes() {
        // If the quotes are not balanced, the code leaves the argument unmodified.
        let input = "'incomplete, 789";
        let expected = vec!["'incomplete, 789"];
        assert_eq!(parse_script_arguments(input), expected);
    }

    #[test]
    fn test_extra_whitespace_and_empty_pieces() {
        // Empty pieces (or extra spaces) are ignored.
        let input = " 42 , ' spaced ' ,   -7 , , ";
        let expected = vec!["42", " spaced ", "-7"];
        assert_eq!(parse_script_arguments(input), expected);
    }
}
