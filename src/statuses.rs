#[derive(PartialEq, Eq, Hash, Debug)]
pub(super) enum Status {
    NonConflicted(git2::Status),
    Conflicted {
        ours: ConflictedStatus,
        theirs: ConflictedStatus,
    },
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub(super) enum ConflictedStatus {
    Unmerged,
    Added,
    Deleted,
}

pub(super) const STATUSES_LENGTH: usize = 5;

pub(super) const INDEX_STATUSES: [git2::Status; STATUSES_LENGTH] = [
    git2::Status::INDEX_MODIFIED,
    git2::Status::INDEX_TYPECHANGE,
    git2::Status::INDEX_RENAMED,
    git2::Status::INDEX_DELETED,
    git2::Status::INDEX_NEW,
];

pub(super) const WORKTREE_STATUSES: [git2::Status; STATUSES_LENGTH] = [
    git2::Status::WT_MODIFIED,
    git2::Status::WT_TYPECHANGE,
    git2::Status::WT_RENAMED,
    git2::Status::WT_DELETED,
    git2::Status::WT_NEW,
];

pub(super) const CONFLICTED_STATUSES: [[ConflictedStatus; 2]; 7] = [
    [ConflictedStatus::Unmerged, ConflictedStatus::Unmerged],
    [ConflictedStatus::Deleted, ConflictedStatus::Unmerged],
    [ConflictedStatus::Unmerged, ConflictedStatus::Deleted],
    [ConflictedStatus::Deleted, ConflictedStatus::Deleted],
    [ConflictedStatus::Added, ConflictedStatus::Unmerged],
    [ConflictedStatus::Unmerged, ConflictedStatus::Added],
    [ConflictedStatus::Added, ConflictedStatus::Added],
];
