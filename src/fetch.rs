use crate::changes::branches::{LocalBranch, UpstreamCommitsDiff};
use crate::commands::command_error;
use crate::open_repository;
use anyhow::{Context, Result};
use git2::BranchType;
use std::process::Command;

pub(crate) fn fetch(current_branch: &LocalBranch) -> Result<UpstreamCommitsDiff> {
    let output = Command::new("git")
        .arg("fetch")
        // Fail instead of prompting for credentials
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_SSH_COMMAND", "ssh -oBatchMode=yes")
        .output()
        .context("Failed to run 'git fetch'")?;
    if !output.status.success() {
        return Err(command_error(&output, "'git fetch' failed"));
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
