use std::{ffi::OsStr, path::Path};

/// From git2 crate: https://docs.rs/git2/0.17.1/src/git2/util.rs.html#86
#[cfg(unix)]
pub(super) fn bytes_to_path(bytes: &[u8]) -> &Path {
    use std::os::unix::prelude::*;
    Path::new(OsStr::from_bytes(bytes))
}

/// From git2 crate: https://docs.rs/git2/0.17.1/src/git2/util.rs.html#91
#[cfg(windows)]
pub(super) fn bytes_to_path(bytes: &[u8]) -> &Path {
    use std::str;
    Path::new(str::from_utf8(bytes).unwrap())
}
