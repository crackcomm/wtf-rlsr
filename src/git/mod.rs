//! Git wrappers and helpers module.

mod commit;
mod diff;
pub mod pull;
mod push;
mod repository;

pub use self::commit::*;
pub use self::diff::*;
pub use self::push::*;
pub use self::repository::*;

use std::path::Path;

use git2::{Error, Oid};

/// WTF RLS latest git commit tag.
pub const WTF_RLSR_TAG: &'static str = "refs/tags/wtf-rlsr-latest";

/// Creates a reference for the `HEAD` of the repository.
pub fn set_head_ref(refspec: &str, repo: &mut git2::Repository) -> Result<Oid, Error> {
    let head_oid = repo.head()?.target().unwrap();
    repo.reference(&refspec, head_oid, true, "")?;
    Ok(head_oid)
}

/// Initializes workspace git repository.
pub fn init_main_repo<P: AsRef<Path>>(path: P) -> Result<Repository, git2::Error> {
    let repo = Repository::open(path)?;
    // let head_oid = ensure_head_tag(&mut repo)?;
    // push_remote(&repo, &opt.remote, &[])?;
    Ok(repo)
}

// /// Ensures a `wtf-rlsr-latest` tag on `HEAD` of the repository.
// fn ensure_head_tag(repo: &mut git2::Repository) -> Result<Oid, Error> {
//     let wtf_rlsr_ref = repo.find_reference(WTF_RLSR_TAG)?;
//     let tag_oid = wtf_rlsr_ref.target().unwrap();
//     let head_oid = repo.head()?.target().unwrap();
//     if tag_oid != head_oid {
//         repo.reference(WTF_RLSR_TAG, head_oid, true, "")?;
//     }
//     Ok(head_oid)
// }

/// Initializes cached git repository.
pub fn init_cache_repo<P: AsRef<Path>, Q: AsRef<Path>>(
    path: P,
    source: Q,
    remote_branch: &str,
) -> Result<Repository, git2::Error> {
    if !path.as_ref().exists() {
        trace!("Cloning repository to {:?}", path.as_ref());
        let mut builder = git2::build::RepoBuilder::new();
        builder.clone_local(git2::build::CloneLocal::Local);
        builder.clone(source.as_ref().to_str().unwrap(), path.as_ref())?;
        let repo = Repository::open(path)?;
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
        let fetch_commit = pull::do_fetch(&repo, &remote_branch, &mut repo.find_remote("origin")?)?;
        pull::do_merge(&repo, &remote_branch, fetch_commit)?;
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
