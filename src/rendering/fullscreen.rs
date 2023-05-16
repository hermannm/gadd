use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};
use git2::Status;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState},
    Terminal,
};

use crate::{
    change_list::{Change, ChangeList},
    statuses::get_status_symbols,
    Stdout,
};

const INPUT_CONTROLS: [[&str; 2]; 5] = [
    ["[enter]", "done"],
    ["[space]", "stage change"],
    ["[r]", "unstage change"],
    ["[up]", "move up"],
    ["[down]", "move down"],
];

pub(crate) struct FullscreenRenderer<'stdout> {
    terminal: Terminal<CrosstermBackend<&'stdout mut Stdout>>,
    input_controls_widget: Block<'static>,
    list_widget_state: ListState,
}

impl FullscreenRenderer<'_> {
    pub fn new(stdout: &mut Stdout) -> Result<FullscreenRenderer> {
        terminal::enable_raw_mode().context("Failed to enter terminal raw mode")?;
        stdout
            .queue(terminal::EnterAlternateScreen)
            .context("Failed to enter fullscreen in terminal")?
            .queue(cursor::Hide)
            .context("Failed to hide cursor")?
            .flush()
            .context("Failed to flush terminal setup to stdout")?;

        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))
            .context("Failed to create terminal instance")?;

        Ok(FullscreenRenderer {
            terminal,
            input_controls_widget: FullscreenRenderer::new_input_controls_widget(),
            list_widget_state: ListState::default(),
        })
    }

    pub fn render(&mut self, change_list: &ChangeList) -> Result<()> {
        self.update_list_widget_state(change_list);

        self.terminal
            .draw(|frame| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(1), Constraint::Length(1)])
                    .split(frame.size());

                let list_widget = FullscreenRenderer::list_widget_from_changes(change_list);
                frame.render_stateful_widget(list_widget, chunks[0], &mut self.list_widget_state);

                frame.render_widget(self.input_controls_widget.clone(), chunks[1]);
            })
            .context("Failed to draw to terminal")?;

        Ok(())
    }

    fn update_list_widget_state(&mut self, change_list: &ChangeList) {
        let changes_length = change_list.changes.len();

        if changes_length == 0 {
            self.list_widget_state.select(None);
            return;
        }

        let reverse_index = changes_length - 1 - change_list.index_of_selected_change;
        self.list_widget_state.select(Some(reverse_index));
    }

    fn list_widget_from_changes<'a>(change_list: &'a ChangeList) -> List<'a> {
        let mut list_items = Vec::<ListItem>::with_capacity(change_list.changes.len());

        for (i, change) in change_list.changes.iter().enumerate().rev() {
            let is_selected = i == change_list.index_of_selected_change;
            let list_item = FullscreenRenderer::list_item_widget_from_change(change, is_selected);
            list_items.push(list_item);
        }

        List::new(list_items).start_corner(Corner::BottomLeft)
    }

    fn list_item_widget_from_change(change: &Change, is_selected: bool) -> ListItem {
        let mut line = Vec::<Span>::new();

        let status = change.status;

        if status == Status::WT_NEW {
            line.push(Span::styled("??", RED_TEXT));
        } else {
            let status_symbols = get_status_symbols(&status);

            if let Some(index_status_symbol) = status_symbols[0] {
                line.push(Span::styled(index_status_symbol, GREEN_TEXT));
            } else {
                line.push(Span::raw(" "));
            }

            if let Some(worktree_status_symbol) = status_symbols[1] {
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
