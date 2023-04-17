use std::{
    collections::HashMap,
    io::{Stdout, Write},
};

use anyhow::{Context, Result};
use crossterm::{
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use git2::{Index, Repository, Status, StatusEntry, Statuses};

pub(super) struct ChangeList<'repo> {
    repository: &'repo Repository,
    index: Index,
    changes: Vec<StatusEntry<'repo>>,
    index_of_selected_change: usize,
}

impl<'repo> ChangeList<'repo> {
    pub(super) fn new(
        repository: &'repo Repository,
        statuses: &'repo Statuses,
    ) -> Result<ChangeList<'repo>> {
        let index = repository
            .index()
            .context("Failed to get Git index for repository")?;

        let mut changes = Vec::<StatusEntry>::new();
        for status_entry in statuses.into_iter() {
            if !status_entry.status().is_ignored() {
                changes.push(status_entry);
            }
        }

        let order_map = make_order_map();

        changes.sort_by(|status_entry_1, status_entry_2| {
            let priority_1 = order_map[&status_entry_1.status()];
            let priority_2 = order_map[&status_entry_2.status()];
            priority_1.cmp(&priority_2)
        });

        let mut index_of_selected_change = 0;

        for (i, change) in changes.iter().enumerate() {
            if change.status() == Status::WT_NEW && i > 0 {
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
        })
    }

    pub(super) fn render(&self, stdout: &mut Stdout) -> Result<()> {
        for (i, change) in self.changes.iter().enumerate() {
            let status = change.status();

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

            stdout.write_all(change.path_bytes())?;

            if is_selected_change {
                stdout.queue(ResetColor)?;
            }

            stdout.write_all("\r\n".as_bytes())?;
        }

        Ok(())
    }

    pub(super) fn increment_selected_change(&mut self) {
        if self.index_of_selected_change < self.changes.len() - 1 {
            self.index_of_selected_change += 1;
        }
    }

    pub(super) fn decrement_selected_change(&mut self) {
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
        order_map.insert(worktree_status, i);

        for (j, index_status_2) in INDEX_STATUSES.into_iter().enumerate() {
            let priority = (i + 2) * status_length + j;
            order_map.insert(index_status_2 | worktree_status, priority);
        }
    }

    order_map.insert(Status::WT_NEW, (2 + status_length) * status_length + 2);

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