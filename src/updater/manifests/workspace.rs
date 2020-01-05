use std::path::{Path, PathBuf};

use cargo::core::Package;

use crate::git::Repository;
use crate::util::Bump;

use super::{ManifestExt, WorkspaceManifestExt};

/// Package manifests on HEAD and index.
pub struct WorkspaceManifests {
    pub head: WorkspaceManifest,
    pub index: WorkspaceManifest,
    pub manifest_path: PathBuf,
}

impl WorkspaceManifests {
    /// Creates new package manifests structure,
    /// containing manifest on HEAD and index.
    pub fn new(repo: &mut Repository) -> Self {
        let path = repo.workdir().unwrap().join("Cargo.toml");
        let head = WorkspaceManifest::new_head(repo);
        let index = WorkspaceManifest::new_index(&path);
        WorkspaceManifests {
            head,
            index,
            manifest_path: path,
        }
    }

    /// Saves TOML manifest to a preview destination.
    pub fn save_preview(&self) -> std::io::Result<()> {
        let source_path = self.manifest_path.parent().unwrap();
        self.head
            .save(&source_path.join("Cargo.preview-head.toml"))?;
        self.index
            .save(&source_path.join("Cargo.preview-index.toml"))
    }

    /// Bumps package version in manifest.
    pub fn bump_ver(&mut self, package: &Package, bump: &Bump) {
        self.head.bump_ver(package, bump);
        self.index.bump_ver(package, bump);
    }
}

/// Package manifest.
pub struct WorkspaceManifest {
    pub lines: Vec<String>,
}

impl WorkspaceManifest {
    pub fn new_head(repo: &mut Repository) -> Self {
        let content = repo.get_contents(&repo.head_tree(), Path::new("Cargo.toml"));
        let content = String::from_utf8(content).unwrap();
        let lines: Vec<String> = content.lines().map(|line| line.to_owned()).collect();
        WorkspaceManifest { lines }
    }

    pub fn new_index<P: AsRef<Path>>(path: P) -> Self {
        let content = std::fs::read_to_string(path).unwrap();
        let lines: Vec<String> = content.lines().map(|line| line.to_owned()).collect();
        WorkspaceManifest { lines }
    }
}

impl ManifestExt for WorkspaceManifest {
    fn lines(&self) -> &Vec<String> {
        &self.lines
    }

    fn lines_mut(&mut self) -> &mut Vec<String> {
        &mut self.lines
    }
}

impl WorkspaceManifestExt for WorkspaceManifest {}
