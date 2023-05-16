use std::{fs::File, path::Path};

use git2::{IndexEntry, IndexTime, Oid};

use crate::Stdout;

/// From git2 crate: https://docs.rs/git2/0.17.1/src/git2/util.rs.html#86
#[cfg(unix)]
pub(super) fn bytes_to_path(bytes: &[u8]) -> &Path {
    use std::{ffi::OsStr, os::unix::prelude::*};
    Path::new(OsStr::from_bytes(bytes))
}

/// From git2 crate: https://docs.rs/git2/0.17.1/src/git2/util.rs.html#91
#[cfg(windows)]
pub(super) fn bytes_to_path(bytes: &[u8]) -> &Path {
    Path::new(std::str::from_utf8(bytes).unwrap())
}

#[cfg(unix)]
pub(super) fn get_raw_stdout() -> Stdout {
    use std::os::unix::io::FromRawFd;

    unsafe { File::from_raw_fd(1) }
}

#[cfg(windows)]
pub(super) fn get_raw_stdout() -> Stdout {
    use kernel32::GetStdHandle;
    use std::os::windows::io::FromRawHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;

    unsafe { File::from_raw_handle(GetStdHandle(STD_OUTPUT_HANDLE)) }
}

pub(super) fn new_index_entry(id: Oid, mode: u32, path: Vec<u8>) -> IndexEntry {
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
