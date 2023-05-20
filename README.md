# gadd

Command-line utility for staging changes to Git (alternative to git-add's interactive mode). Written in Rust, using [git2-rs](https://github.com/rust-lang/git2-rs) to interact with Git, and [ratatui](https://github.com/tui-rs-revival/ratatui) for the terminal UI.

## Screenshots

The Git staging area in `gadd`:<br />
![Screenshot of the gadd terminal application](https://github.com/hermannm/gadd/blob/assets/gadd-staging-area.png?raw=true)

## Installation

- Go to the [Releases page](https://github.com/hermannm/gadd/releases)
- Download the appropriate binary for your OS and architecture under Assets
  - On Linux/MacOS: Save the file as `gadd`
  - On Windows: Save the file as `gadd.exe`
- Update your `PATH` environment variable to include the folder where you saved `gadd`
  - On Linux/MacOS:
    - Using zsh: `echo 'export PATH=${HOME}/bin:${PATH}' >> ~/.zshrc`
    - Using Bash: `echo 'export PATH=${HOME}/bin:${PATH}' >> ~/.bashrc`
    - These examples assume you saved `gadd` in `${HOME}/bin` - replace it with your own path if you saved it somewhere else
  - On Windows:
    - Use the Windows search bar to search for "Edit environment variables", and open the suggested control panel
    - Under "User variables for \[user\]", find the one called "Path", and click "Edit..."
    - Click "New", and enter the path to the folder where you saved `gadd.exe`
- Restart your terminal
- You should now be able to type `gadd` inside a Git repo in the terminal to manage your changes!
