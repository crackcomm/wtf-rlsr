#![feature(str_strip, try_trait)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod git;
pub mod parser;
pub mod ui;
pub mod updater;
pub mod util;
pub mod ws;
pub mod prelude {
    pub use cargo::core::{
        dependency::Dependency as CargoDependency, Package as CargoPackage,
        Workspace as CargoWorkspace,
    };
    pub use cargo::util::Config as CargoConfig;
}
pub use self::git::{CommitBuilder, Repository};
pub use self::updater::{ManifestExt, PackageManifestExt, Updater, WorkspaceManifestExt};
pub use self::util::{Bump, BumpExt, CleanPath, Logger, Update};
pub use self::ws::{Package, Workspace};

use std::path::{Path, PathBuf};

use colored::Colorize;
use structopt::StructOpt;

/// Command line application options.
#[derive(Debug, StructOpt)]
#[structopt(name = "wtf-rlrsr", about = "WTF Releaser.")]
pub struct Opt {
    /// Workspace directory.cargo
    #[structopt(parse(from_os_str), default_value = ".")]
    directory: PathBuf,

    /// Cache directory.
    #[structopt(parse(from_os_str), short = "c", default_value = "../cxmr-core-cache")]
    cache_dir: PathBuf,

    /// Git remote.
    #[structopt(long = "remote", default_value = "origin")]
    remote: String,

    /// Update paths.
    #[structopt(long = "update-paths")]
    update_paths: bool,

    /// Skip packages tests.
    #[structopt(long = "skip-tests")]
    skip_tests: bool,

    /// Don't publish packages.
    #[structopt(long = "no-publish")]
    no_publish: bool,

    /// Publish dry run.
    #[structopt(long = "dry-run")]
    dry_run: bool,
}

pub fn execute() -> Result<(), failure::Error> {
    let opt = util::init::setup_opt()?;
    let mut repo = git::init_main_repo(&opt.directory)?;
    let head_branch = git::get_head_branch(&repo)?;
    let mut cache_repo = git::init_cache_repo(&opt.cache_dir, &opt.directory, &head_branch)?;
    let cache_dir = std::fs::canonicalize(opt.cache_dir)?.fix_path();
    trace!("Canonicalized cache directory: {:?}", cache_dir);
    util::init::set_cargo_workdir(&cache_dir)?;
    util::init::set_cwd(&opt.directory)?;

    let cargo_config = ws::cargo_config(opt.directory.clone());
    let cargo = match ws::cargo_workspace(&cargo_config) {
        Ok(cargo) => cargo,
        Err(err) => {
            println!("Error opening workspace: {:?}", err);
            return Ok(());
        }
    };

    let workspace = Workspace::new(&cargo, &mut repo)?;
    trace!("Workspace version: {}", workspace.version());

    if opt.update_paths {
        let mut updater = Updater::new(&mut repo)?;
        updater.set_paths(&mut repo, &workspace)?;
    }

    if workspace.packages().changed() == 0 {
        println!(
            "No changed packages in {} v{}",
            workspace.name(),
            workspace.version()
        );
        return Ok(());
    }

    let package = match ui::packages::select_changed(&workspace)? {
        Some(package) => package,
        None => return Ok(()),
    };
    let update = match ui::update::prompt(package)? {
        Some(bump) => bump,
        None => return Ok(()),
    };
    let new_pkg_ver = update.bump(package.version());
    let dependants = updater::collect_dependants(&workspace, package);
    println!("Packages affected by update:");
    for pkg in &dependants {
        if !pkg.is_changed() {
            println!("  * {}", pkg.name().to_string().yellow());
        } else {
            println!("  * {}", pkg.name().to_string().red());
        }
    }
    println!();

    if ui::confirm("Do you want to see git diff?")? {
        let opts = util::diff2html::Options::default();
        util::diff2html::spawn_for_pkg(package, &opts)?;
        if !ui::confirm("Do you want to continue?")? {
            return Ok(());
        }
    }

    let header = ui::commit::prompt_header("Commit header")?;
    let message = ui::commit::prompt("Commit message")?;

    let (update_packages, _ignored_packages) = if let Some(bump) = update.as_bump() {
        ui::packages::select_update_deps(&dependants, bump)
    } else {
        (vec![], vec![])
    };

    let mut updater = Updater::new(&mut repo)?;
    if opt.update_paths {
        updater.set_paths(&mut repo, &workspace)?;
    }

    // Update package version using Updater.
    // It will save preview manifest files.
    updater.update(&mut repo, package, &update)?;

    // Save manifest in workspace
    updater.workspace.save_preview()?;
    // Save manifest in cached repo for testing purposes
    updater.workspace.head.save(&cache_dir.join("Cargo.toml"))?;

    // Copy `Cargo.preview-index.toml` as cached repo package manifest.
    if update.as_bump().is_some() {
        let manifests = updater.manifests(package, &mut repo)?;
        manifests.save_preview()?;
        util::copy(
            &manifests.index_preview_path(),
            cache_dir.join(package.manifest_path()),
        )?;
        util::copy(
            &updater.workspace.index_preview_path(),
            cache_dir.join(workspace.manifest_path()),
        )?;
    }

    // Start building a commit for changed package
    let mut commit = CommitBuilder::new(&mut repo)?;
    let diff = package.diff.as_ref().unwrap();
    util::commit::add_files(&mut commit, &diff.changed_files, &cache_dir)?;
    util::commit::remove_files(&mut commit, &diff.deleted_files, &cache_dir)?;

    // Add workspace and package manifests to git commit
    if !opt.dry_run {
        util::commit::add_preview_head(&mut commit, package.manifest_path())?;
        util::commit::add_preview_head(&mut commit, workspace.manifest_path())?;
        util::commit::move_index_manifest(package.manifest_path())?;
        util::commit::move_index_manifest(workspace.manifest_path())?;
    }

    // Bump package dependants versions
    for pkg_dep in update_packages.iter() {
        let bump = update.as_bump().unwrap();
        let pkg_dep_bump = bump.dependency(pkg_dep.is_changed());
        // Bump replace in cargo workspace manifest.
        updater.workspace.bump_replace_ver(pkg_dep, pkg_dep_bump);

        // let cargo_toml = repo.get_contents(tree, pkg_dep)
        let manifest = updater.manifests(pkg_dep, &mut repo)?;
        manifest.bump_ver(pkg_dep_bump);
        manifest.update_dep(package.name().as_str(), package.version(), &new_pkg_ver);

        update_packages.iter().for_each(|deep_dep| {
            let name = deep_dep.name().as_str();
            let new_deep_dep_ver = deep_dep
                .version()
                .bump(bump.dependency(deep_dep.is_changed()));
            manifest.update_dep(name, deep_dep.version(), &new_deep_dep_ver);
        });
        // Save package manifest in `Cargo.preview-head.toml`.
        manifest.save_preview()?;
        // Copy package manifest to cached repository.
        util::copy(
            &manifest.head_preview_path(),
            cache_dir.join(&manifest.manifest_path),
        )?;
    }

    let cache_config = ws::cargo_config(cache_dir.clone());
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
    if update.as_bump().is_some() && !opt.skip_tests && !util::run_tests(cached_pkg, &cache_cargo)?
    {
        util::commit::restore_manifest(package.manifest_path())?;
        util::commit::restore_manifest(workspace.manifest_path())?;
        return Ok(());
    }

    // Start publishing to crates.io
    if update.as_bump().is_some() && !opt.no_publish {
        cargo.status("Publishing", "Starting");
        let mut published = Vec::new();
        if !util::publish_pkg_deep(
            &cached_pkg,
            &cache_workspace,
            &cache_config,
            &update_packages,
            &mut published,
            opt.dry_run,
        )? {
            util::commit::restore_manifest(package.manifest_path())?;
            util::commit::restore_manifest(workspace.manifest_path())?;
            return Ok(());
        }
        cargo.status("Publishing", "Done");
        cargo.status("Committing", "Starting");
    }

    if !opt.dry_run {
        let commit_message =
            util::commit::message(package, update, &header, message.as_ref(), false);
        commit.commit(commit_message.trim(), &mut repo)?;
    }

    let mut commit = CommitBuilder::new(&mut repo)?;

    if !opt.dry_run {
        for dep in update_packages.iter() {
            util::commit::add_preview_head(&mut commit, dep.manifest_path())?;
            util::commit::move_index_manifest(dep.manifest_path())?;
        }
    }

    // Save manifest in workspace
    updater.workspace.save_preview()?;
    // Save manifest in cached repo for testing purposes
    updater.workspace.head.save(&cache_dir.join("Cargo.toml"))?;
    if !opt.dry_run {
        util::commit::add_preview_head(&mut commit, workspace.manifest_path())?;
        util::commit::move_index_manifest(workspace.manifest_path())?;
    }

    if !opt.dry_run {
        if let Some(bump) = update.as_bump() {
            workspace.bump(bump)?;
            let path = Path::new("package.json");
            commit.add_path(&path)?;
            let package_json = opt.directory.join("package.json");
            let cache_pkg_json = cache_dir.join("package.json");
            util::copy(&package_json, cache_pkg_json)?;
        }
        let commit_message =
            util::commit::message(package, update, &header, message.as_ref(), true);
        commit.commit(commit_message.trim(), &mut repo)?;
        let rls_tag = format!("refs/tags/v{}", update.bump(workspace.version()));
        let branch_tag = format!("refs/heads/{}", head_branch);
        git::set_head_ref(&rls_tag, &mut repo)?;
        // git::set_head_ref(&branch_tag, &mut repo)?;
        git::push_remote(&repo, "origin", &[&branch_tag, &rls_tag])?;
    }

    if let Err(err) = cache_repo.stash_apply(0, None) {
        trace!("Stash apply error: {:?}", err);
    }
    Ok(())
}
