use std::path::Path;

use structopt::StructOpt;

use crate::{util::CleanPath, Opt};

/// Initializes options also sets cargo target directory.
pub fn setup_opt() -> std::io::Result<Opt> {
    pretty_env_logger::init();
    cargo::core::features::enable_nightly_features();
    let mut opt = Opt::from_args();
    if opt.directory.to_str().unwrap() == "." {
        opt.directory = std::env::current_dir()?.fix_path();
    } else {
        opt.directory = std::fs::canonicalize(&opt.directory)?.fix_path();
    }
    trace!("Workspace dir: {:?}", opt.directory);
    trace!("Cache dir: {:?}", opt.cache_dir);
    Ok(opt)
}

/// Sets cargo target directory environment variable.
pub fn set_cargo_workdir<P: AsRef<Path>>(dir: P) -> std::io::Result<()> {
    let cargo_dir = dir.as_ref().join("target").fix_path();
    std::env::set_var("CARGO_TARGET_DIR", &cargo_dir);
    trace!("CARGO_TARGET_DIR: {:?}", cargo_dir);
    Ok(())
}

/// Sets current working directory.
pub fn set_cwd<P: AsRef<Path>>(dir: P) -> std::io::Result<()> {
    std::env::set_current_dir(&dir)?;
    trace!("Current working directory: {:?}", std::env::current_dir()?);
    Ok(())
}
