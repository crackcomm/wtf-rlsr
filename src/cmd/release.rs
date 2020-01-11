//! Release command.

use std::path::Path;

use colored::Colorize;
use structopt::StructOpt;

use crate::{
    git::{self, CommitBuilder},
    ui,
    updater::{self, ManifestExt, Updater},
    util::{self, BumpExt, Logger},
    ws::{self, Workspace},
};

use super::exec::ExecRuntime;

/// Release options.
#[derive(Debug, StructOpt)]
#[structopt(name = "release", about = "Release a package.")]
pub struct Command {
    /// Skip packages tests.
    #[structopt(long = "skip-tests")]
    pub skip_tests: bool,

    /// Don't publish packages.
    #[structopt(long = "no-publish")]
    pub no_publish: bool,

    /// Publish dry run.
    #[structopt(long = "dry-run")]
    pub dry_run: bool,
}

/// Executes a `release` command.
pub fn execute(cmd: &Command, mut runtime: ExecRuntime) -> Result<(), failure::Error> {
    if runtime.workspace.packages().changed() == 0 {
        println!(
            "No changed packages in {} v{}",
            runtime.workspace.name(),
            runtime.workspace.version()
        );
        return Ok(());
    }

    // Open cache repo
    let mut cache_repo = runtime.open_cache_repo()?;

    // Select main package to release
    let package = match ui::packages::select_changed(&runtime.workspace)? {
        Some(package) => package,
        None => return Ok(()),
    };

    // Select update kind
    let update = match ui::update::prompt(package)? {
        Some(bump) => bump,
        None => return Ok(()),
    };

    // Get new version of a package (or the same if not bumped)
    let new_pkg_ver = update.bump(package.version());

    // Get all dependandts of the package
    let dependants = updater::collect_dependants(&runtime.workspace, package);

    println!("Packages affected by update:");
    for pkg in &dependants {
        if !pkg.is_changed() {
            println!("  * {}", pkg.name().to_string().yellow());
        } else {
            println!("  * {}", pkg.name().to_string().red());
        }
    }
    println!();

    let (commit_packages, _) = {
        let changed: Vec<_> = dependants
            .iter()
            .filter(|p| p.is_changed())
            .map(|p| *p)
            .collect();
        ui::packages::select_packages("Commit changes of dependencies", &changed, &update, false)
    };

    let (tree_packages, _) = {
        let not_commit: Vec<_> = dependants
            .iter()
            .filter(|p| !commit_packages.contains(*p))
            .map(|p| *p)
            .collect();
        ui::packages::select_packages(
            "Select dependencies to update in tree",
            &not_commit,
            &update,
            true,
        )
    };

    let update_packages = [commit_packages.as_slice(), tree_packages.as_slice()].concat();

    if ui::confirm("Do you want to see git diff?")? {
        let opts = util::diff2html::Options::default();
        let diff_pkgs = &[&[package], commit_packages.as_slice()].concat();
        util::diff2html::spawn_for_pkgs(diff_pkgs.as_slice(), &opts)?;
        if !ui::confirm("Do you want to continue?")? {
            return Ok(());
        }
    }

    let header = ui::commit::prompt_header("Commit header")?;
    let message = ui::commit::prompt("Commit message")?;

    let mut updater = Updater::new(&mut runtime.repo)?;

    // Update package version using Updater.
    // It will save preview manifest files.
    updater.update(&mut runtime.repo, package, &update)?;

    // Save manifest in workspace
    updater.workspace.save_preview()?;
    // Save manifest in cached repo for testing purposes
    updater
        .workspace
        .head
        .save(&runtime.cache_dir.join("Cargo.toml"))?;

    // Copy `Cargo.preview-index.toml` as cached repo package manifest.
    if update.as_bump().is_some() {
        let manifests = updater.manifests(package, &mut runtime.repo)?;
        manifests.save_preview()?;
        util::copy(
            &manifests.index_preview_path(),
            runtime.cache_dir.join(package.manifest_path()),
        )?;
        util::copy(
            &updater.workspace.index_preview_path(),
            runtime.cache_dir.join(runtime.workspace.manifest_path()),
        )?;
    }

    // Start building a commit for changed package
    let mut commit = CommitBuilder::new(&mut runtime.repo)?;
    util::commit::add_diff(
        &mut commit,
        package.diff.as_ref().unwrap(),
        &runtime.cache_dir,
    )?;

    // Add workspace and package manifests to git commit
    if !cmd.dry_run {
        util::commit::add_preview_index(&mut commit, package.manifest_path())?;
        util::commit::add_preview_head(&mut commit, runtime.workspace.manifest_path())?;
        util::commit::move_index_manifest(package.manifest_path())?;
        util::commit::move_index_manifest(runtime.workspace.manifest_path())?;
        for dep in commit_packages.iter() {
            util::commit::add_preview_index(&mut commit, dep.manifest_path())?;
            util::commit::move_index_manifest(dep.manifest_path())?;
            util::commit::add_diff(&mut commit, dep.diff.as_ref().unwrap(), &runtime.cache_dir)?;
        }
    }

    // Bump package dependants versions
    if let Some(bump) = update.as_bump() {
        for pkg_dep in update_packages.iter() {
            let is_commit = commit_packages.contains(pkg_dep);
            let pkg_dep_bump = bump.dependency(pkg_dep.is_changed(), is_commit);
            // Bump replace in cargo workspace manifest.
            updater.workspace.bump_replace_ver(pkg_dep, pkg_dep_bump);

            // let cargo_toml = repo.get_contents(tree, pkg_dep)
            let manifest = updater.manifests(pkg_dep, &mut runtime.repo)?;
            manifest.bump_ver(pkg_dep_bump);
            manifest.update_dep(package.name().as_str(), package.version(), &new_pkg_ver);

            update_packages.iter().for_each(|deep_dep| {
                let name = deep_dep.name().as_str();
                let is_commit = commit_packages.contains(pkg_dep);
                let new_deep_dep_ver = deep_dep
                    .version()
                    .bump(bump.dependency(deep_dep.is_changed(), is_commit));
                manifest.update_dep(name, deep_dep.version(), &new_deep_dep_ver);
            });
            // Save package manifest in `Cargo.preview-head.toml`.
            manifest.save_preview()?;
            // Copy package manifest to cached repository.
            util::copy(
                &manifest.head_preview_path(),
                runtime.cache_dir.join(&manifest.manifest_path),
            )?;
        }
    }

    let cache_config = ws::cargo_config(runtime.cache_dir.clone());
    let cache_cargo = match ws::cargo_workspace(&cache_config) {
        Ok(cargo) => cargo,
        Err(err) => {
            println!("Error opening workspace: {:?}", err);
            return Ok(());
        }
    };

    // Get cached workspace structure
    let cache_workspace = Workspace::new(&cache_cargo, &mut cache_repo)?;
    // Get package in cached workspace
    let cached_pkg = cache_workspace
        .find_package(package.name().as_str())
        .unwrap();
    // Run package tests if they are enabled
    if update.as_bump().is_some() && !cmd.skip_tests && !util::run_tests(cached_pkg, &cache_cargo)?
    {
        util::commit::restore_manifest(package.manifest_path())?;
        util::commit::restore_manifest(runtime.workspace.manifest_path())?;
        return Ok(());
    }

    // Start publishing to crates.io
    if update.as_bump().is_some() && !cmd.no_publish {
        runtime.cargo.status("Publishing", "Starting");
        let mut published = Vec::new();
        if !util::publish_pkg_deep(
            &cached_pkg,
            &cache_workspace,
            &cache_config,
            &update_packages,
            &mut published,
            cmd.dry_run,
        )? {
            util::commit::restore_manifest(package.manifest_path())?;
            util::commit::restore_manifest(runtime.workspace.manifest_path())?;
            return Ok(());
        }
        runtime.cargo.status("Publishing", "Done");
        runtime.cargo.status("Committing", "Starting");
    }

    if !cmd.dry_run {
        let commit_message =
            util::commit::message(package, update, &header, message.as_ref(), false);
        commit.commit(commit_message.trim(), &mut runtime.repo)?;
    }

    let mut commit = CommitBuilder::new(&mut runtime.repo)?;

    if !cmd.dry_run {
        for dep in tree_packages.iter() {
            util::commit::add_preview_head(&mut commit, dep.manifest_path())?;
            util::commit::move_index_manifest(dep.manifest_path())?;
        }
    }

    // Save manifest in workspace
    updater.workspace.save_preview()?;
    // Save manifest in cached repo for testing purposes
    updater
        .workspace
        .head
        .save(&runtime.cache_dir.join("Cargo.toml"))?;
    if !cmd.dry_run {
        util::commit::add_preview_head(&mut commit, runtime.workspace.manifest_path())?;
        util::commit::move_index_manifest(runtime.workspace.manifest_path())?;
    }

    if !cmd.dry_run {
        if let Some(bump) = update.as_bump() {
            runtime.workspace.bump(bump)?;
            let path = Path::new("package.json");
            commit.add_path(&path)?;
            let package_json = runtime.directory.join("package.json");
            let cache_pkg_json = runtime.cache_dir.join("package.json");
            util::copy(&package_json, cache_pkg_json)?;
        }
        let commit_message =
            util::commit::message(package, update, &header, message.as_ref(), true);
        commit.commit(commit_message.trim(), &mut runtime.repo)?;
        let rls_tag = format!("refs/tags/v{}", update.bump(runtime.workspace.version()));
        let branch_tag = format!("refs/heads/{}", runtime.head_branch);
        git::set_head_ref(&rls_tag, &mut runtime.repo)?;
        git::push_remote(&runtime.repo, "origin", &[&branch_tag, &rls_tag])?;
    }

    if let Err(err) = cache_repo.stash_apply(0, None) {
        trace!("Stash apply error: {:?}", err);
    }
    Ok(())
}
