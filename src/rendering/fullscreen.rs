use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{cursor, terminal, QueueableCommand};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, List, ListDirection, ListItem, ListState, Paragraph},
    Terminal,
};

use crate::{
    changes::{branches::FetchStatus, change::Change, change_list::ChangeList},
    Stdout,
};

use super::status_symbols::{get_status_symbols, StatusSymbol};

const INPUT_CONTROLS: [[&str; 2]; 7] = [
    ["[Space]", "Stage"],
    ["[R]", "Unstage"],
    ["[A]", "Stage all"],
    ["[U]", "Unstage all"],
    ["[F]", "Fetch"],
    ["[Enter]", "Commit"],
    ["[Esc]", "Exit"],
];

pub(crate) struct FullscreenRenderer<'stdout> {
    pub mode: RenderMode,
    terminal: Terminal<CrosstermBackend<&'stdout mut Stdout>>,
    list_widget_state: ListState,
    fullscreen_entered: bool,
}

pub(crate) enum RenderMode {
    ChangeList,
    HelpScreen,
}

impl FullscreenRenderer<'_> {
    pub fn new(stdout: &mut Stdout) -> Result<FullscreenRenderer> {
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))
            .context("Failed to create terminal instance")?;

        Ok(FullscreenRenderer {
            terminal,
            mode: RenderMode::ChangeList,
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
                let main_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(1), Constraint::Length(1)])
                    .split(frame.size());

                match self.mode {
                    RenderMode::ChangeList => {
                        let list_widget = FullscreenRenderer::list_widget_from_changes(change_list);
                        frame.render_stateful_widget(
                            list_widget,
                            main_layout[0],
                            &mut self.list_widget_state,
                        );
                    }
                    RenderMode::HelpScreen => {
                        let (help_screen, size) = FullscreenRenderer::new_help_screen_widget();
                        let help_screen_layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Min(1), Constraint::Length(size)])
                            .split(main_layout[0]);
                        frame.render_widget(help_screen, help_screen_layout[1]);
                    }
                };

                let (shortcut_widget, shortcut_size) = match self.mode {
                    RenderMode::ChangeList => FullscreenRenderer::new_help_shortcut_widget(),
                    RenderMode::HelpScreen => FullscreenRenderer::new_back_shortcut_widget(),
                };

                let bottom_bar_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(1), Constraint::Length(shortcut_size)])
                    .split(main_layout[1]);

                let branch_status = FullscreenRenderer::new_branch_status_widget(change_list);
                frame.render_widget(branch_status, bottom_bar_layout[0]);

                frame.render_widget(shortcut_widget, bottom_bar_layout[1])
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

        List::new(list_items).direction(ListDirection::BottomToTop)
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

        ListItem::new(Line::from(line))
    }

    fn new_branch_status_widget<'a>(change_list: &'a ChangeList) -> Block<'a> {
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

        Block::default().title(Line::from(line))
    }

    /// Returns (widget, size).
    fn new_help_screen_widget() -> (Paragraph<'static>, u16) {
        let mut lines = Vec::<Line>::with_capacity(2 + INPUT_CONTROLS.len());

        lines.push(Line::raw(
            "gadd - command-line utility for staging changes to Git.",
        ));
        lines.push(Line::raw(""));

        for [keybind, description] in INPUT_CONTROLS {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(keybind, BLUE_TEXT),
                Span::raw(" "),
                Span::raw(description),
            ]))
        }

        let size = (lines.len() + 1) as u16;
        (Paragraph::new(Text::from(lines)), size)
    }

    /// Returns (widget, size).
    fn new_help_shortcut_widget() -> (Block<'static>, u16) {
        let line = vec![Span::styled("[H]", BLUE_TEXT), Span::raw(" Help")];
        let size = " [H] Help".len() as u16;
        (Block::default().title(Line::from(line)), size)
    }

    /// Returns (widget, size).
    fn new_back_shortcut_widget() -> (Block<'static>, u16) {
        let line = vec![Span::styled("[Esc]", BLUE_TEXT), Span::raw(" Back")];
        let size = " [Esc] Back".len() as u16;
        (Block::default().title(Line::from(line)), size)
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

const BLUE_TEXT: Style = Style {
    fg: Some(Color::Blue),
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
    underline_color: None,
};
