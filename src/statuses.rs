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
