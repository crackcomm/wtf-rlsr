//! Command execution.

use std::path::PathBuf;

use cargo::core::Workspace as CargoWorkspace;
use failure::ResultExt;

use crate::{
    git::{self, Repository},
    util::{self, CleanPath},
    ws::{self, Workspace},
};

use super::{release, update_paths, Command, Opt};

/// Command execution runtime structure.
pub struct ExecRuntime<'a> {
    pub repo: Repository,
    pub cache_dir: PathBuf,
    pub cargo: &'a CargoWorkspace<'a>,
    pub workspace: Workspace<'a>,
    pub head_branch: String,
    pub directory: PathBuf,
}

impl<'a> ExecRuntime<'a> {
    pub fn open_cache_repo(&mut self) -> Result<Repository, git2::Error> {
        git::init_cache_repo(&self.cache_dir, &self.directory, &self.head_branch)
    }
}

/// Executes a particular command.
pub fn execute(opt: &Opt, cmd: &Command) -> Result<(), failure::Error> {
    let mut repo = Repository::open(&opt.directory)?;
    let head_branch = git::get_head_branch(&repo)?;
    let cache_dir = std::fs::canonicalize(&opt.cache_dir)?.fix_path();
    trace!("Canonicalized cache directory: {:?}", cache_dir);
    util::init::set_cargo_workdir(&cache_dir)?;
    util::init::set_cwd(&opt.directory)?;

    let cargo_config = ws::cargo_config(opt.directory.clone());
    let cargo = ws::cargo_workspace(&cargo_config)
        .with_context(|e| format!("Error opening workspace: {}", e))?;

    let workspace = Workspace::new(&cargo, &mut repo)?;

    let runtime = ExecRuntime {
        repo,
        cache_dir,
        cargo: &cargo,
        workspace,
        head_branch,
        directory: opt.directory.clone(),
    };

    match cmd {
        Command::Release(cmd) => release::execute(cmd, runtime),
        Command::ReleaseTest => release::execute(
            &release::Command {
                skip_tests: true,
                dry_run: true,
                no_publish: true,
            },
            runtime,
        ),
        Command::UpdatePaths(cmd) => update_paths::execute(cmd, runtime),
    }
}
