use std::path::{Path, PathBuf};

use cargo::core::Package;
use semver::Version;

use crate::{
    git::Repository,
    util::{Bump, BumpExt, CleanPath},
};

use super::{ManifestExt, PackageManifestExt};

/// Package manifests on HEAD and index.
pub struct PackageManifests<'a> {
    pub pkg: &'a Package,
    pub head: PackageManifest<'a>,
    pub index: PackageManifest<'a>,
    pub manifest_path: PathBuf,
}

impl<'a> PackageManifests<'a> {
    /// Creates new package manifests structure,
    /// containing manifest on HEAD and index.
    pub fn new(repo: &mut Repository, pkg: &'a Package) -> Result<Self, failure::Error> {
        let head = PackageManifest::new_head(repo, pkg)?;
        let index = PackageManifest::new_index(pkg)?;
        let manifest_path = repo.rel_path(pkg.manifest_path());
        Ok(PackageManifests {
            pkg,
            head,
            index,
            manifest_path,
        })
    }

    /// Bumps package version in manifest.
    pub fn bump_ver(&mut self, bump: Bump) {
        trace!(
            "Bumping package {} version {} to {}",
            self.pkg.name(),
            self.pkg.version(),
            self.pkg.version().bump(bump)
        );
        self.head.bump_ver(bump);
        self.index.bump_ver(bump);
    }

    /// Updates dependency version in manifest.
    pub fn update_dep(&mut self, name: &str, old_ver: &semver::Version, new_ver: &semver::Version) {
        trace!(
            "Updating package {} dependency {} version {} to {}",
            self.pkg.name(),
            name,
            old_ver,
            new_ver,
        );
        self.head.update_dep(name, old_ver, new_ver);
        self.index.update_dep(name, old_ver, new_ver);
    }

    /// Updates dependency version in manifest.
    pub fn set_dep_path(&mut self, name: &str, path: &Path, ver: &Version) -> &mut Self {
        self.head.set_dep_path(name, path, ver);
        self.index.set_dep_path(name, path, ver);
        self
    }

    /// Creates a head preview path.
    pub fn head_preview_path(&self) -> PathBuf {
        self.manifest_path
            .parent()
            .unwrap()
            .join("Cargo.preview-head.toml")
            .clean_path()
    }

    /// Creates a index preview path.
    pub fn index_preview_path(&self) -> PathBuf {
        self.manifest_path
            .parent()
            .unwrap()
            .join("Cargo.preview-index.toml")
            .clean_path()
    }

    /// Saves TOML manifest to a preview destination.
    pub fn save_preview(&self) -> std::io::Result<()> {
        self.head.save(&self.head_preview_path())?;
        self.index.save(&self.index_preview_path())
    }
}

/// Package manifest.
pub struct PackageManifest<'a> {
    pub pkg: &'a Package,
    pub lines: Vec<String>,
}

impl<'a> PackageManifest<'a> {
    pub fn new_head(repo: &mut Repository, pkg: &'a Package) -> Result<Self, failure::Error> {
        let content = repo.get_contents(&repo.head_tree()?, pkg.manifest_path())?;
        let content = String::from_utf8(content)?;
        let lines: Vec<String> = content.lines().map(|line| line.to_owned()).collect();
        Ok(PackageManifest { pkg, lines })
    }

    pub fn new_index(pkg: &'a Package) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(pkg.manifest_path())?;
        let lines: Vec<String> = content.lines().map(|line| line.to_owned()).collect();
        Ok(PackageManifest { pkg, lines })
    }
}

impl PackageManifestExt for PackageManifest<'_> {
    fn version(&self) -> &semver::Version {
        self.pkg.version()
    }
}

impl ManifestExt for PackageManifest<'_> {
    fn lines(&self) -> &Vec<String> {
        &self.lines
    }

    fn lines_mut(&mut self) -> &mut Vec<String> {
        &mut self.lines
    }
}
