use std::io::{stdout, StdoutLock, Write};

use anyhow::{Context, Result};
use crossterm::{
    cursor,
    terminal::{self, ClearType},
    QueueableCommand,
};
use git2::Repository;
use input::{render_input_controls, user_input_event_loop};

use crate::change_list::ChangeList;

mod change_list;
mod input;
mod utils;

fn main() -> Result<()> {
    let _cleanup = Cleanup;

    let mut stdout = stdout().lock();

    terminal::enable_raw_mode()?;

    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(cursor::Hide)?;

    let repository = Repository::discover(".").context("Failed to open repository")?;

    let mut change_list = ChangeList::new(&repository)?;

    render(&mut stdout, &change_list)?;

    user_input_event_loop(&mut stdout, &mut change_list)?;

    Ok(())
}

pub(self) fn render(stdout: &mut StdoutLock, change_list: &ChangeList) -> Result<()> {
    stdout.queue(terminal::Clear(ClearType::All))?;

    change_list.render(stdout)?;

    render_input_controls(stdout)?;

    stdout.write_all("\r".as_bytes())?;

    stdout.flush()?;

    Ok(())
}

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        let mut stdout = stdout();

        let alternate_screen_err = stdout
            .queue(terminal::LeaveAlternateScreen)
            .context("Failed to leave the alternate screen")
            .err();

        let show_cursor_err = stdout
            .queue(cursor::Show)
            .context("Failed to re-enable the cursor")
            .err();

        let flush_err = stdout
            .flush()
            .context("Failed to flush terminal cleanup")
            .err();

        let raw_mode_err = terminal::disable_raw_mode()
            .context("Failed to disable terminal raw mode")
            .err();

        for error in [
            alternate_screen_err,
            show_cursor_err,
            flush_err,
            raw_mode_err,
        ]
        .into_iter()
        .flatten()
        {
            println!("Error on cleanup: {error}");
        }
    }
}
