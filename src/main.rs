use std::fs::File;

use anyhow::{Context, Result};
use changes::change_list::ChangeList;
use clap::Parser;
use event_loop::run_event_loop;
use git2::Repository;
use rendering::{fullscreen::FullscreenRenderer, inline::render_inline};

mod changes;
mod event_loop;
mod fetch;
mod rendering;
mod statuses;

/// Command-line utility for staging changes to Git (alternative to git-add's interactive mode).
#[derive(Parser, Debug)]
struct Args {
    /// Skip the interactive staging area and just print the current status of the repository, in a
    /// similar format to `git status -s`.
    #[arg(short, long)]
    status: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let repo =
        Repository::discover(".").context("Failed to find Git repository at current location")?;

    let mut change_list = ChangeList::new(&repo)?;

    let mut stdout = get_raw_stdout();

    if !args.status {
        if change_list.changes.is_empty() {
            println!("No changes!");
            return Ok(());
        }

        let mut renderer = FullscreenRenderer::new(&mut stdout)?;
        renderer.render(&change_list)?;
        // Consumes renderer, exiting fullscreen when it's done
        run_event_loop(&mut change_list, renderer)?;

        change_list
            .refresh_changes()
            .context("Failed to refresh changes on exit")?;
        change_list
            .update_upstream_commits_diff()
            .context("Failed to update difference with upstream on exit")?;
    }

    render_inline(&mut stdout, &change_list).context("Failed to render changes")?;

    Ok(())
}

type Stdout = File;

#[cfg(unix)]
fn get_raw_stdout() -> Stdout {
    use std::os::unix::io::FromRawFd;

    const STDOUT_FILE_DESCRIPTOR: i32 = 1;
    unsafe { File::from_raw_fd(STDOUT_FILE_DESCRIPTOR) }
}

#[cfg(windows)]
fn get_raw_stdout() -> Stdout {
    use kernel32::GetStdHandle;
    use std::os::windows::io::FromRawHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;

    unsafe { File::from_raw_handle(GetStdHandle(STD_OUTPUT_HANDLE)) }
}
