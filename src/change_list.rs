use std::io::Write;

use anyhow::{Context, Result};
use crossterm::{
    style::{ResetColor, SetForegroundColor},
    QueueableCommand,
};
use git2::{Index, Repository, Status};
use ratatui::{
    layout::{Corner, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{List, ListItem, ListState},
};

use crate::{
    change_ordering::ChangeOrdering,
    statuses::{get_status_symbol, INDEX_STATUSES, WORKTREE_STATUSES},
    utils::{bytes_to_path, new_index_entry},
    Frame, InlineTerminal,
};

pub(super) struct ChangeList<'repo> {
    pub changes: Vec<Change>,
    selected_change: ListState,
    change_ordering: ChangeOrdering,
    repository: &'repo Repository,
    index: Index,
}

pub(super) struct Change {
    pub path: Vec<u8>,
    pub status: Status,
}

impl<'repo> ChangeList<'repo> {
    pub fn new(repository: &'repo Repository) -> Result<ChangeList<'repo>> {
        let index = repository
            .index()
            .context("Failed to get Git index for repository")?;

        let mut changes = ChangeList::get_changes(repository)?;

        let change_ordering = ChangeOrdering::sort_changes_and_save_ordering(&mut changes);

        let mut change_list = ChangeList {
            changes,
            selected_change: ListState::default(),
            change_ordering,
            repository,
            index,
        };

        change_list.select_default_change();

        Ok(change_list)
    }

    fn get_changes(repository: &Repository) -> Result<Vec<Change>> {
        let statuses = repository.statuses(None)?;

        let mut changes = Vec::<Change>::with_capacity(statuses.len());

        for status_entry in statuses.iter() {
            let status = status_entry.status();

            if !status.is_ignored() {
                changes.push(Change {
                    path: status_entry.path_bytes().to_owned(),
                    status,
                });
            }
        }

        Ok(changes)
    }

    pub fn refresh_changes(&mut self) -> Result<()> {
        self.changes = ChangeList::get_changes(self.repository)?;
        let new_changes = self.change_ordering.sort_changes(&mut self.changes);

        let changes_length = self.changes.len();

        if let Some(mut index_of_selected_change) = self.get_selected_change() {
            index_of_selected_change += new_changes;

            if index_of_selected_change >= changes_length {
                index_of_selected_change = changes_length - 1;
            }

            self.select_change(index_of_selected_change);
        }

        Ok(())
    }

    pub fn render_fullscreen(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::style::Color;

        let red_text = Style::default().fg(Color::Red);
        let green_text = Style::default().fg(Color::Green);
        let selected_text = Style::default().fg(Color::Black).bg(Color::White);

        let mut items = Vec::<ListItem>::with_capacity(self.changes.len());

        for (i, change) in self.changes.iter().enumerate() {
            let mut line = Vec::<Span>::new();

            let status = change.status;

            if status == Status::WT_NEW {
                line.push(Span::styled("??", red_text));
            } else {
                if let Some(index_status_symbol) = get_status_symbol(status, INDEX_STATUSES) {
                    line.push(Span::styled(index_status_symbol, green_text));
                } else {
                    line.push(Span::raw(" "));
                }

                if let Some(worktree_status_symbol) = get_status_symbol(status, WORKTREE_STATUSES) {
                    line.push(Span::styled(worktree_status_symbol, red_text));
                } else {
                    line.push(Span::raw(" "));
                }
            }

            line.push(Span::raw(" "));

            line.push({
                let path_string = String::from_utf8_lossy(&change.path);

                if matches!(self.get_selected_change(), Some(selected) if selected == i) {
                    Span::styled(path_string, selected_text)
                } else {
                    Span::raw(path_string)
                }
            });

            items.push(ListItem::new(Spans::from(line)));
        }

        let list = List::new(items).start_corner(Corner::BottomLeft);

        frame.render_stateful_widget(list, area, &mut self.selected_change);
    }

    pub fn render_inline(&self, terminal: &mut InlineTerminal) -> Result<()> {
        use crossterm::style::Color;

        for change in self.changes.iter().rev() {
            let status = change.status;

            if status == Status::WT_NEW {
                terminal.queue(SetForegroundColor(Color::Red))?;
                terminal.write_all(b"??")?;
                terminal.queue(ResetColor)?;
            } else {
                if let Some(index_status_symbol) = get_status_symbol(status, INDEX_STATUSES) {
                    terminal.queue(SetForegroundColor(Color::Green))?;
                    terminal.write_all(index_status_symbol.as_bytes())?;
                    terminal.queue(ResetColor)?;
                } else {
                    terminal.write_all(b" ")?;
                }

                if let Some(worktree_status_symbol) = get_status_symbol(status, WORKTREE_STATUSES) {
                    terminal.queue(SetForegroundColor(Color::Red))?;
                    terminal.write_all(worktree_status_symbol.as_bytes())?;
                    terminal.queue(ResetColor)?;
                } else {
                    terminal.write_all(b" ")?;
                }
            }

            terminal.write_all(b" ")?;
            terminal.write_all(&change.path)?;
            terminal.write_all("\r\n".as_bytes())?;
        }

        terminal.flush()?;

        Ok(())
    }

    pub fn stage_selected_change(&mut self) -> Result<()> {
        let Some(selected_change) = self.get_selected_change() else {
            return Ok(());
        };

        let change = &self.changes[selected_change];
        let path = bytes_to_path(&change.path);

        if change.status.is_wt_deleted() {
            self.index.remove_path(path)?;
        } else {
            self.index.add_path(path)?;
        }

        self.index.write()?;
        self.refresh_changes()?;
        Ok(())
    }

    pub fn unstage_selected_change(&mut self) -> Result<()> {
        let Some(selected_change) = self.get_selected_change() else {
            return Ok(());
        };

        let change = &self.changes[selected_change];
        let path = bytes_to_path(&change.path);

        if change.status.is_index_new() {
            self.index.remove_path(path)?;
        } else {
            let head = self.repository.head()?;
            let tree = head.peel_to_tree()?;
            let tree_entry = tree.get_path(path)?;

            let index_entry = new_index_entry(
                tree_entry.id(),
                tree_entry.filemode() as u32,
                change.path.clone(),
            );

            self.index.add(&index_entry)?;
        }

        self.index.write()?;
        self.refresh_changes()?;

        Ok(())
    }

    pub fn increment_selected_change(&mut self) {
        let Some(selected_change) = self.get_selected_change() else {
            return;
        };

        if selected_change < self.changes.len() - 1 {
            self.select_change(selected_change + 1);
        }
    }

    pub fn decrement_selected_change(&mut self) {
        let Some(index_of_selected_change) = self.get_selected_change() else {
            return;
        };

        if index_of_selected_change > 0 {
            self.select_change(index_of_selected_change - 1);
        }
    }

    fn get_selected_change(&self) -> Option<usize> {
        self.selected_change.selected()
    }

    fn select_change(&mut self, index_of_selected_change: usize) {
        self.selected_change.select(Some(index_of_selected_change));
    }

    fn select_default_change(&mut self) {
        let mut highest_index_per_worktree_status: Vec<(Status, Option<usize>)> = WORKTREE_STATUSES
            .into_iter()
            .map(|status| (status, None))
            .collect();

        for (i, change) in self.changes.iter().enumerate() {
            for (status, index) in highest_index_per_worktree_status.iter_mut() {
                if change.status == *status {
                    *index = Some(i);
                    break;
                }
            }
        }

        for (_, index) in highest_index_per_worktree_status.into_iter().rev() {
            if let Some(index) = index {
                self.select_change(index);
                return;
            }
        }
    }
}
