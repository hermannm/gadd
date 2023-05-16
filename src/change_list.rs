use anyhow::{Context, Result};
use git2::{Index, Repository, Status};
use ratatui::widgets::ListState;

use crate::{
    change_ordering::ChangeOrdering,
    statuses::WORKTREE_STATUSES,
    utils::{bytes_to_path, new_index_entry},
};

pub(super) struct ChangeList<'repo> {
    pub changes: Vec<Change>,
    pub selected_change: ListState,
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
        let statuses = repository
            .statuses(None)
            .context("Failed to get change statuses for repository")?;

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

    pub fn stage_selected_change(&mut self) -> Result<()> {
        let Some(selected_change) = self.get_selected_change() else {
            return Ok(());
        };

        let change = &self.changes[selected_change];
        let path = bytes_to_path(&change.path);

        if change.status.is_wt_deleted() {
            self.index.remove_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to remove deleted file '{path}' from Git index")
            })?;
        } else {
            self.index.add_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to add '{path}' to Git index")
            })?;
        }

        self.index.write().context("Failed to write to Git index")?;

        self.refresh_changes()
            .context("Failed to refresh changes after staging")?;

        Ok(())
    }

    pub fn unstage_selected_change(&mut self) -> Result<()> {
        let Some(selected_change) = self.get_selected_change() else {
            return Ok(());
        };

        let change = &self.changes[selected_change];
        let path = bytes_to_path(&change.path);

        if change.status.is_index_new() {
            self.index.remove_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to remove '{path}' from Git index")
            })?;
        } else {
            let head = self
                .repository
                .head()
                .context("Failed to get HEAD reference from repository")?;

            let tree = head
                .peel_to_tree()
                .context("Failed to get file tree from HEAD reference in repository")?;

            let tree_entry = tree.get_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to get tree entry for '{path}' from HEAD tree in repository")
            })?;

            let index_entry = new_index_entry(
                tree_entry.id(),
                tree_entry.filemode() as u32,
                change.path.clone(),
            );

            self.index.add(&index_entry).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to restore '{path}' from Git index to HEAD version")
            })?;
        }

        self.index.write().context("Failed to write to Git index")?;

        self.refresh_changes()
            .context("Failed to refresh changes after unstaging")?;

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

    pub fn get_selected_change(&self) -> Option<usize> {
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
