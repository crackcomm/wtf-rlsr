use std::path::{Path, PathBuf};

use structopt::StructOpt;

use crate::{cmd::Opt, util::CleanPath};

/// Wtf-rlsr workspace configuration structure.
#[derive(Deserialize)]
struct Config {
    cache: PathBuf,
}

impl Config {
    fn merge_into(self, opt: &mut Opt) {
        if opt.cache_dir.to_str().unwrap() == "../cache" {
            opt.cache_dir = self.cache;
        }
    }
}

/// Initializes options also sets cargo target directory.
pub fn setup_opt() -> Result<Opt, failure::Error> {
    pretty_env_logger::init();
    cargo::core::features::enable_nightly_features();
    let mut opt = Opt::from_args();
    if let Some(config) = read_config()? {
        config.merge_into(&mut opt);
    }
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

fn read_config() -> Result<Option<Config>, failure::Error> {
    let cfg_path: PathBuf = ".wtf-rlsr.json".into();
    if cfg_path.exists() {
        let contents = std::fs::read_to_string(&cfg_path)?;
        let config = serde_json::from_str(&contents)?;
        Ok(Some(config))
    } else {
        Ok(None)
    }
}
