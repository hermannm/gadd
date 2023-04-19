use std::{cmp::Ordering, collections::HashMap};

use git2::Status;

use crate::{
    change_list::Change,
    statuses::{INDEX_STATUSES, WORKTREE_STATUSES},
};

pub(super) struct ChangeOrdering {
    map: HashMap<Vec<u8>, usize>,
    status_priorities: StatusPriorityMap,
}

impl ChangeOrdering {
    pub fn sort_changes_and_save_ordering(changes: &mut Vec<Change>) -> ChangeOrdering {
        let mut ordering = ChangeOrdering {
            map: HashMap::with_capacity(changes.len()),
            status_priorities: StatusPriorityMap::new(),
        };

        ordering.sort_changes_by_status(changes);

        for (i, change) in changes.iter().enumerate() {
            ordering.map.insert(change.path.clone(), i);
        }

        ordering
    }

    pub fn sort_changes_by_status(&self, changes: &mut [Change]) {
        changes.sort_by(|change_1, change_2| {
            self.status_priorities
                .compare_statuses(&change_1.status, &change_2.status)
        });
    }

    pub fn sort_changes(&mut self, changes: &mut [Change]) {
        let mut new_paths = Vec::<Vec<u8>>::new();

        changes.sort_by(|change_1, change_2| {
            let order_1 = self.map.get(&change_1.path);
            let order_2 = self.map.get(&change_2.path);

            match (order_1, order_2) {
                (Some(order_1), Some(order_2)) => order_1.cmp(order_2),
                (Some(_), None) => {
                    if !new_paths.contains(&change_2.path) {
                        new_paths.push(change_2.path.clone());
                    }
                    Ordering::Less
                }
                (None, Some(_)) => {
                    if !new_paths.contains(&change_1.path) {
                        new_paths.push(change_1.path.clone());
                    }
                    Ordering::Greater
                }
                (None, None) => {
                    if !new_paths.contains(&change_1.path) {
                        new_paths.push(change_1.path.clone());
                    }
                    if !new_paths.contains(&change_2.path) {
                        new_paths.push(change_2.path.clone());
                    }
                    ChangeOrdering::compare_paths(&change_1.path, &change_2.path)
                }
            }
        });

        if !new_paths.is_empty() {
            new_paths.sort_by(ChangeOrdering::compare_paths);

            let ordering_length = self.map.len();

            for (i, new_path) in new_paths.into_iter().enumerate() {
                self.map.insert(new_path, ordering_length + i);
            }
        }
    }

    #[allow(clippy::ptr_arg)]
    fn compare_paths(path_1: &Vec<u8>, path_2: &Vec<u8>) -> Ordering {
        let name_1 = String::from_utf8_lossy(path_1);
        let name_2 = String::from_utf8_lossy(path_2);
        name_1.cmp(&name_2)
    }
}

struct StatusPriorityMap {
    map: HashMap<Status, usize>,
}

impl StatusPriorityMap {
    fn new() -> StatusPriorityMap {
        let status_length = INDEX_STATUSES.len();

        let capacity = status_length * 4 + (2 * status_length.pow(2)) + 1;
        let mut map = HashMap::<Status, usize>::with_capacity(capacity);

        let not_added_priority = (2 + status_length) * status_length;
        let conflicted_base_priority = not_added_priority + 1;

        for i in 0..status_length {
            let index_status = INDEX_STATUSES[i];
            let worktree_status = WORKTREE_STATUSES[i];

            map.insert(index_status, i);
            map.insert(worktree_status, i + status_length);

            map.insert(
                index_status | Status::CONFLICTED,
                conflicted_base_priority + i,
            );
            map.insert(
                worktree_status | Status::CONFLICTED,
                conflicted_base_priority + i + status_length,
            );

            for (j, index_status_2) in INDEX_STATUSES.into_iter().enumerate() {
                let combined_status = index_status_2 | worktree_status;
                let priority = (i + 2) * status_length + j;
                map.insert(combined_status, priority);

                map.insert(
                    combined_status | Status::CONFLICTED,
                    conflicted_base_priority + priority,
                );
            }
        }

        map.insert(Status::WT_NEW, not_added_priority);

        StatusPriorityMap { map }
    }

    fn compare_statuses(&self, status_1: &Status, status_2: &Status) -> Ordering {
        let priority_1 = self.map[status_1];
        let priority_2 = self.map[status_2];
        priority_1.cmp(&priority_2)
    }
}
