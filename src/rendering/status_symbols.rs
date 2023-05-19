use crate::statuses::{
    ConflictingStatus, Status, INDEX_STATUSES, STATUSES_LENGTH, WORKTREE_STATUSES,
};

pub(super) enum StatusSymbol {
    Green(&'static str),
    Red(&'static str),
    Space,
}

pub(super) fn get_status_symbols(status: &Status) -> [StatusSymbol; 2] {
    use StatusSymbol::*;

    match status {
        Status::NonConflicting(git2::Status::WT_NEW) => [Red("?"), Red("?")],
        Status::NonConflicting(status) => {
            const STATUS_SYMBOL_OPTIONS: [&str; STATUSES_LENGTH] = ["M", "T", "R", "D", "A"];

            let mut status_symbols = [Space, Space];

            for (i, status_to_check) in INDEX_STATUSES.into_iter().enumerate() {
                if status.intersects(status_to_check) {
                    status_symbols[0] = Green(STATUS_SYMBOL_OPTIONS[i]);
                    break;
                }
            }

            for (i, status_to_check) in WORKTREE_STATUSES.into_iter().enumerate() {
                if status.intersects(status_to_check) {
                    status_symbols[1] = Red(STATUS_SYMBOL_OPTIONS[i]);
                    break;
                }
            }

            status_symbols
        }
        Status::Conflicting { ours, theirs } => [ours, theirs].map(|status| match status {
            ConflictingStatus::Unmerged => Red("U"),
            ConflictingStatus::Added => Red("A"),
            ConflictingStatus::Deleted => Red("D"),
        }),
    }
}
