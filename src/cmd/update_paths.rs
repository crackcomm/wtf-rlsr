//! Update paths command.

use std::path::PathBuf;

use failure::ResultExt;
use structopt::StructOpt;

use crate::{
    parser::collect_members,
    updater::Updater,
    util::{self, CleanPath},
    ws::{self, Workspace},
};

use super::exec::ExecRuntime;

/// Update options.
#[derive(Debug, StructOpt)]
#[structopt(name = "update-paths", about = "Updates workspace packages paths.")]
pub struct Command {
    /// Saves results to preview file.
    #[structopt(long = "dry-run")]
    pub dry_run: bool,

    /// Forces update of submodules versions.
    #[structopt(long = "force-deps", short = "f")]
    pub force_versions: bool,

    /// Git submodule dependency workspace.
    #[structopt(parse(from_os_str), long = "dep", short = "d")]
    pub dependencies: Vec<PathBuf>,
}

/// Executes an `update-paths` command.
pub fn execute(cmd: &Command, mut runtime: ExecRuntime) -> Result<(), failure::Error> {
    for package in runtime.workspace.packages().iter() {
        trace!(
            "Package: {} used workspace members: {:?}",
            package.name(),
            collect_members(&runtime.workspace, package)
        );
    }
    let mut updater = Updater::new(&mut runtime.repo)?;
    updater.set_paths(
        &mut runtime.repo,
        &runtime.workspace,
        cmd.dry_run,
        cmd.force_versions,
    )?;
    util::init::set_cwd(&runtime.directory)?;

    for dep in &cmd.dependencies {
        let dep_dir = std::fs::canonicalize(dep)?.fix_path();
        let config = ws::cargo_config(dep_dir.clone());
        let cargo = ws::cargo_workspace(&config)
            .with_context(|e| format!("Error opening dependency {:?} workspace: {}", dep, e))?;
        // let mut repo = Repository::open(&dep_dir)?;
        let workspace = Workspace::new(&cargo, &mut runtime.repo)?;
        updater.set_submodule_paths(
            &mut runtime.repo,
            &runtime.workspace,
            &workspace,
            cmd.dry_run,
            cmd.force_versions,
        )?;
    }

    updater.workspace.save_preview()?;
    if !cmd.dry_run {
        util::rename(
            updater.workspace.index_preview_path(),
            runtime.workspace.manifest_path(),
        )?;
    }
    Ok(())
}
