use std::path::{Path, PathBuf};

use crate::git::Repository;
use crate::util::{Bump, CleanPath};
use crate::ws::Package;

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
    pub fn new(repo: &mut Repository) -> Result<Self, failure::Error> {
        let path = repo.workdir().unwrap().join("Cargo.toml");
        let head = WorkspaceManifest::new_head(repo)?;
        let index = WorkspaceManifest::new_index(&path)?;
        Ok(WorkspaceManifests {
            head,
            index,
            manifest_path: path,
        })
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

    /// Bumps package version in manifest.
    pub fn bump_replace_ver(&mut self, package: &Package, bump: Bump) {
        self.head.bump_replace_ver(package, bump);
        self.index.bump_replace_ver(package, bump);
    }

    /// Bumps package version in manifest.
    pub fn set_replace(&mut self, package: &Package, path: &Path) {
        self.head.set_replace(package, path);
        self.index.set_replace(package, path);
    }
}

/// Package manifest.
pub struct WorkspaceManifest {
    pub lines: Vec<String>,
}

impl WorkspaceManifest {
    pub fn new_head(repo: &mut Repository) -> Result<Self, git2::Error> {
        let content = repo.get_contents(&repo.head_tree()?, Path::new("Cargo.toml"))?;
        let content = String::from_utf8(content).unwrap();
        let lines: Vec<String> = content.lines().map(|line| line.to_owned()).collect();
        Ok(WorkspaceManifest { lines })
    }

    pub fn new_index<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<String> = content.lines().map(|line| line.to_owned()).collect();
        Ok(WorkspaceManifest { lines })
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
