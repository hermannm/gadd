use std::{cmp::Ordering, collections::HashMap, fmt::Debug};

use crate::statuses::{
    Status, CONFLICTING_STATUSES, INDEX_STATUSES, STATUSES_LENGTH, WORKTREE_STATUSES,
};

pub(super) struct StatusPriorityMap {
    map: HashMap<Status, usize>,
}

impl StatusPriorityMap {
    pub fn new() -> StatusPriorityMap {
        let worktree_base_priority = (1 + STATUSES_LENGTH) * STATUSES_LENGTH;
        let conflicting_priority = worktree_base_priority + STATUSES_LENGTH;

        let mut map = HashMap::<Status, usize>::with_capacity(
            conflicting_priority + CONFLICTING_STATUSES.len(),
        );

        for i in 0..STATUSES_LENGTH {
            let index_status = INDEX_STATUSES[i];
            let worktree_status = WORKTREE_STATUSES[i];

            map.insert(Status::NonConflicting(index_status), i);
            map.insert(
                Status::NonConflicting(worktree_status),
                worktree_base_priority + i,
            );

            for (j, worktree_status_2) in WORKTREE_STATUSES.into_iter().enumerate() {
                let combined_status = worktree_status_2 | index_status;
                let priority = (i + 1) * STATUSES_LENGTH + j;
                map.insert(Status::NonConflicting(combined_status), priority);
            }
        }

        for (i, [ours, theirs]) in CONFLICTING_STATUSES.into_iter().enumerate() {
            map.insert(
                Status::Conflicting { ours, theirs },
                conflicting_priority + i,
            );
        }

        StatusPriorityMap { map }
    }

    pub fn compare_statuses(&self, status_1: &Status, status_2: &Status) -> Ordering {
        let priority_1 = self.map.get(status_1);
        let priority_2 = self.map.get(status_2);
        priority_1.cmp(&priority_2)
    }
}

impl Debug for StatusPriorityMap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut status_priorities: Vec<(&Status, &usize)> = self.map.iter().collect();
        status_priorities.sort_by(|(_, priority_1), (_, priority_2)| priority_1.cmp(priority_2));

        formatter.write_str("StatusPriorityMap {\n")?;

        for (status, priority) in status_priorities {
            formatter.write_fmt(format_args!("    {status:?}: {priority:?},\n"))?;
        }

        formatter.write_str("}")?;

        Ok(())
    }
}
