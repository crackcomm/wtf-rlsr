//! Diff structure and utilities.

use std::path::PathBuf;

/// Git diff structure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diff {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub changed_files: Vec<PathBuf>,
    pub deleted_files: Vec<PathBuf>,
}

impl Diff {
    /// Returns true if no files were changed.
    pub fn is_empty(&self) -> bool {
        self.files_changed == 0
    }
}

impl From<git2::Diff<'_>> for Diff {
    fn from(diff: git2::Diff<'_>) -> Diff {
        let stats = diff.stats().unwrap();
        let deleted_files = diff
            .deltas()
            .map(|delta| delta.new_file())
            .filter(|file| file.size() == 0)
            .filter_map(|file| Some(file.path()?.to_path_buf()))
            .collect();
        let changed_files = diff
            .deltas()
            .map(|delta| delta.new_file())
            .filter(|file| file.size() > 0)
            .filter_map(|file| Some(file.path()?.to_path_buf()))
            .collect();
        Diff {
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
            deleted_files,
            changed_files,
        }
    }
}
