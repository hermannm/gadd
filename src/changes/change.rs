use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use arboard::Clipboard;
use git2::{Index, IndexAddOption, IndexEntry, IndexTime, Oid, Tree};
use wsl::is_wsl;

use crate::statuses::Status;

pub(crate) struct Change {
    pub path: Vec<u8>,
    pub status: Status,
}

impl Change {
    pub fn stage(&self, index: &mut Index) -> Result<()> {
        let path = bytes_to_path(&self.path);

        let is_deleted =
            matches!(self.status, Status::NonConflicting(status) if status.is_wt_deleted());

        if path.is_file() {
            if is_deleted {
                index.remove_path(path).with_context(|| {
                    let path = path.to_string_lossy();
                    format!("Failed to remove deleted file '{path}' from Git index")
                })?;
            } else {
                index.add_path(path).with_context(|| {
                    let path = path.to_string_lossy();
                    format!("Failed to add '{path}' to Git index")
                })?;
            }
        } else {
            let mut pathspec = self.path.clone();
            pathspec.push(b'*'); // Matches on everything under the given directory

            if is_deleted {
                index.remove_all([pathspec], None).with_context(|| {
                    let path = path.to_string_lossy();
                    format!("Failed to remove deleted directory '{path}' from Git index")
                })?;
            } else {
                index
                    .add_all([pathspec], IndexAddOption::default(), None)
                    .with_context(|| {
                        let path = path.to_string_lossy();
                        format!("Failed to add directory '{path}' to Git index")
                    })?;
            }
        }

        Ok(())
    }

    pub fn unstage(&self, index: &mut Index, repository_head_tree: &Tree) -> Result<()> {
        let path = bytes_to_path(&self.path);

        if matches!(self.status, Status::NonConflicting(status) if status.is_index_new()) {
            index.remove_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to remove '{path}' from Git index")
            })?;
        } else {
            // Unstaging changes to a previously added file involves:
            // 1. Getting the "tree entry" for the file in the HEAD tree of the repository
            //    (i.e. the current state of the file)
            // 2. Creating a new "index entry" from that tree entry and adding it to the Git index

            let tree_entry = repository_head_tree.get_path(path).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to get tree entry for '{path}' from HEAD tree in repository")
            })?;

            let index_entry = new_index_entry(
                tree_entry.id(),
                tree_entry.filemode() as u32,
                self.path.clone(),
            );

            index.add(&index_entry).with_context(|| {
                let path = path.to_string_lossy();
                format!("Failed to restore '{path}' from Git index to HEAD version")
            })?;
        }

        Ok(())
    }

    pub fn copy_path_to_clipboard(&self) -> Result<()> {
        if is_wsl() {
            let mut clip_exe = Command::new("clip.exe")
                .stdin(Stdio::piped())
                .spawn()
                .context("Failed to launch clip.exe")?;

            let mut stdin = clip_exe
                .stdin
                .take()
                .context("Failed to get standard input handle from clip.exe")?;

            stdin
                .write_all(&self.path)
                .context("Failed to write to standard input of clip.exe")?;
        } else {
            let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;

            let path = std::str::from_utf8(&self.path).context(
                "Selected path is non-UTF8, not supported by this clipboard implementation",
            )?;

            clipboard
                .set_text(path)
                .context("Failed to set text of clipboard")?;
        }

        Ok(())
    }
}

/// From git2 crate: https://docs.rs/git2/0.17.1/src/git2/util.rs.html#86
#[cfg(unix)]
fn bytes_to_path(bytes: &[u8]) -> &Path {
    use std::{ffi::OsStr, os::unix::prelude::*};
    Path::new(OsStr::from_bytes(bytes))
}

/// From git2 crate: https://docs.rs/git2/0.17.1/src/git2/util.rs.html#91
#[cfg(windows)]
fn bytes_to_path(bytes: &[u8]) -> &Path {
    Path::new(std::str::from_utf8(bytes).unwrap())
}

fn new_index_entry(id: Oid, mode: u32, path: Vec<u8>) -> IndexEntry {
    IndexEntry {
        id,
        mode,
        path,
        ctime: IndexTime::new(0, 0),
        mtime: IndexTime::new(0, 0),
        dev: 0,
        ino: 0,
        uid: 0,
        gid: 0,
        file_size: 0,
        flags: 0,
        flags_extended: 0,
    }
}
