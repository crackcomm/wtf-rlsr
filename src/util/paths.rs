//! Packages paths utilities.

use std::path::{Path, PathBuf};

use cargo::core::Package;
use glob::glob;

/// Creates a source code git diff paths for package root.
pub fn source_git_diff_paths(pkg: &Package) -> Vec<&Path> {
    pkg.targets()
        .iter()
        .filter_map(|target| target.src_path().path())
        .map(|path| path.parent().unwrap())
        .collect()
}

/// Creates a source code glob paths for package root.
pub fn source_glob_paths(pkg: &Package) -> Vec<PathBuf> {
    pkg.targets()
        .iter()
        .filter_map(|target| target.src_path().path())
        .map(|path| path.parent().unwrap().join("**").join("*.rs"))
        .collect()
}

/// Globs a list of source code files in a package root.
pub fn glob_source(pkg: &Package) -> Vec<PathBuf> {
    source_glob_paths(pkg)
        .into_iter()
        .map(|path| {
            trace!("Package glob: {:?}", path.to_str().unwrap());
            glob(path.to_str().unwrap())
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        })
        .flatten()
        .collect::<Vec<_>>()
}

/// Clean UNIX-like path trait.
pub trait CleanPath {
    fn clean_path(&self) -> PathBuf;
}

impl CleanPath for Path {
    fn clean_path(&self) -> PathBuf {
        self.to_path_buf()
            .into_os_string()
            .into_string()
            .unwrap()
            .replace("\\\\?\\", "")
            .replace(std::path::MAIN_SEPARATOR, "/")
            .into()
    }
}
