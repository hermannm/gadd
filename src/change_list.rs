use anyhow::{Context, Result};
use git2::{Index, Repository, Status};

use crate::{
    change_ordering::ChangeOrdering,
    statuses::WORKTREE_STATUSES,
    utils::{bytes_to_path, new_index_entry},
};

pub(super) struct ChangeList<'repo> {
    pub changes: Vec<Change>,
    pub index_of_selected_change: usize,
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
            index_of_selected_change: 0,
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
        self.change_ordering.sort_changes(&mut self.changes);

        let changes_length = self.changes.len();

        if changes_length == 0 {
            self.index_of_selected_change = 0;
        } else if changes_length <= self.index_of_selected_change {
            self.index_of_selected_change = changes_length - 1;
        }

        Ok(())
    }

    pub fn stage_selected_change(&mut self) -> Result<()> {
        if self.changes.is_empty() {
            return Ok(());
        }

        let change = &self.changes[self.index_of_selected_change];
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
        if self.changes.is_empty() {
            return Ok(());
        }

        let change = &self.changes[self.index_of_selected_change];
        let path = bytes_to_path(&change.path);

        if change.status.is_index_new() {
            self.index.remove_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to remove '{path}' from Git index")
            })?;
        } else {
            // Unstaging changes to a previously added file involves:
            // 1. Getting the "tree entry" for the file in the HEAD tree of the repository
            //    (i.e. the current state of the file)
            // 2. Creating a new "index entry" from that tree entry and adding it to the Git index

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

    pub fn select_next_change(&mut self) {
        let changes_length = self.changes.len();

        if changes_length > 0 && self.index_of_selected_change < changes_length - 1 {
            self.index_of_selected_change += 1;
        }
    }

    pub fn select_previous_change(&mut self) {
        if self.index_of_selected_change > 0 {
            self.index_of_selected_change -= 1;
        }
    }

    fn select_default_change(&mut self) {
        if self.changes.is_empty() {
            return;
        }

        for (i, change) in self.changes.iter().enumerate() {
            if WORKTREE_STATUSES.contains(&change.status) {
                self.index_of_selected_change = i;
                return;
            }
        }

        for (i, change) in self.changes.iter().enumerate() {
            if change.status.is_conflicted() {
                self.index_of_selected_change = i;
            }
        }
    }
}
