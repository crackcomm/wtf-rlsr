mod manifests;
pub use self::manifests::*;

use hashbrown::HashMap;

use crate::{
    git::Repository,
    util::CleanPath,
    ws::{Package, Workspace},
};

pub struct Updater<'a> {
    pub manifests: WorkspaceManifests,
    pub toml_files: HashMap<String, PackageManifests<'a>>,
}

impl<'a> Updater<'a> {
    pub fn new(repo: &mut Repository) -> Self {
        Updater {
            manifests: WorkspaceManifests::new(repo),
            toml_files: HashMap::default(),
        }
    }

    pub fn set_paths(&mut self, repo: &mut Repository, workspace: &'a Workspace<'a>) {
        for pkg in workspace.packages().iter() {
            self.set_pkg_paths(pkg, repo, &workspace);
        }
    }

    pub fn set_pkg_paths(
        &mut self,
        pkg: &'a Package,
        repo: &mut Repository,
        workspace: &Workspace,
    ) {
        let manifests = self.manifests(pkg, repo);
        let pkg_path = repo.rel_path(pkg.manifest_path().parent().unwrap());
        let changed = pkg
            .dependencies()
            .iter()
            .filter(|dep| dep.is_member())
            .fold(false, |_, dep| {
                let name = dep.package_name().as_str();
                let dep_pkg = workspace.find_package(name).unwrap();
                let dep_path = repo.rel_path(dep_pkg.manifest_path().parent().unwrap());
                let rel_path = pathdiff::diff_paths(&dep_path, &pkg_path)
                    .unwrap()
                    .clean_path();
                manifests.set_dep_path(name, &rel_path, dep_pkg.version());
                true
            });
        if changed {
            manifests.save_preview().unwrap();
        }
    }

    pub fn manifests(
        &mut self,
        pkg: &'a Package,
        repo: &mut Repository,
    ) -> &mut PackageManifests<'a> {
        let name = pkg.name().as_str();
        if !self.toml_files.contains_key(name) {
            let toml_file = PackageManifests::new(repo, pkg);
            self.toml_files.insert(name.to_owned(), toml_file);
        }
        self.toml_files.get_mut(name).unwrap()
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
