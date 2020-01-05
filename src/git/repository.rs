//! Git repository wrapper module.

use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use hashbrown::HashMap;

use cargo::core::Package;
use git2::{DiffOptions, Error, Repository as GitRepository};

use crate::util::{source_git_diff_paths, CleanPath};

use super::Diff;

/// Workspace Git repository structure.
pub struct Repository {
    inner: GitRepository,
    cache: HashMap<String, Diff>,
}

impl Repository {
    /// Creates new Git repository structure.
    pub fn new() -> Result<Self, Error> {
        let inner = GitRepository::open_from_env()?;
        let cache = HashMap::default();
        Ok(Repository { inner, cache })
    }

    /// Attempt to open an already-existing repository at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let inner = GitRepository::open(path)?;
        let cache = HashMap::default();
        Ok(Repository { inner, cache })
    }

    /// Returns HEAD commit.
    pub fn head_commit(&self) -> Result<git2::Commit<'_>, Error> {
        let head_oid = self.head()?.target().unwrap();
        self.find_commit(head_oid)
    }

    /// Returns HEAD tree.
    pub fn head_tree(&self) -> Result<git2::Tree<'_>, Error> {
        self.head_commit()?.tree()
    }

    /// Returns HEAD revision id.
    pub fn head_rev_id(&self) -> Result<String, Error> {
        let mut rev_id = self.head()?.target().unwrap().to_string();
        rev_id.truncate(8);
        Ok(rev_id)
    }

    /// Creates a diff for a package.
    pub fn diff(&mut self, pkg: &Package) -> Result<&Diff, Error> {
        let pkg_name = pkg.name().to_string();
        if !self.cache.contains_key(&pkg_name) {
            let mut diff_opts = DiffOptions::new();
            diff_opts.include_untracked(true);
            diff_opts.recurse_untracked_dirs(true);
            for path in source_git_diff_paths(pkg) {
                diff_opts.pathspec(self.rel_path(&path));
            }
            let diff = self
                .diff_index_to_workdir(None, Some(&mut diff_opts))?
                .into();
            self.cache.insert(pkg_name.to_owned(), diff);
        }
        Ok(self.cache.get(&pkg_name).unwrap())
    }

    /// Gets cached diff for a package.
    pub fn cached_diff(&mut self, name: &str) -> Option<&Diff> {
        self.cache.get(name)
    }

    /// Creates path relative to git repo root.
    pub fn rel_path(&self, path: &Path) -> PathBuf {
        match path.strip_prefix(self.workdir().unwrap()) {
            Ok(res) => res.clean_path(),
            Err(_) => path.clean_path(),
        }
    }

    /// Gets contents of a file on HEAD.
    pub fn get_contents(&self, tree: &git2::Tree, path: &Path) -> Result<Vec<u8>, Error> {
        let path = self.rel_path(path);
        let entry = tree.get_path(&path)?;
        let entry_object = entry.to_object(&self)?;
        let entry_blob = entry_object.into_blob().unwrap();
        Ok(entry_blob.content().to_owned())
    }
}

impl Deref for Repository {
    type Target = GitRepository;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Repository {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
