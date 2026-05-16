use anyhow::{Context, Result};
use git2::{ErrorCode, Repository};

/// User-defined config for gadd, set through git config variables. Can be defined per repo with
/// `git config <key> <value>`, or globally with `git config --global <key> <value>`.
pub(crate) struct Config {
    /// Flags to add to the `git commit` command that runs when the user presses 'Enter' inside gadd
    /// (also added to the `git commit --amend` command that runs when 'm' is pressed).
    ///
    /// Set by Git config variable `gadd.commitFlags` (multiple flags separated by space).
    ///
    /// Allowed flags are restricted by [Config::ALLOWED_COMMIT_FLAGS].
    pub commit_flags: Vec<String>,
}

impl Config {
    /// Defensive measure to prevent "hijacking" of commits by changing the `gadd.commitFlags`
    /// config variable. Consider expanding this list in the future.
    const ALLOWED_COMMIT_FLAGS: &[&str] = &["-n", "--no-verify"];

    pub(crate) fn load(repo: &Repository) -> Result<Config> {
        let mut git_config = repo.config().context("Failed to open Git config")?;
        git_config = git_config
            .snapshot()
            .context("Failed to create snapshot of Git config")?;

        let commit_flags: Vec<String> = git_config
            .get_str("gadd.commitFlags")
            .map(|commit_flags_string| {
                commit_flags_string
                    .split(' ')
                    .filter_map(|flag| {
                        if Config::ALLOWED_COMMIT_FLAGS.contains(&flag) {
                            Some(flag.to_owned())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .or_else(|err| fallback_if_not_found(err, Vec::new))
            .context("Failed to get 'gadd.commitFlags' Git config variable")?;

        Ok(Config { commit_flags })
    }
}

fn fallback_if_not_found<T>(
    err: git2::Error,
    default: impl FnOnce() -> T,
) -> Result<T, git2::Error> {
    if err.code() == ErrorCode::NotFound {
        Ok(default())
    } else {
        Err(err)
    }
}
