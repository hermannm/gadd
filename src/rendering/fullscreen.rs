use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, List, ListItem, ListState},
    Terminal,
};

use crate::{
    changes::{branches::FetchStatus, change::Change, change_list::ChangeList},
    Stdout,
};

use super::status_symbols::{get_status_symbols, StatusSymbol};

const INPUT_CONTROLS: [[&str; 2]; 6] = [
    ["[Space]", "Stage"],
    ["[R]", "Unstage"],
    ["[A]", "Stage all"],
    ["[U]", "Unstage all"],
    ["[Enter]", "Commit"],
    ["[Esc]", "Exit"],
];

pub(crate) struct FullscreenRenderer<'stdout> {
    terminal: Terminal<CrosstermBackend<&'stdout mut Stdout>>,
    input_controls_widget: Block<'static>,
    list_widget_state: ListState,
    fullscreen_entered: bool,
}

impl FullscreenRenderer<'_> {
    pub fn new(stdout: &mut Stdout) -> Result<FullscreenRenderer> {
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))
            .context("Failed to create terminal instance")?;

        Ok(FullscreenRenderer {
            terminal,
            input_controls_widget: FullscreenRenderer::new_input_controls_widget(),
            list_widget_state: ListState::default(),
            fullscreen_entered: false,
        })
    }

    pub fn enter_fullscreen(&mut self) -> Result<()> {
        if !self.fullscreen_entered {
            terminal::enable_raw_mode().context("Failed to enter terminal raw mode")?;
            self.terminal
                .backend_mut()
                .queue(terminal::EnterAlternateScreen)
                .context("Failed to enter fullscreen in terminal")?
                .queue(cursor::Hide)
                .context("Failed to hide cursor when setting up terminal")?
                .flush()
                .context("Failed to flush terminal setup to stdout")?;
            self.fullscreen_entered = true
        }
        Ok(())
    }

    pub fn exit_fullscreen(&mut self) -> Result<()> {
        if self.fullscreen_entered {
            self.terminal
                .backend_mut()
                .queue(terminal::LeaveAlternateScreen)
                .context("Failed to exit fullscreen in terminal")?
                .queue(cursor::Show)
                .context("Failed to re-enable the cursor on fullscreen exit")?
                .flush()
                .context("Failed to flush terminal cleanup")?;
            terminal::disable_raw_mode().context("Failed to disable terminal raw mode")?;
            self.fullscreen_entered = false;
        }
        Ok(())
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

                let bottom_bar = FullscreenRenderer::new_bottom_bar_widget(change_list);
                frame.render_widget(bottom_bar, chunks[1]);
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

        for status_symbol in get_status_symbols(&change.status) {
            line.push(match status_symbol {
                StatusSymbol::Green(symbol) => Span::styled(symbol, GREEN_TEXT),
                StatusSymbol::Red(symbol) => Span::styled(symbol, RED_TEXT),
                StatusSymbol::Space => Span::raw(" "),
            });
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

    fn new_bottom_bar_widget<'a>(change_list: &'a ChangeList) -> Block<'a> {
        let mut line = Vec::<Span>::new();

        line.push(Span::styled("##", GRAY_TEXT));
        line.push(Span::raw(" "));

        line.push(Span::styled(&change_list.current_branch.name, GREEN_TEXT));

        if let Some(upstream) = &change_list.upstream {
            line.push(Span::raw("..."));
            line.push(Span::styled(&upstream.full_name, RED_TEXT));

            let diff = &upstream.commits_diff;
            if diff.ahead != 0 || diff.behind != 0 {
                line.push(Span::raw(" ["));

                if diff.ahead != 0 {
                    line.push(Span::raw("ahead "));
                    line.push(Span::styled(diff.ahead.to_string(), GREEN_TEXT));

                    if diff.behind != 0 {
                        line.push(Span::raw(", "))
                    }
                }

                if diff.behind != 0 {
                    line.push(Span::raw("behind "));
                    line.push(Span::styled(diff.behind.to_string(), RED_TEXT));
                }

                line.push(Span::raw("]"));
            }

            match &upstream.fetch_status {
                FetchStatus::Fetching => line.push(Span::styled(" Fetching...", GRAY_TEXT)),
                FetchStatus::FetchFailed => line.push(Span::styled(" Fetch failed", GRAY_TEXT)),
                FetchStatus::FetchComplete => {}
            }
        }

        Block::default().title(Spans::from(line))
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

const RED_TEXT: Style = Style {
    fg: Some(Color::Red),
    ..EMPTY_STYLE
};

const GREEN_TEXT: Style = Style {
    fg: Some(Color::Green),
    ..EMPTY_STYLE
};

const GRAY_TEXT: Style = Style {
    fg: Some(Color::Gray),
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
