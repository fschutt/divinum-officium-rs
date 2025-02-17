//! Text-based I/O for the Divinum Officium Project.
//!
//! This module provides basic file reading and writing functionality,
//! analogous to the FileIO.pm module in the Perl codebase.
//!
//! Both functions assume UTF-8 encoding for input and output.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Reads a text file (assumed to be in UTF‑8) and returns its lines as a vector of strings.
///
/// This function:
/// - Opens the file at the specified path.
/// - Reads the entire contents into a string.
/// - Removes a leading UTF‑8 byte order mark (BOM) if present.
/// - Splits the contents into lines (handling both Unix (`\n`) and Windows (`\r\n`) line breaks).
///
/// # Arguments
///
/// * `filename` - A path-like value that specifies the file to read.
///
/// # Returns
///
/// * `Ok(Vec<String>)` containing the lines of the file if successful.
/// * `Err(io::Error)` if there is an error opening or reading the file.
///
/// # Examples
///
/// ```no_run
/// use divinum_officium::fileio::do_read;
///
/// # fn main() -> std::io::Result<()> {
/// let lines = do_read("data/some_file.txt")?;
/// for line in lines {
///     println!("{}", line);
/// }
/// # Ok(())
/// # }
/// ```
pub fn do_read<P: AsRef<Path>>(filename: P) -> io::Result<Vec<String>> {
    // Read the entire file contents as a UTF-8 string.
    let content = fs::read_to_string(filename)?;

    // If the file is empty, return an empty vector.
    if content.is_empty() {
        return Ok(Vec::new());
    }

    // Remove the UTF-8 BOM if it exists.
    let content = if content.starts_with('\u{FEFF}') {
        content.trim_start_matches('\u{FEFF}').to_string()
    } else {
        content
    };

    // Split the content into lines.
    // The `.lines()` iterator splits on both `\n` and `\r\n` and does not include the newline characters.
    let lines = content.lines().map(|line| line.to_string()).collect();

    Ok(lines)
}

/// Writes the given content to a file in UTF‑8 encoding.
///
/// The function takes a filename and an iterator of items that can be
/// converted to a string slice. Each item is written sequentially to the file.
///
/// # Arguments
///
/// * `filename` - A path-like value specifying the file to write to.
/// * `contents` - An iterator of items convertible to string slices; the caller
///   is responsible for including newlines (`\n`) if desired.
///
/// # Returns
///
/// * `Ok(())` if the write operation succeeds.
/// * `Err(io::Error)` if an error occurs while opening or writing the file.
///
/// # Examples
///
/// ```no_run
/// use divinum_officium::fileio::do_write;
///
/// # fn main() -> std::io::Result<()> {
/// let lines = vec![
///     "Line one\n",
///     "Line two\n",
///     "Line three\n",
/// ];
/// do_write("output.txt", lines)?;
/// # Ok(())
/// # }
/// ```
pub fn do_write<P, I, S>(filename: P, contents: I) -> io::Result<()>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut file = fs::File::create(filename)?;
    for content in contents {
        file.write_all(content.as_ref().as_bytes())?;
    }
    Ok(())
}
