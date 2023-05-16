use git2::Status;

use crate::statuses::{INDEX_STATUSES, STATUSES_LENGTH, WORKTREE_STATUSES};

const STATUS_SYMBOLS: [&str; STATUSES_LENGTH] = ["M", "T", "R", "D", "A"];

pub(super) struct StatusText {
    pub green_text: Option<&'static str>,
    pub red_text: Option<&'static str>,
}

impl From<&Status> for StatusText {
    fn from(status: &Status) -> Self {
        let mut status_text = StatusText {
            green_text: None,
            red_text: None,
        };

        if status == &Status::WT_NEW {
            status_text.red_text = Some("??");
        } else if status.is_conflicted() {
            status_text.red_text = Some("UU");
        } else {
            for (i, status_to_check) in INDEX_STATUSES.into_iter().enumerate() {
                if status.intersects(status_to_check) {
                    status_text.green_text = Some(STATUS_SYMBOLS[i]);
                }
            }

            for (j, status_to_check) in WORKTREE_STATUSES.into_iter().enumerate() {
                if status.intersects(status_to_check) {
                    status_text.red_text = Some(STATUS_SYMBOLS[j]);
                }
            }
        }

        status_text
    }
}
