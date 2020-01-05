use std::path::Path;

use fs_extra::dir::{copy as copy_dir, CopyOptions};

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

/// Copies directory recursively ignoring `target/` and some other files.
pub fn copy_files<P: AsRef<Path>>(files: Vec<P>, dest: P) -> std::io::Result<()> {
    let options = CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64_000,
        copy_inside: true,
        depth: 0,
    };
    for path in files {
        let fname = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        if is_excluded(&fname) {
            trace!("Excluded path: {:?}", path.as_ref());
        } else {
            trace!("Copying path: {:?}", path.as_ref());
            let meta = std::fs::metadata(&path)?;
            let dest_path = dest.as_ref().join(fname);
            if meta.is_dir() {
                copy_dir(path, dest_path, &options).unwrap();
            } else {
                std::fs::copy(path, dest_path)?;
            }
        }
    }
    Ok(())
}

fn is_excluded(fname: &str) -> bool {
    if fname.ends_with(".log") {
        true
    } else if fname.ends_with(".git") {
        true
    } else if fname.ends_with(".lock") {
        true
    } else if fname.ends_with(".dtf") {
        true
    } else if fname == "target" {
        true
    } else {
        false
    }
}
