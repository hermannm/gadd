use anyhow::{Context, Result};
use git2::{BranchType, ErrorCode, Oid, Repository};

#[derive(Clone)]
pub(crate) struct LocalBranch {
    pub name: String,
    pub object_id: Oid,
}

#[derive(Clone)]
pub(crate) struct UpstreamBranch {
    pub name: String,
    pub remote_name: String,
    pub full_name: String,
    pub object_id: Oid,
}

pub(crate) fn get_current_branch(
    repo: &Repository,
) -> Result<(LocalBranch, Option<UpstreamBranch>)> {
    let head = repo
        .head()
        .context("Failed to get HEAD reference for repository")?;

    let current_branch_name = head
        .shorthand()
        .context("Current branch name was not valid UTF-8")?;

    let current_branch_object_id = head
        .target()
        .context("Failed to get the Git object ID of the HEAD reference")?;

    let current_branch = repo
        .find_branch(current_branch_name, BranchType::Local)
        .with_context(|| format!("Failed to find branch with name {current_branch_name}"))?;

    let upstream = match current_branch.upstream() {
        Ok(upstream_branch) => {
            let full_name = upstream_branch
                .name()
                .context("Failed to get name of upstream branch")?
                .context("Upstream branch name was not valid UTF-8")?;

            let (remote_name, name) = full_name
                .split_once('/')
                .context("Failed to get remote name from upstream branch")?;

            let upstream_object_id = upstream_branch
                .get()
                .target()
                .context("Failed to get the Git object ID of the upstream branch")?;

            Some(UpstreamBranch {
                name: name.to_string(),
                remote_name: remote_name.to_string(),
                full_name: full_name.to_string(),
                object_id: upstream_object_id,
            })
        }
        Err(err) => {
            if err.code() == ErrorCode::NotFound {
                None
            } else {
                return Err(err).context("Failed to get upstream of current branch");
            }
        }
    };

    Ok((
        LocalBranch {
            name: current_branch_name.to_string(),
            object_id: current_branch_object_id,
        },
        upstream,
    ))
}
