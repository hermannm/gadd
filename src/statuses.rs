use git2::Status;

const STATUSES_LENGTH: usize = 5;

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

const STATUS_SYMBOLS: [&str; STATUSES_LENGTH] = ["M", "T", "R", "D", "A"];

const CONFLICTED_STATUS_SYMBOLS: [&str; STATUSES_LENGTH] = ["U", "T", "R", "D", "A"];

pub(super) fn get_status_symbols(status: &Status) -> [Option<&'static str>; 2] {
    let symbol_alternatives: [&str; STATUSES_LENGTH];
    let mut status_symbols: [Option<&str>; 2];

    if status.is_conflicted() {
        symbol_alternatives = CONFLICTED_STATUS_SYMBOLS;
        status_symbols = [Some("U"), Some("U")];
    } else {
        symbol_alternatives = STATUS_SYMBOLS;
        status_symbols = [None, None];
    }

    for (i, statuses_to_check) in [INDEX_STATUSES, WORKTREE_STATUSES].into_iter().enumerate() {
        for (j, status_to_check) in statuses_to_check.into_iter().enumerate() {
            if status.intersects(status_to_check) {
                status_symbols[i] = Some(symbol_alternatives[j]);
            }
        }
    }

    status_symbols
}
