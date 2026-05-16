use crate::changes::branches::{LocalBranch, UpstreamCommitsDiff};
use crate::open_repository;
use anyhow::{anyhow, Context, Error, Result};
use git2::BranchType;
use std::process::{Command, Output};

pub(crate) fn fetch(current_branch: &LocalBranch) -> Result<UpstreamCommitsDiff> {
    let output = Command::new("git")
        .arg("fetch")
        .output()
        .context("Failed to run 'git fetch'")?;
    if !output.status.success() {
        return Err(build_command_error(&output, "'git fetch' failed"));
    }

    let repo = open_repository()?;

    let current_branch_reference = repo
        .find_branch(&current_branch.name, BranchType::Local)
        .with_context(|| format!("Failed to find branch with name {}", current_branch.name))?;

    let upstream_reference = current_branch_reference
        .upstream()
        .context("Failed to get upstream of current branch")?;

    let new_upstream_object_id = upstream_reference
        .get()
        .target()
        .context("Failed to get the Git object ID of the upstream branch")?;

    UpstreamCommitsDiff::from_repo(&repo, current_branch.object_id, new_upstream_object_id)
}

/// Builds an `anyhow::Error` from the given command output and a main error message. `stderr` and
/// `stdout` from the command are attached to the cause chain of the error (if they're not blank),
/// and the main message is used as the outer error.
fn build_command_error(output: &Output, main_message: &'static str) -> Error {
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
