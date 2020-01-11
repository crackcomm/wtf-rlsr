mod manifests;
pub use self::manifests::*;

use hashbrown::HashMap;

use crate::{
    git::Repository,
    util::{self, CleanPath, Update},
    ws::{Package, Workspace},
};

pub struct Updater<'a> {
    pub workspace: WorkspaceManifests,
    pub toml_files: HashMap<String, PackageManifests<'a>>,
}

impl<'a> Updater<'a> {
    pub fn new(repo: &mut Repository) -> Result<Self, failure::Error> {
        Ok(Updater {
            workspace: WorkspaceManifests::new(repo)?,
            toml_files: HashMap::default(),
        })
    }

    pub fn update(
        &mut self,
        repo: &mut Repository,
        package: &'a Package,
        update: &Update,
    ) -> Result<(), failure::Error> {
        if let Some(bump) = update.as_bump() {
            // Get change package manigests
            let manifest = self.manifests(package, repo)?;
            // Bump version accordingly
            manifest.bump_ver(bump);
            // Save `Cargo.preview-head.toml` and `Cargo.preview-index.toml`.
            manifest.save_preview()?;
            // Bump replace in cargo workspace manifest.
            self.workspace.bump_replace_ver(package, bump);
        }
        Ok(())
    }

    pub fn set_paths(
        &mut self,
        repo: &mut Repository,
        workspace: &'a Workspace<'a>,
        dry_run: bool,
        force_versions: bool,
    ) -> Result<(), failure::Error> {
        for pkg in workspace.packages().iter() {
            self.set_pkg_paths_impl(pkg, repo, workspace, dry_run, force_versions, false)?;
        }
        Ok(())
    }

    pub fn set_submodule_paths(
        &mut self,
        repo: &mut Repository,
        workspace: &'a Workspace<'a>,
        source: &Workspace,
        dry_run: bool,
        force_versions: bool,
    ) -> Result<(), failure::Error> {
        for pkg in workspace.packages().iter() {
            self.set_pkg_paths_impl(pkg, repo, source, dry_run, force_versions, true)?;
        }
        Ok(())
    }

    fn set_pkg_paths_impl(
        &mut self,
        pkg: &'a Package,
        repo: &mut Repository,
        source: &Workspace,
        dry_run: bool,
        force_versions: bool,
        is_submodule: bool,
    ) -> Result<(), failure::Error> {
        let pkg_path = repo.rel_path(pkg.directory());
        let packages: Vec<_> = pkg
            .dependencies()
            .iter()
            .filter_map(|dep| {
                source
                    .find_package(dep.package_name().as_str())
                    .map(|dep_pkg| (dep, dep_pkg))
            })
            .collect();
        if packages.len() == 0 {
            trace!("No members in package {}.", pkg.name());
            return Ok(());
        }

        for (dep, dep_pkg) in packages {
            let manifests = self.manifests(pkg, repo)?;
            let name = dep_pkg.name().as_str();
            trace!(
                "Dependency {} manifest_path: {:?}",
                name,
                dep_pkg.directory()
            );
            let dep_path = repo.rel_path(dep_pkg.directory());
            trace!("Dependency {} path: {:?}", name, dep_path);
            let rel_path = pathdiff::diff_paths(&dep_path, &pkg_path)
                .unwrap()
                .clean_path();
            if force_versions {
                manifests.set_dep_force(
                    dep,
                    dep_pkg,
                    if is_submodule { None } else { Some(&rel_path) },
                );
                self.workspace.set_replace(dep_pkg, &dep_path);
            } else {
                manifests.set_dep_path(name, &rel_path, dep_pkg.version());
            }
        }

        trace!("Saving package {} paths.", pkg.name());
        let manifests = self.manifests(pkg, repo)?;
        manifests.save_preview().unwrap();
        if !dry_run {
            util::rename(manifests.index_preview_path(), &manifests.manifest_path)?;
        }
        Ok(())
    }

    pub fn manifests(
        &mut self,
        pkg: &'a Package,
        repo: &mut Repository,
    ) -> Result<&mut PackageManifests<'a>, failure::Error> {
        let name = pkg.name().as_str();
        if !self.toml_files.contains_key(name) {
            let toml_file = PackageManifests::new(repo, pkg)?;
            self.toml_files.insert(name.to_owned(), toml_file);
        }
        Ok(self.toml_files.get_mut(name).unwrap())
    }
}

/// Colects dependats to update.
/// Returns vector of tuples containing name and wether package is changed.
pub fn collect_dependants<'a>(
    workspace: &'a Workspace,
    package: &'a Package,
) -> Vec<&'a Package<'a>> {
    let mut result = collect_dependants_impl(workspace, &package.name().to_string());
    result.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));
    result.dedup_by(|a, b| a.name().as_str() == b.name().as_str());
    result.sort_by_key(|dep| dep.is_changed());
    result
}

fn collect_dependants_impl<'a>(workspace: &'a Workspace, name: &String) -> Vec<&'a Package<'a>> {
    let mut result = Vec::new();
    for (node, _) in workspace.graphs.dependants.edges(name) {
        let package = workspace.find_package(node.as_str()).unwrap();
        result.push(package);
        result.extend(collect_dependants_impl(workspace, &node));
    }
    result
}
