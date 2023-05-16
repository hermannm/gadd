use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};
use git2::Status;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem},
    Frame, Terminal,
};

use crate::{
    change_list::{Change, ChangeList},
    statuses::{get_status_symbol, INDEX_STATUSES, WORKTREE_STATUSES},
    Stdout,
};

const INPUT_CONTROLS: [[&str; 2]; 5] = [
    ["[enter]", "done"],
    ["[space]", "stage change"],
    ["[r]", "unstage change"],
    ["[up]", "move up"],
    ["[down]", "move down"],
];

type TerminalBackend<'stdout> = CrosstermBackend<&'stdout mut Stdout>;

pub(crate) struct FullscreenRenderer<'stdout> {
    terminal: Terminal<TerminalBackend<'stdout>>,
    input_controls_widget: Block<'static>,
}

impl FullscreenRenderer<'_> {
    pub fn new(stdout: &mut Stdout) -> Result<FullscreenRenderer> {
        terminal::enable_raw_mode()?;
        stdout
            .queue(terminal::EnterAlternateScreen)?
            .queue(cursor::Hide)?
            .flush()?;

        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))?;
        let input_controls_widget = FullscreenRenderer::new_input_controls_widget();

        Ok(FullscreenRenderer {
            terminal,
            input_controls_widget,
        })
    }

    pub fn render(&mut self, change_list: &mut ChangeList) -> Result<()> {
        self.terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(frame.size());

            FullscreenRenderer::render_change_list(change_list, frame, chunks[0]);
            frame.render_widget(self.input_controls_widget.clone(), chunks[1]);
        })?;

        Ok(())
    }

    fn render_change_list(
        change_list: &mut ChangeList,
        frame: &mut Frame<TerminalBackend<'_>>,
        area: Rect,
    ) {
        let mut list_items = Vec::<ListItem>::with_capacity(change_list.changes.len());

        for (i, change) in change_list.changes.iter().enumerate() {
            let is_selected =
                matches!(change_list.get_selected_change(), Some(selected) if selected == i);

            let list_item = FullscreenRenderer::new_change_list_item(change, is_selected);
            list_items.push(list_item);
        }

        let list_widget = List::new(list_items).start_corner(Corner::BottomLeft);

        frame.render_stateful_widget(list_widget, area, &mut change_list.selected_change);
    }

    fn new_change_list_item(change: &Change, is_selected: bool) -> ListItem {
        let mut line = Vec::<Span>::new();

        let status = change.status;

        if status == Status::WT_NEW {
            line.push(Span::styled("??", RED_TEXT));
        } else {
            if let Some(index_status_symbol) = get_status_symbol(status, INDEX_STATUSES) {
                line.push(Span::styled(index_status_symbol, GREEN_TEXT));
            } else {
                line.push(Span::raw(" "));
            }

            if let Some(worktree_status_symbol) = get_status_symbol(status, WORKTREE_STATUSES) {
                line.push(Span::styled(worktree_status_symbol, RED_TEXT));
            } else {
                line.push(Span::raw(" "));
            }
        }

        line.push(Span::raw(" "));

        line.push({
            let path_string = String::from_utf8_lossy(&change.path);

            if is_selected {
                Span::styled(path_string, SELECTED_TEXT)
            } else {
                Span::raw(path_string)
            }
        });

        ListItem::new(Spans::from(line))
    }

    fn new_input_controls_widget() -> Block<'static> {
        let blue_text = Style::default().fg(Color::Blue);

        let mut line = Vec::<Span>::with_capacity(3 * INPUT_CONTROLS.len() + 1);

        for (i, [button, description]) in INPUT_CONTROLS.into_iter().enumerate() {
            line.push(Span::styled(button, blue_text));
            line.push(Span::raw(" "));
            line.push(Span::raw(description));

            if i < INPUT_CONTROLS.len() - 1 {
                line.push(Span::raw(" "));
            }
        }

        Block::default().title(Spans::from(line))
    }
}

impl Drop for FullscreenRenderer<'_> {
    fn drop(&mut self) {
        let stdout = self.terminal.backend_mut();

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
            writeln!(stdout, "Error on cleanup: {error}").expect("Failed to write to stdout");
        }
    }
}

const RED_TEXT: Style = Style {
    fg: Some(Color::Red),
    ..EMPTY_STYLE
};

const GREEN_TEXT: Style = Style {
    fg: Some(Color::Green),
    ..EMPTY_STYLE
};

const SELECTED_TEXT: Style = Style {
    fg: Some(Color::Black),
    bg: Some(Color::White),
    ..EMPTY_STYLE
};

const EMPTY_STYLE: Style = Style {
    fg: None,
    bg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
