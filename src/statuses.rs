#[derive(PartialEq, Eq, Hash, Debug)]
pub(super) enum Status {
    NonConflicting(git2::Status),
    Conflicting {
        ours: ConflictingStatus,
        theirs: ConflictingStatus,
    },
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub(super) enum ConflictingStatus {
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

pub(super) const CONFLICTING_STATUSES: [[ConflictingStatus; 2]; 7] = [
    [ConflictingStatus::Unmerged, ConflictingStatus::Unmerged],
    [ConflictingStatus::Deleted, ConflictingStatus::Unmerged],
    [ConflictingStatus::Unmerged, ConflictingStatus::Deleted],
    [ConflictingStatus::Deleted, ConflictingStatus::Deleted],
    [ConflictingStatus::Added, ConflictingStatus::Unmerged],
    [ConflictingStatus::Unmerged, ConflictingStatus::Added],
    [ConflictingStatus::Added, ConflictingStatus::Added],
];
