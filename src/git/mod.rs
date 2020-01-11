//! Git wrappers and helpers module.

mod commit;
mod diff;
mod options;
mod repository;
mod util;

pub use self::commit::*;
pub use self::diff::*;
pub(crate) use self::options::*;
pub use self::repository::*;
pub use self::util::*;

use std::path::Path;

use git2::{Error, Oid};

/// Creates a reference for the `HEAD` of the repository.
pub fn set_head_ref(refspec: &str, repo: &mut git2::Repository) -> Result<Oid, Error> {
    let head_oid = repo.head()?.target().unwrap();
    repo.reference(&refspec, head_oid, true, "")?;
    Ok(head_oid)
}

/// Initializes cached git repository.
pub fn init_cache_repo<P: AsRef<Path>, Q: AsRef<Path>>(
    path: P,
    source: Q,
    remote_branch: &str,
) -> Result<Repository, git2::Error> {
    if !path.as_ref().exists() {
        trace!("Cloning repository to {:?}", path.as_ref());
        let repo = Repository::clone_recurse(source.as_ref().to_str().unwrap(), path)?;
        Ok(repo)
    } else {
        let mut repo = Repository::open(path.as_ref())?;
        let signature = repo.signature()?;
        repo.reset(repo.head_commit()?.as_object(), git2::ResetType::Hard, None)?;
        if let Err(err) = repo.stash_save(
            &signature,
            "cxmr-rlsr",
            Some(git2::StashFlags::INCLUDE_UNTRACKED),
        ) {
            trace!("Stash error: {:?}", err);
        }
        pull_remote(&repo, &remote_branch, &mut repo.find_remote("origin")?)?;
        Ok(repo)
    }
}

/// Attempts to get a repo branch name on the head.
pub fn get_head_branch(repo: &Repository) -> Result<String, git2::Error> {
    let branch_name = if let Some(head_branch) = repo
        .branches(Some(git2::BranchType::Local))?
        .filter_map(|b| b.ok().map(|(b, _)| b))
        .find(|b| b.is_head())
    {
        head_branch.name()?.unwrap().to_owned()
    } else {
        panic!("No branch set.");
    };
    trace!("Git head tag {:?} ", branch_name);
    Ok(branch_name)
}
