use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

use anyhow::{Context, Result};
use crossterm::{
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use git2::{Index, Repository, Status};

use crate::utils::bytes_to_path;

pub(super) struct ChangeList<'repo> {
    repository: &'repo Repository,
    index: Index,
    changes: Vec<Change>,
    index_of_selected_change: usize,
    order_map: HashMap<Status, usize>,
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

        let order_map = make_order_map();

        let mut change_list = ChangeList {
            repository,
            index,
            changes: Vec::<Change>::new(),
            index_of_selected_change: 0,
            order_map,
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

        self.changes.sort_by(|status_entry_1, status_entry_2| {
            let priority_1 = self.order_map[&status_entry_1.status];
            let priority_2 = self.order_map[&status_entry_2.status];
            priority_1.cmp(&priority_2)
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
                if let Some(printed_index_status) = get_printed_status(status, INDEX_STATUSES) {
                    stdout.queue(SetForegroundColor(Color::Green))?;
                    stdout.write_all(printed_index_status.as_bytes())?;
                    stdout.queue(ResetColor)?;
                } else {
                    stdout.write_all(" ".as_bytes())?;
                }

                if let Some(printed_worktree_status) = get_printed_status(status, WORKTREE_STATUSES)
                {
                    stdout.queue(SetForegroundColor(Color::Red))?;
                    stdout.write_all(printed_worktree_status.as_bytes())?;
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

fn get_printed_status(status: Status, statuses_to_check: [Status; 5]) -> Option<&'static str> {
    let status_symbols = ["M", "A", "R", "T", "D"];

    for (i, status_to_check) in statuses_to_check.into_iter().enumerate() {
        if status.intersects(status_to_check) {
            return Some(status_symbols[i]);
        }
    }

    None
}

fn make_order_map() -> HashMap<Status, usize> {
    let status_length = INDEX_STATUSES.len();

    let mut order_map = HashMap::<Status, usize>::new();

    for i in 0..status_length {
        let index_status = INDEX_STATUSES[i];
        let worktree_status = WORKTREE_STATUSES[i];

        order_map.insert(index_status, i);
        order_map.insert(worktree_status, i + status_length);

        for (j, index_status_2) in INDEX_STATUSES.into_iter().enumerate() {
            let priority = (i + 2) * status_length + j;
            order_map.insert(index_status_2 | worktree_status, priority);
        }
    }

    order_map.insert(Status::WT_NEW, (2 + status_length) * status_length);

    // TODO: Deal with conflicted

    order_map
}

const INDEX_STATUSES: [Status; 5] = [
    Status::INDEX_MODIFIED,
    Status::INDEX_NEW,
    Status::INDEX_RENAMED,
    Status::INDEX_TYPECHANGE,
    Status::INDEX_DELETED,
];

const WORKTREE_STATUSES: [Status; 5] = [
    Status::WT_MODIFIED,
    Status::WT_NEW,
    Status::WT_RENAMED,
    Status::WT_TYPECHANGE,
    Status::WT_DELETED,
];
