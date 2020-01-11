use git2::{
    build::CheckoutBuilder, AnnotatedCommit, AutotagOption, Error, FetchOptions, Reference, Remote,
    Repository,
};

use super::default_push_options;

/// Attempts to do `git push` to a remote.
pub fn push_remote(repo: &Repository, remote: &str, refspecs: &[&str]) -> Result<(), Error> {
    let mut remote = repo.find_remote(remote)?;
    let mut push_opts = default_push_options();
    remote.push(refspecs, Some(&mut push_opts))
}

/// Attempts to do `git pull` from a remote.
pub fn pull_remote(
    repo: &Repository,
    remote_branch: &str,
    remote: &mut Remote,
) -> Result<(), Error> {
    let fetch_commit = fetch_remote(&repo, remote_branch, remote)?;
    merge_commit(&repo, remote_branch, fetch_commit)
}

fn fetch_remote<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    remote: &mut Remote,
) -> Result<AnnotatedCommit<'a>, Error> {
    let mut fo = FetchOptions::new();
    // Always fetch all tags.
    // Perform a download and also update tips
    fo.download_tags(AutotagOption::Auto);
    trace!("Fetching {} from {}", remote_branch, remote.name().unwrap());
    remote.fetch(&[&remote_branch], Some(&mut fo), None)?;

    // If there are local objects (we got a thin pack), then tell the user
    // how many objects we saved from having to cross the network.
    let stats = remote.stats();
    if stats.local_objects() > 0 {
        trace!(
            "Received {}/{} objects in {} bytes (used {} local \
             objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        trace!(
            "Received {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    trace!("Fetched head: {:?}", fetch_head.target());
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn fast_forward(repo: &Repository, lb: &mut Reference, rc: &AnnotatedCommit) -> Result<(), Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    trace!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &AnnotatedCommit,
    remote: &AnnotatedCommit,
) -> Result<(), Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        trace!("Merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

fn merge_commit(
    repo: &Repository,
    remote_branch: &str,
    fetch_commit: AnnotatedCommit<'_>,
) -> Result<(), Error> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    // 2. Do the appopriate merge
    if analysis.0.is_fast_forward() {
        trace!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    } else {
        trace!("Nothing to do...");
    }
    Ok(())
}
