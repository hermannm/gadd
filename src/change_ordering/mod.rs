use std::{cmp::Ordering, collections::HashMap};

use crate::change_list::Change;

use self::status_priorities::StatusPriorityMap;

mod status_priorities;

pub(super) struct ChangeOrdering {
    map: HashMap<Vec<u8>, i64>,
}

impl ChangeOrdering {
    pub fn sort_changes_and_save_ordering(changes: &mut Vec<Change>) -> ChangeOrdering {
        changes.reverse();

        let status_priorities = StatusPriorityMap::new();

        changes.sort_by(|change_1, change_2| {
            status_priorities.compare_statuses(&change_1.status, &change_2.status)
        });

        let mut ordering = ChangeOrdering {
            map: HashMap::with_capacity(changes.len()),
        };

        for (change, i) in changes.iter().zip(0i64..) {
            ordering.map.insert(change.path.clone(), i);
        }

        ordering
    }

    pub fn sort_changes(&mut self, changes: &mut [Change]) -> usize {
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
                    Ordering::Greater
                }
                (None, Some(_)) => {
                    if !new_paths.contains(&change_1.path) {
                        new_paths.push(change_1.path.clone());
                    }
                    Ordering::Less
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

        let new_paths_length = new_paths.len();

        if !new_paths.is_empty() {
            new_paths.sort_by(|path_1, path_2| ChangeOrdering::compare_paths(path_1, path_2));

            let mut priority = self.map.values().min().cloned().unwrap_or(-1);

            for new_path in new_paths {
                self.map.insert(new_path, priority);
                priority -= 1;
            }
        }

        new_paths_length
    }

    fn compare_paths(path_1: &[u8], path_2: &[u8]) -> Ordering {
        let name_1 = String::from_utf8_lossy(path_1);
        let name_2 = String::from_utf8_lossy(path_2);
        name_1.cmp(&name_2)
    }
}
