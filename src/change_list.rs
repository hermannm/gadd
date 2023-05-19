use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use git2::{Index, IndexConflict, Repository};

use crate::{
    change_ordering::ChangeOrdering,
    statuses::{ConflictingStatus, Status, WORKTREE_STATUSES},
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

        let mut changes = ChangeList::get_changes(repository, &index)?;

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

    fn get_changes(repository: &Repository, index: &Index) -> Result<Vec<Change>> {
        let statuses = repository
            .statuses(None)
            .context("Failed to get change statuses for repository")?;

        let statuses_length = statuses.len();
        let mut changes = Vec::<Change>::with_capacity(statuses_length);
        let mut conflicting_change_paths = Vec::<Vec<u8>>::with_capacity(statuses_length);

        for status_entry in statuses.iter() {
            let status = status_entry.status();

            if status.is_ignored() {
                continue;
            }

            let path = status_entry.path_bytes().to_owned();

            if status.is_conflicted() {
                conflicting_change_paths.push(path);
            } else {
                changes.push(Change {
                    path,
                    status: Status::NonConflicting(status),
                });
            }
        }

        if !conflicting_change_paths.is_empty() {
            ChangeList::get_conflicting_change_statuses(
                &mut changes,
                conflicting_change_paths,
                index,
            )?;
        }

        Ok(changes)
    }

    fn get_conflicting_change_statuses(
        changes: &mut Vec<Change>,
        conflicting_change_paths: Vec<Vec<u8>>,
        index: &Index,
    ) -> Result<()> {
        let conflicts_length = conflicting_change_paths.len();

        let conflicts = index
            .conflicts()
            .context("Failed to get merge conflicts from Git index")?;

        let mut conflict_map = HashMap::<Vec<u8>, IndexConflict>::with_capacity(conflicts_length);

        for conflict in conflicts {
            let conflict = conflict.context("Failed to get merge conflict from Git index")?;

            let path: Vec<u8>;
            if let Some(ancestor) = &conflict.ancestor {
                path = ancestor.path.clone()
            } else if let Some(our) = &conflict.our {
                path = our.path.clone();
            } else if let Some(their) = &conflict.their {
                path = their.path.clone();
            } else {
                bail!("Failed to find path for merge conflict in Git index");
            }

            conflict_map.insert(path, conflict);
        }

        for path in conflicting_change_paths {
            let Some(conflict) = conflict_map.get(&path) else {
                let path = String::from_utf8_lossy(&path);
                bail!("Expected to find merge conflict in Git index for path '{path}', but found nothing");
            };

            use ConflictingStatus::*;

            let (ours, theirs) = match (&conflict.ancestor, &conflict.our, &conflict.their) {
                (Some(_), Some(_), Some(_)) => (Unmerged, Unmerged),
                (Some(_), Some(_), None) => (Unmerged, Deleted),
                (Some(_), None, Some(_)) => (Deleted, Unmerged),
                (Some(_), None, None) => (Deleted, Deleted),
                (None, Some(_), Some(_)) => (Added, Added),
                (None, Some(_), None) => (Added, Unmerged),
                (None, None, Some(_)) => (Unmerged, Added),
                (None, None, None) => bail!("Invalid index merge conflict entry"),
            };

            changes.push(Change {
                path,
                status: Status::Conflicting { ours, theirs },
            });
        }

        Ok(())
    }

    pub fn refresh_changes(&mut self) -> Result<()> {
        self.changes = ChangeList::get_changes(self.repository, &self.index)?;
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

        if matches!(change.status, Status::NonConflicting(status) if status.is_wt_deleted()) {
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

        if matches!(change.status, Status::NonConflicting(status) if status.is_index_new()) {
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

        let mut first_worktree_change_index: Option<usize> = None;

        for (i, change) in self.changes.iter().enumerate() {
            match change.status {
                Status::Conflicting { .. } => {
                    self.index_of_selected_change = i;
                    return;
                }
                Status::NonConflicting(status) => {
                    if first_worktree_change_index.is_none()
                        && WORKTREE_STATUSES
                            .into_iter()
                            .any(|worktree_status| status.intersects(worktree_status))
                    {
                        first_worktree_change_index = Some(i);
                    }
                }
            }
        }

        if let Some(index) = first_worktree_change_index {
            self.index_of_selected_change = index;
        }
    }
}
