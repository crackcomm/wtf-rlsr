#![feature(str_strip, try_trait)]

#[macro_use]
extern crate log;

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
pub use self::git::{pull, CommitBuilder, Repository};
pub use self::updater::{ManifestExt, PackageManifestExt, Updater, WorkspaceManifestExt};
pub use self::util::{publish_pkg_deep, test_pkg, Bump, BumpExt, CleanPath, Logger};
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

pub fn init() {
    pretty_env_logger::init();
    cargo::core::features::enable_nightly_features();
}

fn set_cargo_target_dir<P: AsRef<std::ffi::OsStr>>(dir: P) {
    trace!("CARGO_TARGET_DIR: {:?}", dir.as_ref());
    std::env::set_var("CARGO_TARGET_DIR", dir);
}

pub fn execute() -> Result<(), failure::Error> {
    let mut opt = Opt::from_args();
    if opt.directory.to_str().unwrap() == "." {
        opt.directory = std::env::current_dir()?.clean_path();
    } else {
        opt.directory = std::fs::canonicalize(&opt.directory)?.clean_path();
    }

    opt.cache_dir = opt.cache_dir.clean_path();
    let cache_dir = std::fs::canonicalize(&opt.cache_dir)?.clean_path();
    trace!("Cache dir: {:?}", opt.cache_dir);
    trace!("Canonicalized cache dir: {:?}", cache_dir);
    set_cargo_target_dir(cache_dir.join("target/cxmr-rlsrs"));

    if !opt.cache_dir.exists() {
        trace!("Cloning repository to {:?}", opt.cache_dir);
        let mut builder = git2::build::RepoBuilder::new();
        builder.clone_local(git2::build::CloneLocal::Local);
        builder.clone(opt.directory.to_str().unwrap(), &opt.cache_dir)?;
    }

    trace!("Workspace dir: {:?}", opt.directory);
    std::env::set_current_dir(&opt.directory)?;
    trace!("Current working directory: {:?}", std::env::current_dir()?);
    let cargo_config = ws::cargo_config(opt.directory.clone());
    let cargo = match ws::cargo_workspace(&cargo_config) {
        Ok(cargo) => cargo,
        Err(err) => {
            println!("Error opening workspace: {:?}", err);
            return Ok(());
        }
    };

    let mut repo = Repository::new();
    let workspace = Workspace::new(&cargo, &mut repo);
    let remote_url = repo.find_remote("origin")?.url().unwrap().to_owned();
    let mut rev_id = repo.head_commit().id().to_string();
    rev_id.truncate(8);

    if opt.update_paths {
        let mut updater = Updater::new(&mut repo);
        updater.set_paths(&mut repo, &workspace);
    }

    let mut cache_repo = Repository::open(&cache_dir);
    let signature = repo.signature()?;
    if let Err(err) = cache_repo.stash_save(
        &signature,
        "cxmr-rlsr",
        Some(git2::StashFlags::INCLUDE_UNTRACKED),
    ) {
        trace!("Stash error: {:?}", err);
    }
    {
        let mut remote = repo.find_remote("origin")?;
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        let homedir = dirs::home_dir().unwrap();
        let username = whoami::username();
        let ssh_pub = homedir.join(".ssh").join("id_rsa.pub");
        let ssh_priv = homedir.join(".ssh").join("id_rsa.pub");
        remote_callbacks.credentials(move |_, _, _| {
            git2::Cred::ssh_key("git", Some(&ssh_pub), &ssh_priv, None)
        });
        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(remote_callbacks);
        remote.push(&["HEAD"], Some(&mut push_opts))?;
    }
    {
        let branch_name = if let Some(head_branch) = repo
            .branches(Some(git2::BranchType::Local))?
            .filter_map(|b| b.ok().map(|(b, _)| b))
            .find(|b| b.is_head())
        {
            head_branch.name()?.unwrap().to_owned()
        } else {
            git::WTF_RLSR_TAG.to_owned()
        };
        trace!("Git head tag {:?} ", branch_name);
        let mut remote = cache_repo.find_remote("origin")?;
        let fetch_commit = pull::do_fetch(&cache_repo, &[&branch_name], &mut remote)?;
        pull::do_merge(&cache_repo, &branch_name, fetch_commit)?;
    }

    loop {
        let package = match ui::packages::select_changed(&workspace)? {
            Some(package) => package,
            None => break,
        };
        let bump = match ui::bump::prompt(package)? {
            Some(bump) => bump,
            None => break,
        };
        let new_pkg_ver = package.version().bump(bump);

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
        }

        let header = ui::commit::prompt_header("Commit header")?;
        let message = ui::commit::prompt("Commit message")?;

        let (update_packages, packages_to_git) = if !bump.is_chore() {
            ui::packages::select_update_deps(&dependants, bump)
        } else {
            (vec![], vec![])
        };

        let mut updater = Updater::new(&mut repo);
        if opt.update_paths {
            updater.set_paths(&mut repo, &workspace);
        }

        {
            // Get change package manigests
            let manifest = updater.manifests(package, &mut repo);
            // Bump version accordingly
            manifest.bump_ver(bump);
            // Save `Cargo.preview-head.toml` and `Cargo.preview-index.toml`.
            manifest.save_preview()?;
            // Copy `Cargo.preview-index.toml` as cached repo package manifest.
            util::copy(
                &manifest.index_preview_path(),
                cache_dir.join(&manifest.manifest_path),
            )?;
        }

        // Bump replace in cargo workspace manifest.
        updater.manifests.bump_ver(package, bump);
        if packages_to_git.len() > 0 {
            // Insert git replace for not updated repos
            updater
                .manifests
                .head
                .git_replace(package, &remote_url, &rev_id);
        }

        // Start building a commit for changed package
        let mut commit = CommitBuilder::new(&mut repo)?;
        let diff = package.diff.as_ref().unwrap();
        for file in &diff.deleted_files {
            // Add removed file to commit
            commit.add_path(&file.clean_path())?;
            // Remove file in cached repo
            let dest = cache_dir.join(&file);
            util::remove_file(dest)?;
        }
        for file in &diff.changed_files {
            // Add changed file to commit
            commit.add_path(&file.clean_path())?;
            // Copy changed file to cached repo
            let dest = cache_dir.join(&file);
            util::copy(&file, dest)?;
        }

        // Save manifest in workspace
        updater.manifests.save_preview()?;
        // Save manifest in cached repo for testing purposes
        updater.manifests.head.save(&cache_dir.join("Cargo.toml"))?;

        // Add workspace and package manifests to git commit
        let workspace_manifest = repo
            .rel_path(&opt.directory.join("Cargo.toml"))
            .clean_path();
        let manifest_path = repo.rel_path(package.manifest_path());
        add_preview_head(&mut commit, &manifest_path)?;
        add_preview_head(&mut commit, &workspace_manifest)?;

        let cache_config = ws::cargo_config(cache_dir.clone());
        let cache_cargo = match ws::cargo_workspace(&cache_config) {
            Ok(cargo) => cargo,
            Err(err) => {
                println!("Error opening workspace: {:?}", err);
                return Ok(());
            }
        };

        // Get cached workspace structure
        let cache_workspace = Workspace::new(&cache_cargo, &mut cache_repo);
        // Get package in cached workspace
        let cached_pkg = cache_workspace
            .find_package(package.name().as_str())
            .unwrap();
        if !opt.skip_tests {
            // Run package tests
            if !test_pkg(cached_pkg, &cache_cargo)? {
                return Ok(());
            }
            // Test all package dependants
            for pkg_dep in &update_packages {
                let cached_dep = cache_workspace
                    .find_package(pkg_dep.name().as_str())
                    .unwrap();
                if !test_pkg(cached_dep, &cache_cargo)? {
                    return Ok(());
                }
            }
        }

        if !bump.is_chore() && !opt.no_publish {
            cargo.status("Publishing", "Starting");
            let mut published = Vec::new();
            if !publish_pkg_deep(
                &cached_pkg,
                &cache_workspace,
                &cache_config,
                &update_packages,
                &mut published,
                opt.dry_run,
            )? {
                return Ok(());
            }
            cargo.status("Publishing", "Done");
            cargo.status("Committing", "Starting");
        }

        let commit_prepend = format!(
            "{}({}): {} of {} {} ({})",
            match bump {
                Bump::Chore => "chore",
                Bump::Patch => "fix",
                Bump::Minor => "feat",
                Bump::Major => "feat",
            },
            package.name().to_string().replacen("-", "/", 1),
            match bump {
                Bump::Chore => "cleanup",
                Bump::Patch => "patch",
                Bump::Minor => "minor update",
                Bump::Major => "major update",
            },
            package.name(),
            if bump.is_chore() {
                format!("v{}", package.version())
            } else {
                format!("v{} → {}", package.version(), new_pkg_ver)
            },
            header
        );
        let mut commit_lines = vec![commit_prepend];
        if let Some(lines) = &message {
            commit_lines.push("".to_owned());
            commit_lines.extend(lines.clone());
        }
        for line in &commit_lines {
            trace!("Package commit message: {}", line);
        }
        let commit_message = commit_lines.join("\n");
        // commit.commit(commit_message.trim(), &mut repo)?;

        let mut commit = CommitBuilder::new(&mut repo)?;

        for pkg_dep in &update_packages {
            let pkg_dep_bump = if pkg_dep.is_changed() {
                bump
            } else {
                &Bump::Patch
            };
            // Bump replace in cargo workspace manifest.
            updater.manifests.bump_ver(pkg_dep, pkg_dep_bump);

            // let cargo_toml = repo.get_contents(tree, pkg_dep)
            let manifest = updater.manifests(pkg_dep, &mut repo);
            manifest.bump_ver(pkg_dep_bump);
            manifest.update_dep(package.name().as_str(), package.version(), &new_pkg_ver);

            update_packages.iter().for_each(|deep_dep| {
                let name = deep_dep.name().as_str();
                manifest.update_dep(
                    name,
                    deep_dep.version(),
                    &deep_dep.version().bump(if deep_dep.is_changed() {
                        match bump {
                            Bump::Chore => panic!("Chore dependency update."),
                            Bump::Major | Bump::Minor => &Bump::Minor,
                            Bump::Patch => bump,
                        }
                    } else {
                        &Bump::Patch
                    }),
                );
            });
            // Save package manifest in `Cargo.preview-head.toml`.
            manifest.save_preview()?;
            // Copy package manifest to cached repository.
            util::copy(
                &manifest.head_preview_path(),
                cache_dir.join(&manifest.manifest_path),
            )?;
            // Add manifest to git commit
            add_preview_head(&mut commit, &manifest.manifest_path)?;
        }

        // Save manifest in workspace
        updater.manifests.save_preview()?;
        // Save manifest in cached repo for testing purposes
        updater.manifests.head.save(&cache_dir.join("Cargo.toml"))?;
        add_preview_head(&mut commit, &workspace_manifest)?;

        let update = match bump {
            Bump::Chore => "cleanup",
            Bump::Patch => "patch",
            Bump::Minor => "minor update",
            Bump::Major => "major update",
        };
        let dep_commit = format!(
            "chore(*): {} of {} {} ({})",
            update,
            package.name(),
            if bump.is_chore() {
                format!("v{}", package.version())
            } else {
                format!("v{} → {}", package.version(), new_pkg_ver)
            },
            header
        );
        let mut commit_lines = vec![dep_commit];
        if let Some(lines) = &message {
            commit_lines.push("".to_owned());
            commit_lines.extend(lines.clone());
        }
        for line in &commit_lines {
            trace!("Dependency commit message: {}", line);
        }
        let commit_message = commit_lines.join("\n");
        // commit.commit(commit_message.trim(), &mut repo)?;

        // TODO: after commit
        // cache_repo.stash_apply(0, None)?;
    }
    Ok(())
}

fn move_index_manifest<P: AsRef<Path>>(manifest_path: P) -> Result<(), failure::Error> {
    let source_dir = manifest_path.as_ref().parent().unwrap();
    let backup_toml = source_dir.join("Cargo.backup.toml");
    let preview_index = source_dir.join("Cargo.preview-index.toml");
    util::rename(&manifest_path, &backup_toml)?;
    util::rename(preview_index, &manifest_path)?;
    Ok(())
}

fn add_preview_head<P: AsRef<Path>>(
    commit: &mut CommitBuilder,
    manifest_path: P,
) -> Result<(), failure::Error> {
    let source_dir = manifest_path.as_ref().parent().unwrap();
    let backup_toml = source_dir.join("Cargo.backup.toml");
    let preview_head = source_dir.join("Cargo.preview-head.toml");
    util::rename(&manifest_path, &backup_toml)?;
    util::rename(preview_head, &manifest_path)?;
    commit.add_path(&manifest_path)?;
    util::rename(&backup_toml, &manifest_path)?;
    move_index_manifest(manifest_path)?;
    Ok(())
}
