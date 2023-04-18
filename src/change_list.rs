use std::io::{StdoutLock, Write};

use anyhow::{Context, Result};
use crossterm::{
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use git2::{Index, Repository, Status};

use crate::{
    statuses::{get_status_symbol, StatusPriorityMap, INDEX_STATUSES, WORKTREE_STATUSES},
    utils::bytes_to_path,
};

pub(super) struct ChangeList<'repo> {
    repository: &'repo Repository,
    index: Index,
    changes: Vec<Change>,
    index_of_selected_change: usize,
    status_priority_map: StatusPriorityMap,
}

pub(super) struct Change {
    path: Vec<u8>,
    status: Status,
}

impl<'repo> ChangeList<'repo> {
    pub fn new(repository: &'repo Repository) -> Result<ChangeList<'repo>> {
        let index = repository
            .index()
            .context("Failed to get Git index for repository")?;

        let status_priority_map = StatusPriorityMap::new();

        let mut change_list = ChangeList {
            repository,
            index,
            changes: Vec::<Change>::new(),
            index_of_selected_change: 0,
            status_priority_map,
        };

        change_list.refresh_changes()?;

        for (i, change) in change_list.changes.iter().enumerate() {
            if change.status == Status::WT_NEW && i > 0 {
                change_list.index_of_selected_change = i - 1;
            } else if i == change_list.changes.len() - 1 {
                change_list.index_of_selected_change = i;
            }
        }

        Ok(change_list)
    }

    pub fn refresh_changes(&mut self) -> Result<()> {
        let statuses = self.repository.statuses(None)?;

        self.changes = Vec::<Change>::with_capacity(statuses.len());

        for status_entry in statuses.iter() {
            let status = status_entry.status();

            if !status.is_ignored() {
                self.changes.push(Change {
                    path: status_entry.path_bytes().to_owned(),
                    status,
                });
            }
        }

        self.changes.sort_by(|change_1, change_2| {
            self.status_priority_map
                .compare_statuses(&change_1.status, &change_2.status)
        });

        Ok(())
    }

    pub fn render(&self, stdout: &mut StdoutLock) -> Result<()> {
        for (i, change) in self.changes.iter().enumerate() {
            let status = change.status;

            if status == Status::WT_NEW {
                stdout.queue(SetForegroundColor(Color::Red))?;
                stdout.write_all("??".as_bytes())?;
                stdout.queue(ResetColor)?;
            } else {
                if let Some(index_status_symbol) = get_status_symbol(status, INDEX_STATUSES) {
                    stdout.queue(SetForegroundColor(Color::Green))?;
                    stdout.write_all(index_status_symbol.as_bytes())?;
                    stdout.queue(ResetColor)?;
                } else {
                    stdout.write_all(" ".as_bytes())?;
                }

                if let Some(worktree_status_symbol) = get_status_symbol(status, WORKTREE_STATUSES) {
                    stdout.queue(SetForegroundColor(Color::Red))?;
                    stdout.write_all(worktree_status_symbol.as_bytes())?;
                    stdout.queue(ResetColor)?;
                } else {
                    stdout.write_all(" ".as_bytes())?;
                }
            }

            stdout.write_all(" ".as_bytes())?;

            let is_selected_change = i == self.index_of_selected_change;
            if is_selected_change {
                stdout
                    .queue(SetBackgroundColor(Color::White))?
                    .queue(SetForegroundColor(Color::Black))?;
            }

            stdout.write_all(&change.path)?;

            if is_selected_change {
                stdout.queue(ResetColor)?;
            }

            stdout.write_all("\r\n".as_bytes())?;
        }

        Ok(())
    }

    pub fn stage_selected_change(&mut self) -> Result<()> {
        let change = &mut self.changes[self.index_of_selected_change];
        let path = bytes_to_path(&change.path);
        self.index.add_path(path)?;
        self.index.write()?;

        self.refresh_changes()?;

        let changes_length = self.changes.len();
        if self.index_of_selected_change >= changes_length {
            self.index_of_selected_change = changes_length - 1;
        }

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
