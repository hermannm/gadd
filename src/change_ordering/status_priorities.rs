use std::{cmp::Ordering, collections::HashMap};

use git2::Status;

use crate::statuses::{INDEX_STATUSES, WORKTREE_STATUSES};

pub(super) struct StatusPriorityMap {
    map: HashMap<Status, usize>,
}

impl StatusPriorityMap {
    pub fn new() -> StatusPriorityMap {
        let status_length = INDEX_STATUSES.len();

        let worktree_base_priority = (1 + status_length) * status_length;
        let conflicted_base_priority = worktree_base_priority + status_length;
        let not_added_priority = conflicted_base_priority * 2;

        let capacity = not_added_priority + 1;
        let mut map = HashMap::<Status, usize>::with_capacity(capacity);

        for i in 0..status_length {
            let index_status = INDEX_STATUSES[i];
            let worktree_status = WORKTREE_STATUSES[i];

            map.insert(index_status, i);
            map.insert(worktree_status, worktree_base_priority + i);

            map.insert(
                index_status | Status::CONFLICTED,
                conflicted_base_priority + i,
            );
            map.insert(
                worktree_status | Status::CONFLICTED,
                conflicted_base_priority + worktree_base_priority + i,
            );

            for (j, worktree_status_2) in WORKTREE_STATUSES.into_iter().enumerate() {
                let combined_status = worktree_status_2 | index_status;
                let priority = (i + 1) * status_length + j;
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
