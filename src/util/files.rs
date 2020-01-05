use std::path::Path;

use crate::util::CleanPath;

/// Copies file from source to destination.
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> std::io::Result<u64> {
    trace!(
        "Copying file from {:?} to {:?}",
        src.as_ref().clean_path(),
        dest.as_ref().clean_path()
    );
    std::fs::copy(src, dest)
}

/// Renames a file from source to destination.
/// Replaces the destination file if it exists.
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> std::io::Result<()> {
    trace!(
        "Renaming file from {:?} to {:?}",
        src.as_ref().clean_path(),
        dest.as_ref().clean_path()
    );
    std::fs::rename(src, dest)
}

/// Removes a file from the file system.
pub fn remove_file<P: AsRef<Path>>(src: P) -> std::io::Result<()> {
    trace!("Removing file {:?}", src.as_ref().clean_path());
    std::fs::remove_file(src)
}
