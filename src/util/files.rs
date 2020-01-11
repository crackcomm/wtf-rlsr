use std::path::Path;

use failure::Error;

use crate::util::CleanPath;

/// Copies file from source to destination.
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> Result<u64, Error> {
    let src = src.as_ref();
    let dest = dest.as_ref();
    trace!(
        "Copying file from {:?} to {:?}",
        src.clean_path(),
        dest.clean_path()
    );
    ensure_dir_exists(&dest)?;
    std::fs::copy(src, dest).map_err(|err| {
        format_err!(
            "Copying file from {:?} to {:?} Error: {:?}",
            src.clean_path(),
            dest.clean_path(),
            err
        )
    })
}

/// Renames a file from source to destination.
/// Replaces the destination file if it exists.
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dest: Q) -> Result<(), Error> {
    let src = src.as_ref();
    let dest = dest.as_ref();
    trace!(
        "Renaming file from {:?} to {:?}",
        src.clean_path(),
        dest.clean_path()
    );
    std::fs::rename(src, dest).map_err(|err| {
        format_err!(
            "Renaming file from {:?} to {:?} Error: {:?}",
            src.clean_path(),
            dest.clean_path(),
            err
        )
    })
}

/// Removes a file from the file system.
pub fn remove_file<P: AsRef<Path>>(src: P) -> Result<(), Error> {
    let src = src.as_ref();
    trace!("Removing file {:?}", src.clean_path());
    std::fs::remove_file(src)
        .map_err(|e| format_err!("Removing file {:?} error: {:?}", src.clean_path(), e))
}

/// Ensures directory exists for file path.
pub fn ensure_dir_exists<P: AsRef<Path>>(filepath: P) -> Result<(), Error> {
    // Get file directory path
    if let Some(dirpath) = filepath.as_ref().parent() {
        // Create directory if doesn't exist
        if !dirpath.exists() {
            trace!("Creating directory {}", dirpath.display());
            std::fs::create_dir_all(dirpath)?;
        }
    }
    Ok(())
}
