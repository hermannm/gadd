<img alt="Ferris the Crab, mascot of the Rust programming language, holding Git logo" width="106" height="75" src="https://github.com/hermannm/gadd/blob/assets/ferris-the-git-crab.png?raw=true" />

# gadd

Command-line utility for staging changes to Git (alternative to git-add's interactive mode). Written
in Rust, using [git2-rs](https://github.com/rust-lang/git2-rs) to interact with Git, and
[ratatui](https://github.com/tui-rs-revival/ratatui) for the terminal UI.

**Published on:** [crates.io/crates/gadd](https://crates.io/crates/gadd)

**Contents**:

- [Screenshots](#screenshots)
- [Installation](#installation)
  - [Through Cargo (Rust package manager)](#through-cargo-rust-package-manager)
  - [Manually](#manually)
- [Maintainer's guide](#maintainers-guide)
- [Credits](#credits)

## Screenshots

The Git staging area in `gadd`:

![Screenshot of the gadd terminal application](https://github.com/hermannm/gadd/blob/assets/gadd-staging-area.png?raw=true)

## Installation

### Through Cargo (Rust package manager)

- Run `cargo install gadd`
- You should now be able to type `gadd` inside a Git repo in the terminal to manage your changes!

### Manually

- Go to the [Releases page](https://github.com/hermannm/gadd/releases)
- Download the appropriate binary for your OS and architecture under Assets
  - On Linux/MacOS: Save the file as `gadd`
  - On Windows: Save the file as `gadd.exe`
- Update your `PATH` environment variable to include the folder where you saved `gadd`
  - On Linux/MacOS:
    - Using zsh: `echo 'export PATH=${HOME}/bin:${PATH}' >> ~/.zshrc`
    - Using Bash: `echo 'export PATH=${HOME}/bin:${PATH}' >> ~/.bashrc`
    - These examples assume you saved `gadd` in `${HOME}/bin` - replace it with your own path if you
      saved it somewhere else
  - On Windows:
    - Use the Windows search bar to search for "Edit environment variables", and open the suggested
      control panel
    - Under "User variables for \[user\]", find the one called "Path", and click "Edit..."
    - Click "New", and enter the path to the folder where you saved `gadd.exe`
- Restart your terminal
- You should now be able to type `gadd` inside a Git repo in the terminal to manage your changes!

## Maintainer's guide

### Publishing a new release

- Bump version in `Cargo.toml`
- Add an entry to `CHANGELOG.md` (with the current date)
  - Remember to update the link section, and bump the version for the `[Unreleased]` link
- Create commit and tag for the release (update `TAG` variable in below command):
  ```
  TAG=vX.Y.Z && git commit -m "Release ${TAG}" && git tag -a "${TAG}" -m "Release ${TAG}" && git log --oneline -2
  ```
- Compile release binaries for all platforms:
  ```
  ./crosscompile.sh
  ```
  - You may have to install [`cross`](https://github.com/cross-rs/cross) first:
    ```
    cargo install cross --git https://github.com/cross-rs/cross
    ```
- Publish to [crates.io](https://crates.io):
  ```
  cargo publish
  ```
  - You may have to run `cargo login` first - see the Cargo book for help:
    [doc.rust-lang.org/cargo/reference/publishing.html](https://doc.rust-lang.org/cargo/reference/publishing.html)
- Push the commit and tag:
  ```
  git push && git push --tags
  ```
  - Our release workflow will then create a GitHub release with the pushed tag's changelog entry
- Attach binaries (built in cross-compile step above) to release on GitHub:
  [github.com/hermannm/gadd/releases](https://github.com/hermannm/gadd/releases)

## Credits

- Git logo adapted from [Jason Long](https://git-scm.com/downloads/logos) (licensed under
  [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/))
