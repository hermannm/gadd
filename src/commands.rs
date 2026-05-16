use anyhow::{anyhow, Error};
use std::process::Output;

/// Builds an `anyhow::Error` from the given command output and a main error message. `stderr` and
/// `stdout` from the command are attached to the cause chain of the error (if they're not blank),
/// and the main message is used as the outer error.
pub(crate) fn command_error(output: &Output, main_message: &'static str) -> Error {
    let mut error: Option<Error> = None;
    error = add_command_output_if_not_blank(error, &output.stderr);
    error = add_command_output_if_not_blank(error, &output.stdout);

    if let Some(error) = error {
        error.context(main_message)
    } else {
        anyhow!(main_message)
    }
}

fn add_command_output_if_not_blank(error: Option<Error>, output: &[u8]) -> Option<Error> {
    let output_string = String::from_utf8_lossy(output);
    let output_string = output_string.trim().to_owned();
    if !output_string.is_empty() {
        if let Some(existing_error) = error {
            Some(existing_error.context(output_string))
        } else {
            Some(anyhow!(output_string))
        }
    } else {
        error
    }
}
