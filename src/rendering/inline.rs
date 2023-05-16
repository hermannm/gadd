use std::io::Write;

use anyhow::Result;
use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    QueueableCommand,
};

use crate::{change_list::ChangeList, Stdout};

use super::status_text::StatusText;

pub(crate) fn render_inline(stdout: &mut Stdout, change_list: &ChangeList) -> Result<()> {
    for change in change_list.changes.iter() {
        let status_text = StatusText::from(&change.status);

        if let Some(green_status_text) = status_text.green_text {
            stdout.queue(SetForegroundColor(Color::Green))?;
            stdout.write_all(green_status_text.as_bytes())?;
            stdout.queue(ResetColor)?;
        } else {
            stdout.write_all(b" ")?;
        }

        if let Some(red_status_text) = status_text.red_text {
            stdout.queue(SetForegroundColor(Color::Red))?;
            stdout.write_all(red_status_text.as_bytes())?;
            stdout.queue(ResetColor)?;
        } else {
            stdout.write_all(b" ")?;
        }

        stdout.write_all(b" ")?;
        stdout.write_all(&change.path)?;
        stdout.write_all(b"\r\n")?;
    }

    stdout.flush()?;

    Ok(())
}
