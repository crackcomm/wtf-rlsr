use git2::{Cred, PushOptions, RemoteCallbacks, Repository};

/// Attempts to do `git push` to a remote.
pub fn push_remote(repo: &Repository, remote: &str, refspecs: &[&str]) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote(remote)?;
    let mut remote_callbacks = RemoteCallbacks::new();
    let homedir = dirs::home_dir().unwrap();
    let ssh_pub = homedir.join(".ssh").join("id_rsa.pub");
    let ssh_priv = homedir.join(".ssh").join("id_rsa");
    trace!("SSH Public key location: {:?}", ssh_pub);
    trace!("SSH Private key location: {:?}", ssh_priv);
    remote_callbacks
        .credentials(move |_, _, _| Cred::ssh_key("git", Some(&ssh_pub), &ssh_priv, None));
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(remote_callbacks);
    // if set to 0 the packbuilder will auto-detect the number of
    // threads to create and the default value is 1.
    push_opts.packbuilder_parallelism(0);
    remote.push(refspecs, Some(&mut push_opts))
}
