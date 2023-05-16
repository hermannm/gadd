use git2::Status;

pub(super) const STATUSES_LENGTH: usize = 5;

pub(super) const INDEX_STATUSES: [Status; STATUSES_LENGTH] = [
    Status::INDEX_MODIFIED,
    Status::INDEX_TYPECHANGE,
    Status::INDEX_RENAMED,
    Status::INDEX_DELETED,
    Status::INDEX_NEW,
];

pub(super) const WORKTREE_STATUSES: [Status; STATUSES_LENGTH] = [
    Status::WT_MODIFIED,
    Status::WT_TYPECHANGE,
    Status::WT_RENAMED,
    Status::WT_DELETED,
    Status::WT_NEW,
];
