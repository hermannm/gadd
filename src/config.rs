use crate::event_loop::{spawn_named_thread, MustReceive, MustSend};
use crate::open_repository;
use anyhow::{Context, Result};
use crossbeam_channel::Receiver;
use git2::ErrorCode;

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

    pub(crate) fn load() -> Result<Config> {
        let mut git_config = open_repository()?
            .config()
            .context("Failed to open Git config")?;
        git_config = git_config
            .snapshot()
            .context("Failed to create snapshot of Git config")?;

        let commit_flags: Vec<String> = match git_config.get_str("gadd.commitFlags") {
            Ok(commit_flags_string) => commit_flags_string
                .split(' ')
                .filter_map(|flag| {
                    if Config::ALLOWED_COMMIT_FLAGS.contains(&flag) {
                        Some(flag.to_owned())
                    } else {
                        None
                    }
                })
                .collect(),
            Err(err) => {
                if err.code() == ErrorCode::NotFound {
                    Vec::new()
                } else {
                    return Err(err)
                        .context("Failed to get 'gadd.commitFlags' Git config variable");
                }
            }
        };

        Ok(Config { commit_flags })
    }
}

/// Struct for loading [Config] asynchronously in a separate thread, as this is a blocking operation
/// that we can start performing immediately when the application loads, and then have the result
/// ready for when another operation may need the config (e.g. when committing).
pub(crate) struct ConfigLoader {
    /// `None` until we receive the loaded config from the `receiver` channel.
    ///
    /// Need to store the error in an `Option`, so that the first caller can take ownership of the
    /// error by calling [Option::take].
    config_result: Option<Result<Config, Option<anyhow::Error>>>,
    receiver: Receiver<Result<Config>>,
}

impl ConfigLoader {
    pub fn new() -> ConfigLoader {
        let (config_sender, config_receiver) = crossbeam_channel::bounded::<Result<Config>>(1);

        spawn_named_thread("ConfigLoader", move || {
            config_sender.must_send(Config::load())
        });

        ConfigLoader {
            config_result: None,
            receiver: config_receiver,
        }
    }

    /// See [ConfigLoader::config_result] for why we wrap the error in an option.
    pub fn get_config(&mut self) -> &mut Result<Config, Option<anyhow::Error>> {
        self.config_result
            .get_or_insert_with(|| self.receiver.must_recv().map_err(|err| Some(err)))
    }
}
