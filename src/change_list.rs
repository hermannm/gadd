use anyhow::{Context, Result};
use git2::{Index, Repository, Status};
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use crate::{
    change_ordering::ChangeOrdering,
    statuses::{get_status_symbol, INDEX_STATUSES, WORKTREE_STATUSES},
    utils::{bytes_to_path, new_index_entry},
};

pub(super) struct ChangeList<'repo> {
    repository: &'repo Repository,
    index: Index,
    changes: Vec<Change>,
    index_of_selected_change: usize,
    change_ordering: ChangeOrdering,
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

        let mut index_of_selected_change = 0;

        for (i, change) in changes.iter().enumerate() {
            if change.status == Status::WT_NEW && i > 0 {
                index_of_selected_change = i - 1;
            } else if i == changes.len() - 1 {
                index_of_selected_change = i;
            }
        }

        Ok(ChangeList {
            repository,
            index,
            changes,
            index_of_selected_change,
            change_ordering,
        })
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
        self.change_ordering.sort_changes(&mut self.changes);

        let changes_length = self.changes.len();
        if self.index_of_selected_change >= changes_length {
            self.index_of_selected_change = changes_length - 1;
        }

        Ok(())
    }

    pub fn render(&self) -> Vec<Spans> {
        let red_text = Style::default().fg(Color::Red);
        let green_text = Style::default().fg(Color::Green);
        let selected_text = Style::default().fg(Color::Black).bg(Color::White);

        let mut lines = Vec::<Spans>::with_capacity(self.changes.len());

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

                if i == self.index_of_selected_change {
                    Span::styled(path_string, selected_text)
                } else {
                    Span::raw(path_string)
                }
            });

            lines.push(Spans::from(line));
        }

        lines
    }

    pub fn stage_selected_change(&mut self) -> Result<()> {
        let change = &self.changes[self.index_of_selected_change];
        let path = bytes_to_path(&change.path);
        self.index.add_path(path)?;
        self.index.write()?;
        self.refresh_changes()?;
        Ok(())
    }

    pub fn unstage_selected_change(&mut self) -> Result<()> {
        let change = &self.changes[self.index_of_selected_change];
        let path = bytes_to_path(&change.path);

        let head = self.repository.head()?;
        let tree = head.peel_to_tree()?;
        let tree_entry = tree.get_path(path)?;

        let index_entry = new_index_entry(
            tree_entry.id(),
            tree_entry.filemode() as u32,
            change.path.clone(),
        );

        self.index.add(&index_entry)?;
        self.index.write()?;

        self.refresh_changes()?;

        Ok(())
    }

    pub fn increment_selected_change(&mut self) {
        if self.index_of_selected_change < self.changes.len() - 1 {
            self.index_of_selected_change += 1;
        }
    }

    pub fn decrement_selected_change(&mut self) {
        if self.index_of_selected_change > 0 {
            self.index_of_selected_change -= 1;
        }
    }
}
