use std::io::{stdout, Stdout, Write};

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};
use git2::Repository;
use input::{render_input_controls, user_input_event_loop};
use tui::{self, backend::CrosstermBackend, widgets::Paragraph};

use crate::change_list::ChangeList;

mod change_list;
mod change_ordering;
mod input;
mod statuses;
mod utils;

type Terminal = tui::Terminal<CrosstermBackend<Stdout>>;

fn main() -> Result<()> {
    let _cleanup = Cleanup;

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(cursor::Hide)?
        .flush()?;

    let terminal_backend = CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(terminal_backend)?;

    let repository = Repository::discover(".").context("Failed to open repository")?;

    let mut change_list = ChangeList::new(&repository)?;

    render(&mut terminal, &change_list)?;

    user_input_event_loop(&mut terminal, &mut change_list)?;

    Ok(())
}

pub(self) fn render(terminal: &mut Terminal, change_list: &ChangeList) -> Result<()> {
    terminal.draw(|frame| {
        let mut lines = change_list.render();
        let input_control_line = render_input_controls();
        lines.push(input_control_line);

        let paragraph = Paragraph::new(lines);

        frame.render_widget(paragraph, frame.size());
    })?;

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
