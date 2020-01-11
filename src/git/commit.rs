//! Workspace committing tools.

use std::path::Path;

use failure::Error;
use git2::{Index, Oid, Repository, Signature};

use crate::util::CleanPath;

/// Commit builder structure.
pub struct CommitBuilder {
    signature: Signature<'static>,
    index: Index,
}

impl CommitBuilder {
    pub fn new(repo: &Repository) -> Result<Self, Error> {
        let signature = repo.signature()?;
        trace!(
            "Commit builder: {} {}",
            signature.name().unwrap(),
            signature.email().unwrap()
        );
        let index = repo.index()?;
        Ok(CommitBuilder { signature, index })
    }

    /// Adds file path to git commit.
    pub fn add_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let path = path.as_ref().clean_path();
        trace!("Git add: {:?}", path);
        self.index.add_path(&path)?;
        Ok(())
    }

    /// Commits changes and sets detached HEAD.
    pub fn commit(&mut self, message: &str, repo: &mut Repository) -> Result<Oid, Error> {
        self.index.write()?;
        let tree_oid = self.index.write_tree_to(repo)?;
        let tree = repo.find_tree(tree_oid)?;
        let head_oid = repo.head()?.target().unwrap();
        let commit = repo.find_commit(head_oid)?;
        trace!("Creating new commit on prev: {:?}", commit);
        trace!("Commit message: {}", message);

        let new_commit = repo.commit(
            None,
            &self.signature,
            &self.signature,
            message,
            &tree,
            &[&commit],
        )?;
        repo.head()?.set_target(new_commit, "")?;
        Ok(new_commit)
    }
}
