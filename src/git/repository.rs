//! Git repository wrapper module.

use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use hashbrown::HashMap;

use cargo::core::Package;
use git2::{DiffOptions, Error, Repository as GitRepository};

use crate::util::{source_git_diff_paths, CleanPath};

use super::{default_fetch_options, Diff};

/// Workspace Git repository structure.
pub struct Repository {
    inner: GitRepository,
    cache: HashMap<String, Diff>,
}

impl Repository {
    /// Attempt to open an already-existing repository at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let inner = GitRepository::open(path)?;
        let cache = HashMap::default();
        Ok(Repository { inner, cache })
    }

    /// Attempt to clone repository recursively to `dest`.
    pub fn clone_recurse<P: AsRef<Path>>(source: &str, dest: P) -> Result<Self, Error> {
        let inner = GitRepository::clone(source, dest)?;
        let cache = HashMap::default();
        let repo = Repository { inner, cache };
        repo.update_submodules(true, false)?;
        Ok(repo)
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

    /// Creates a diff for a package.
    pub fn diff(&mut self, pkg: &Package) -> Result<&Diff, Error> {
        let pkg_name = pkg.name().to_string();
        if !self.cache.contains_key(&pkg_name) {
            let mut diff_opts = DiffOptions::new();
            diff_opts.include_untracked(true);
            diff_opts.recurse_untracked_dirs(true);
            for path in source_git_diff_paths(pkg) {
                trace!("Diff path: {}", self.rel_path(&path).clean_path().display());
                diff_opts.pathspec(self.rel_path(&path).clean_path());
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
            Ok(res) => res.fix_path(),
            Err(_) => path.fix_path(),
        }
    }

    /// Gets contents of a file on HEAD.
    pub fn get_contents(&self, tree: &git2::Tree, path: &Path) -> Result<Vec<u8>, Error> {
        let path = self.rel_path(path).clean_path();
        let entry = tree.get_path(&path)?;
        let entry_object = entry.to_object(&self)?;
        let entry_blob = entry_object.into_blob().unwrap();
        Ok(entry_blob.content().to_owned())
    }

    /// Update submodules recursively.
    pub fn update_submodules(&self, init: bool, deep: bool) -> Result<(), Error> {
        fn add_subrepos(
            repo: &git2::Repository,
            list: &mut Vec<git2::Repository>,
            init: bool,
        ) -> Result<(), Error> {
            for mut subm in repo.submodules()? {
                let mut opts = git2::SubmoduleUpdateOptions::new();
                trace!("Fetching submodule {:?}", subm.url().unwrap());
                opts.fetch(default_fetch_options());
                subm.update(init, Some(&mut opts))?;
                list.push(subm.open()?);
            }
            Ok(())
        }

        let mut repos = Vec::new();
        add_subrepos(self, &mut repos, init)?;
        if deep {
            while let Some(repo) = repos.pop() {
                add_subrepos(&repo, &mut repos, init)?;
            }
        }
        Ok(())
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
