use std::io::{BufRead, Write, stdin, stdout};

use crate::Result;

/// Ask the user the given yes or no question and wait for their input.
///
/// Returns `true` for yes.
pub fn ask_yes_no(question: &str) -> Result<bool> {
    let mut input = String::new();
    let stdin = stdin();

    print!("{question} [y/N]: ");
    stdout().flush()?;

    let mut handle = stdin.lock();
    handle.read_line(&mut input)?;

    input = input.trim().to_ascii_lowercase();

    Ok(input == "y" || input == "yes")
}
