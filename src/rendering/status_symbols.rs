use git2::Status;

use crate::statuses::{INDEX_STATUSES, STATUSES_LENGTH, WORKTREE_STATUSES};

const STATUS_SYMBOLS: [&str; STATUSES_LENGTH] = ["M", "T", "R", "D", "A"];

pub(super) enum StatusSymbol {
    Green(&'static str),
    Red(&'static str),
    Space,
}

pub(super) fn get_status_symbols(status: Status) -> [StatusSymbol; 2] {
    use StatusSymbol::*;

    if status == Status::WT_NEW {
        return [Red("?"), Red("?")];
    }

    if status.is_conflicted() {
        return [Red("U"), Red("U")];
    }

    let mut status_symbols = [Space, Space];

    for (i, status_to_check) in INDEX_STATUSES.into_iter().enumerate() {
        if status.intersects(status_to_check) {
            status_symbols[0] = Green(STATUS_SYMBOLS[i]);
            break;
        }
    }

    for (i, status_to_check) in WORKTREE_STATUSES.into_iter().enumerate() {
        if status.intersects(status_to_check) {
            status_symbols[1] = Red(STATUS_SYMBOLS[i]);
            break;
        }
    }

    status_symbols
}
