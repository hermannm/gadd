use anyhow::{anyhow, bail, Context, Result};
use git2::{Index, IndexAddOption, Repository, StatusOptions, Statuses, Tree};

use crate::{
    change_ordering::ChangeOrdering,
    statuses::{ConflictingStatus, Status, WORKTREE_STATUSES},
    utils::{bytes_to_path, new_index_entry},
};

pub(super) struct ChangeList<'repo> {
    pub changes: Vec<Change>,
    pub index_of_selected_change: usize,
    ordering: ChangeOrdering,
    repository: &'repo Repository,
    index: Index,
}

impl<'repo> ChangeList<'repo> {
    pub fn new(repository: &'repo Repository) -> Result<ChangeList<'repo>> {
        let index = repository
            .index()
            .context("Failed to get Git index for repository")?;

        let statuses = get_statuses(repository)?;
        let statuses_length = statuses.len();

        let mut change_list = ChangeList {
            changes: Vec::<Change>::with_capacity(statuses_length),
            index_of_selected_change: 0,
            ordering: ChangeOrdering::with_capacity(statuses_length),
            repository,
            index,
        };

        change_list.populate_changes(statuses)?;

        change_list
            .ordering
            .sort_changes_and_save_ordering(&mut change_list.changes);

        change_list.select_default_change();

        Ok(change_list)
    }

    fn populate_changes(&mut self, statuses: Statuses) -> Result<()> {
        self.changes.clear();

        let mut conflicting_change_paths = Vec::<Vec<u8>>::with_capacity(self.changes.capacity());

        for status_entry in statuses.iter() {
            let status = status_entry.status();

            let path = status_entry.path_bytes().to_owned();

            if status.is_conflicted() {
                conflicting_change_paths.push(path);
            } else {
                self.changes.push(Change {
                    path,
                    status: Status::NonConflicting(status),
                });
            }
        }

        if !conflicting_change_paths.is_empty() {
            self.populate_conflicting_changes(conflicting_change_paths)
                .context("Failed to get statuses for paths in merge conflict")?;
        }

        Ok(())
    }

    fn populate_conflicting_changes(
        &mut self,
        mut conflicting_change_paths: Vec<Vec<u8>>,
    ) -> Result<()> {
        let conflicts = self
            .index
            .conflicts()
            .context("Failed to get merge conflicts from Git index")?;

        for conflict in conflicts {
            let conflict = conflict.context("Failed to get merge conflict from Git index")?;

            let conflict_path: &[u8];
            if let Some(ancestor) = &conflict.ancestor {
                conflict_path = &ancestor.path;
            } else if let Some(our) = &conflict.our {
                conflict_path = &our.path;
            } else if let Some(their) = &conflict.their {
                conflict_path = &their.path;
            } else {
                bail!("Failed to find path for merge conflict in Git index");
            }

            let index = conflicting_change_paths
                .iter()
                .position(|change_path| change_path == conflict_path)
                .ok_or_else(|| {
                    let path = String::from_utf8_lossy(conflict_path);
                    anyhow!("Expected to find conflicting change path for merge conflict in '{path}', but found nothing")
                })?;

            let change_path = conflicting_change_paths.remove(index);

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

            self.changes.push(Change {
                path: change_path,
                status: Status::Conflicting { ours, theirs },
            });
        }

        if !conflicting_change_paths.is_empty() {
            bail!("Failed to find Git index entries for all conflicting paths in merge conflict");
        }

        Ok(())
    }

    pub fn refresh_changes(&mut self) -> Result<()> {
        let statuses = get_statuses(self.repository)?;
        self.populate_changes(statuses)?;
        self.ordering.sort_changes(&mut self.changes);

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
        change.stage(&mut self.index)?;

        self.index.write().context("Failed to write to Git index")?;

        self.refresh_changes()
            .context("Failed to refresh changes after staging")?;

        Ok(())
    }

    pub fn stage_all_changes(&mut self) -> Result<()> {
        self.index
            .add_all(["*"], IndexAddOption::DEFAULT, None)
            .context("Failed to add all changes to Git index")?;

        self.index.write().context("Failed to write to Git index")?;

        self.refresh_changes()
            .context("Failed to refresh changes after staging")?;

        Ok(())
    }

    pub fn unstage_selected_change(&mut self) -> Result<()> {
        if self.changes.is_empty() {
            return Ok(());
        }

        let repository_head_tree = get_repository_head_tree(self.repository)?;

        let change = &self.changes[self.index_of_selected_change];
        change.unstage(&mut self.index, &repository_head_tree)?;

        self.index.write().context("Failed to write to Git index")?;

        self.refresh_changes()
            .context("Failed to refresh changes after unstaging")?;

        Ok(())
    }

    pub fn unstage_all_changes(&mut self) -> Result<()> {
        let repository_head_tree = get_repository_head_tree(self.repository)?;

        for change in &self.changes {
            change.unstage(&mut self.index, &repository_head_tree)?;
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

pub(super) struct Change {
    pub path: Vec<u8>,
    pub status: Status,
}

impl Change {
    pub fn stage(&self, index: &mut Index) -> Result<()> {
        let path = bytes_to_path(&self.path);

        let is_deleted =
            matches!(self.status, Status::NonConflicting(status) if status.is_wt_deleted());

        if path.is_file() {
            if is_deleted {
                index.remove_path(path).with_context(|| {
                    let path = path.to_string_lossy();
                    format!("Failed to remove deleted file '{path}' from Git index")
                })?;
            } else {
                index.add_path(path).with_context(|| {
                    let path = path.to_string_lossy();
                    format!("Failed to add '{path}' to Git index")
                })?;
            }
        } else {
            let mut pathspec = self.path.clone();
            pathspec.push(b'*'); // Matches on everything under the given directory

            if is_deleted {
                index.remove_all([pathspec], None).with_context(|| {
                    let path = path.to_string_lossy();
                    format!("Failed to remove deleted directory '{path}' from Git index")
                })?;
            } else {
                index
                    .add_all([pathspec], IndexAddOption::default(), None)
                    .with_context(|| {
                        let path = path.to_string_lossy();
                        format!("Failed to add directory '{path}' to Git index")
                    })?;
            }
        }

        Ok(())
    }

    pub fn unstage(&self, index: &mut Index, repository_head_tree: &Tree) -> Result<()> {
        let path = bytes_to_path(&self.path);

        if matches!(self.status, Status::NonConflicting(status) if status.is_index_new()) {
            index.remove_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to remove '{path}' from Git index")
            })?;
        } else {
            // Unstaging changes to a previously added file involves:
            // 1. Getting the "tree entry" for the file in the HEAD tree of the repository
            //    (i.e. the current state of the file)
            // 2. Creating a new "index entry" from that tree entry and adding it to the Git index

            let tree_entry = repository_head_tree.get_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to get tree entry for '{path}' from HEAD tree in repository")
            })?;

            let index_entry = new_index_entry(
                tree_entry.id(),
                tree_entry.filemode() as u32,
                self.path.clone(),
            );

            index.add(&index_entry).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to restore '{path}' from Git index to HEAD version")
            })?;
        }

        Ok(())
    }
}

fn get_statuses(repository: &Repository) -> Result<Statuses> {
    let mut options = StatusOptions::default();
    options.include_ignored(false);
    options.include_untracked(true);

    repository
        .statuses(Some(&mut options))
        .context("Failed to get change statuses for repository")
}

fn get_repository_head_tree(repository: &Repository) -> Result<Tree> {
    let head = repository
        .head()
        .context("Failed to get HEAD reference from repository")?;

    let tree = head
        .peel_to_tree()
        .context("Failed to get file tree from HEAD reference in repository")?;

    Ok(tree)
}
