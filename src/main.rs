use std::fs::File;

use anyhow::{Context, Result};
use changes::ChangeList;
use git2::Repository;
use input_handling::user_input_event_loop;
use rendering::{render_inline, FullscreenRenderer};

mod changes;
mod input_handling;
mod rendering;
mod statuses;

fn main() -> Result<()> {
    let repository =
        Repository::discover(".").context("Failed to find Git repository at current location")?;

    let mut change_list = ChangeList::new(&repository)?;

    if change_list.changes.is_empty() {
        println!("No changes!");
        return Ok(());
    }

    let mut stdout = get_raw_stdout();

    {
        let mut renderer = FullscreenRenderer::new(&mut stdout)?;
        renderer.render(&change_list)?;
        user_input_event_loop(&mut change_list, &mut renderer)?;
    } // Drops renderer, exiting fullscreen

    change_list
        .refresh_changes()
        .context("Failed to refresh changes on exit")?;

    render_inline(&mut stdout, &change_list).context("Failed to render changes on exit")?;

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
