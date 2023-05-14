use std::{cmp::Ordering, collections::HashMap};

use git2::Status;

use crate::statuses::{INDEX_STATUSES, WORKTREE_STATUSES};

pub struct StatusPriorityMap {
    map: HashMap<Status, usize>,
}

impl StatusPriorityMap {
    pub fn new() -> StatusPriorityMap {
        let status_length = INDEX_STATUSES.len();

        let index_base_priority = (1 + status_length) * status_length;
        let no_conflicts_base_priority = index_base_priority + status_length;

        let mut map = HashMap::<Status, usize>::with_capacity(2 * no_conflicts_base_priority);

        for i in 0..status_length {
            let worktree_status = WORKTREE_STATUSES[i];
            let index_status = INDEX_STATUSES[i];

            map.insert(worktree_status, no_conflicts_base_priority + i);
            map.insert(
                index_status,
                no_conflicts_base_priority + index_base_priority + i,
            );

            map.insert(worktree_status | Status::CONFLICTED, i);
            map.insert(index_status | Status::CONFLICTED, index_base_priority + i);

            for (j, worktree_status_2) in WORKTREE_STATUSES.into_iter().enumerate() {
                let combined_status = worktree_status_2 | index_status;
                let priority = (i + 1) * status_length + j;
                map.insert(combined_status, no_conflicts_base_priority + priority);

                map.insert(combined_status | Status::CONFLICTED, priority);
            }
        }

        StatusPriorityMap { map }
    }

    pub fn compare_statuses(&self, status_1: &Status, status_2: &Status) -> Ordering {
        let priority_1 = self.map[status_1];
        let priority_2 = self.map[status_2];
        priority_1.cmp(&priority_2)
    }
}
