use std::io::{stdout, Stdout, Write};

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};
use git2::Repository;
use input::{event_loop::user_input_event_loop, widget::InputControlsWidget};
use ratatui::{
    self,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use utils::get_inline_terminal;

use crate::change_list::ChangeList;

mod change_list;
mod change_ordering;
mod input;
mod statuses;
mod utils;

type FullscreenTerminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type InlineTerminal = std::fs::File;
type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;

fn main() -> Result<()> {
    let repository = Repository::discover(".").context("Failed to open repository")?;
    let mut change_list = ChangeList::new(&repository)?;

    if change_list.changes.is_empty() {
        println!("No changes!");
        return Ok(());
    }

    run_fullscreen_application(&mut change_list)?;
    render_changes_on_exit(&mut change_list)?;

    Ok(())
}

fn run_fullscreen_application(change_list: &mut ChangeList) -> Result<()> {
    let cleanup = Cleanup;

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(cursor::Hide)?
        .flush()?;

    let mut terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))?;

    let input_widget = InputControlsWidget::new();

    render(&mut terminal, change_list, &input_widget)?;

    user_input_event_loop(&mut terminal, change_list, &input_widget)?;

    drop(cleanup);

    Ok(())
}

pub(self) fn render(
    terminal: &mut FullscreenTerminal,
    change_list: &mut ChangeList,
    input_widget: &InputControlsWidget,
) -> Result<()> {
    terminal.draw(|frame| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(frame.size());

        change_list.render_fullscreen(frame, chunks[0]);
        input_widget.render(frame, chunks[1]);
    })?;

    Ok(())
}

fn render_changes_on_exit(change_list: &mut ChangeList) -> Result<()> {
    change_list.refresh_changes()?;
    let mut terminal = get_inline_terminal();
    change_list.render_inline(&mut terminal)?;

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
