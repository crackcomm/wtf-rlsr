use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks};

/// Default git remote callbacks.
pub(crate) fn default_remote_callbacks() -> RemoteCallbacks<'static> {
    let mut remote_callbacks = RemoteCallbacks::new();
    remote_callbacks.transfer_progress(|progress| {
        if progress.received_objects() % 1000 == 0 {
            trace!(
                "Progress: received {}/{} total objects",
                progress.received_objects(),
                progress.total_objects()
            );
        }
        true
    });
    let homedir = dirs::home_dir().unwrap();
    let ssh_pub = homedir.join(".ssh").join("id_rsa.pub");
    let ssh_priv = homedir.join(".ssh").join("id_rsa");
    trace!("SSH Public key location: {:?}", ssh_pub);
    trace!("SSH Private key location: {:?}", ssh_priv);
    remote_callbacks
        .credentials(move |_, _, _| Cred::ssh_key("git", Some(&ssh_pub), &ssh_priv, None));
    remote_callbacks
}

/// Default git push options.
pub(crate) fn default_push_options() -> PushOptions<'static> {
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(default_remote_callbacks());
    // if set to 0 the packbuilder will auto-detect the number of
    // threads to create and the default value is 1.
    push_opts.packbuilder_parallelism(0);
    push_opts
}

/// Default git fetch options.
pub(crate) fn default_fetch_options() -> FetchOptions<'static> {
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(default_remote_callbacks());
    fetch_opts
}
