use std::io::Write;

use anyhow::Result;
use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    QueueableCommand,
};
use git2::Status;

use crate::{change_list::ChangeList, statuses::get_status_symbols, Stdout};

pub(crate) fn render_inline(stdout: &mut Stdout, change_list: &ChangeList) -> Result<()> {
    for change in change_list.changes.iter() {
        let status = change.status;

        if status == Status::WT_NEW {
            stdout.queue(SetForegroundColor(Color::Red))?;
            stdout.write_all(b"??")?;
            stdout.queue(ResetColor)?;
        } else {
            let status_symbols = get_status_symbols(&status);

            if let Some(index_status_symbol) = status_symbols[0] {
                stdout.queue(SetForegroundColor(Color::Green))?;
                stdout.write_all(index_status_symbol.as_bytes())?;
                stdout.queue(ResetColor)?;
            } else {
                stdout.write_all(b" ")?;
            }

            if let Some(worktree_status_symbol) = status_symbols[1] {
                stdout.queue(SetForegroundColor(Color::Red))?;
                stdout.write_all(worktree_status_symbol.as_bytes())?;
                stdout.queue(ResetColor)?;
            } else {
                stdout.write_all(b" ")?;
            }
        }

        stdout.write_all(b" ")?;
        stdout.write_all(&change.path)?;
        stdout.write_all(b"\r\n")?;
    }

    stdout.flush()?;

    Ok(())
}
