use std::{fs::File, io::Write};

use anyhow::Result;
use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    QueueableCommand,
};
use git2::Status;

use crate::{
    change_list::ChangeList,
    statuses::{get_status_symbol, INDEX_STATUSES, WORKTREE_STATUSES},
};

pub(crate) fn render_inline(change_list: &ChangeList) -> Result<()> {
    let mut stdout = get_raw_stdout();

    for change in change_list.changes.iter().rev() {
        let status = change.status;

        if status == Status::WT_NEW {
            stdout.queue(SetForegroundColor(Color::Red))?;
            stdout.write_all(b"??")?;
            stdout.queue(ResetColor)?;
        } else {
            if let Some(index_status_symbol) = get_status_symbol(status, INDEX_STATUSES) {
                stdout.queue(SetForegroundColor(Color::Green))?;
                stdout.write_all(index_status_symbol.as_bytes())?;
                stdout.queue(ResetColor)?;
            } else {
                stdout.write_all(b" ")?;
            }

            if let Some(worktree_status_symbol) = get_status_symbol(status, WORKTREE_STATUSES) {
                stdout.queue(SetForegroundColor(Color::Red))?;
                stdout.write_all(worktree_status_symbol.as_bytes())?;
                stdout.queue(ResetColor)?;
            } else {
                stdout.write_all(b" ")?;
            }
        }

        stdout.write_all(b" ")?;
        stdout.write_all(&change.path)?;
        stdout.write_all("\r\n".as_bytes())?;
    }

    stdout.flush()?;

    Ok(())
}

#[cfg(unix)]
fn get_raw_stdout() -> File {
    use std::os::unix::io::FromRawFd;

    unsafe { File::from_raw_fd(1) }
}

#[cfg(windows)]
fn get_raw_stdout() -> File {
    use kernel32::GetStdHandle;
    use std::os::windows::io::FromRawHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;

    unsafe { File::from_raw_handle(GetStdHandle(STD_OUTPUT_HANDLE)) }
}
