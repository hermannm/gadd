use std::io::Write;

use anyhow::Result;
use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    QueueableCommand,
};

use crate::{change_list::ChangeList, Stdout};

use super::status_symbols::{get_status_symbols, StatusSymbol};

pub(crate) fn render_inline(stdout: &mut Stdout, change_list: &ChangeList) -> Result<()> {
    for change in change_list.changes.iter() {
        for status_symbol in get_status_symbols(&change.status) {
            match status_symbol {
                StatusSymbol::Green(symbol) => {
                    stdout.queue(SetForegroundColor(Color::Green))?;
                    stdout.write_all(symbol.as_bytes())?;
                    stdout.queue(ResetColor)?;
                }
                StatusSymbol::Red(symbol) => {
                    stdout.queue(SetForegroundColor(Color::Red))?;
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
