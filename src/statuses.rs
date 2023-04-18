use std::{cmp::Ordering, collections::HashMap};

use git2::Status;

pub(super) const INDEX_STATUSES: [Status; 5] = [
    Status::INDEX_MODIFIED,
    Status::INDEX_NEW,
    Status::INDEX_RENAMED,
    Status::INDEX_TYPECHANGE,
    Status::INDEX_DELETED,
];

pub(super) const WORKTREE_STATUSES: [Status; 5] = [
    Status::WT_MODIFIED,
    Status::WT_NEW,
    Status::WT_RENAMED,
    Status::WT_TYPECHANGE,
    Status::WT_DELETED,
];

pub(super) fn get_status_symbol(
    status: Status,
    statuses_to_check: [Status; 5],
) -> Option<&'static str> {
    let status_symbols = ["M", "A", "R", "T", "D"];

    for (i, status_to_check) in statuses_to_check.into_iter().enumerate() {
        if status.intersects(status_to_check) {
            return Some(status_symbols[i]);
        }
    }

    None
}

pub(super) struct StatusPriorityMap {
    map: HashMap<Status, usize>,
}

impl StatusPriorityMap {
    pub fn new() -> StatusPriorityMap {
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

    pub fn compare_statuses(&self, status_1: &Status, status_2: &Status) -> Ordering {
        let priority_1 = self.map[status_1];
        let priority_2 = self.map[status_2];
        priority_1.cmp(&priority_2)
    }
}
