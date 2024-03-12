use std::io::Write;

use anyhow::Result;
use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    QueueableCommand,
};

use crate::{changes::change_list::ChangeList, Stdout};

use super::status_symbols::{get_status_symbols, StatusSymbol};

pub(crate) fn render_inline(stdout: &mut Stdout, change_list: &ChangeList) -> Result<()> {
    stdout.queue(SetForegroundColor(Color::Grey))?;
    stdout.write_all(b"##")?;
    stdout.queue(ResetColor)?;
    stdout.write_all(b" ")?;

    stdout.queue(SetForegroundColor(Color::DarkGreen))?;
    stdout.write_all(change_list.current_branch.name.as_bytes())?;
    stdout.queue(ResetColor)?;

    if let Some(upstream) = &change_list.upstream {
        stdout.write_all(b"...")?;
        stdout.queue(SetForegroundColor(Color::DarkRed))?;
        stdout.write_all(upstream.full_name.as_bytes())?;
        stdout.queue(ResetColor)?;

        let diff = &upstream.commits_diff;
        if diff.ahead != 0 || diff.behind != 0 {
            stdout.write_all(b" [")?;

            if diff.ahead != 0 {
                stdout.write_all(b"ahead ")?;
                stdout.queue(SetForegroundColor(Color::DarkGreen))?;
                stdout.write_all(diff.ahead.to_string().as_bytes())?;
                stdout.queue(ResetColor)?;

                if diff.behind != 0 {
                    stdout.write_all(b", ")?;
                }
            }

            if diff.behind != 0 {
                stdout.write_all(b"behind ")?;
                stdout.queue(SetForegroundColor(Color::DarkRed))?;
                stdout.write_all(diff.behind.to_string().as_bytes())?;
                stdout.queue(ResetColor)?;
            }

            stdout.write_all(b"]")?;
        }
    }

    stdout.write_all(b"\r\n")?;

    for change in change_list.changes.iter() {
        for status_symbol in get_status_symbols(&change.status) {
            match status_symbol {
                StatusSymbol::Green(symbol) => {
                    stdout.queue(SetForegroundColor(Color::DarkGreen))?;
                    stdout.write_all(symbol.as_bytes())?;
                    stdout.queue(ResetColor)?;
                }
                StatusSymbol::Red(symbol) => {
                    stdout.queue(SetForegroundColor(Color::DarkRed))?;
                    stdout.write_all(symbol.as_bytes())?;
                    stdout.queue(ResetColor)?;
                }
                StatusSymbol::Space => {
                    stdout.write_all(b" ")?;
                }
            }
        }

        stdout.write_all(b" ")?;
        stdout.write_all(&change.path)?;
        stdout.write_all(b"\r\n")?;
    }

    stdout.flush()?;

    Ok(())
}
