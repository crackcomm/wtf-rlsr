mod graphs;
mod packages;
pub use self::graphs::*;
pub use self::packages::*;

use std::path::{Path, PathBuf};

use failure::ResultExt;

use cargo::core::{shell::Shell, Workspace as CargoWorkspace};
use cargo::util::{config::Config as CargoConfig, errors::CargoResult};

use crate::{
    git::Repository,
    util::{Bump, BumpExt},
};

/// Creates default workspace configuration.
pub fn cargo_config(dir: PathBuf) -> CargoConfig {
    let shell = Shell::new();
    let homedir = dirs::home_dir().unwrap();
    CargoConfig::new(shell, dir, homedir.clone())
}

/// Creates a cargo workspace from directory.
pub fn cargo_workspace<'a>(config: &'a CargoConfig) -> CargoResult<CargoWorkspace<'a>> {
    CargoWorkspace::new(&config.cwd().join("Cargo.toml"), &config)
}

/// Cargo workspace wrapper.
pub struct Workspace<'a> {
    pub graphs: WorkspaceGraphs,
    pub packages: Packages<'a>,
    pub directory: PathBuf,
    // serialized package info
    root_package: SerializedPackage,
    manifest_path: PathBuf,
}

impl<'a> Workspace<'a> {
    /// Creates a new workspace with cargo and git setup.
    pub fn new(
        cargo: &'a CargoWorkspace<'a>,
        repo: &mut Repository,
    ) -> Result<Self, failure::Error> {
        let graphs = workspace_graph(&cargo);
        let packages = Packages::new(cargo.members(), repo)?;
        let directory = cargo.config().cwd().to_path_buf();
        let root_package = SerializedPackage::open(&directory.join("package.json"))?;
        let manifest_path = repo.rel_path(&directory.join("Cargo.toml"));
        trace!(
            "Workspace {} version: {}",
            root_package.name,
            root_package.version
        );
        Ok(Workspace {
            graphs,
            packages,
            directory,
            root_package,
            manifest_path,
        })
    }

    /// Returns workspace package version.
    pub fn version(&self) -> &semver::Version {
        &self.root_package.version
    }

    /// Returns workspace package name.
    pub fn name(&self) -> &str {
        &self.root_package.name
    }

    /// Returns workspace packages.
    pub fn packages(&self) -> &Packages<'a> {
        &self.packages
    }

    /// Finds a package by name.
    pub fn find_package(&self, name: &str) -> Option<&Package> {
        self.packages.find_by_name(name)
    }

    /// Bumps workspace package version in file ONLY.
    pub fn bump(&self, bump: Bump) -> Result<(), failure::Error> {
        self.root_package
            .bump_and_save(bump, &self.directory.join("package.json"))
    }

    /// Returns clean path to workspace manifest.
    pub fn manifest_path(&self) -> &Path {
        self.manifest_path.as_ref()
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct SerializedPackage {
    name: String,
    version: semver::Version,
}

impl SerializedPackage {
    fn open<P: AsRef<Path>>(path: P) -> Result<Self, failure::Error> {
        let content = std::fs::read_to_string(path)
            .with_context(|e| format!("error reading package.json: {}", e))?;
        let package = serde_json::from_str(&content)?;
        Ok(package)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), failure::Error> {
        let content = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn bump_and_save<P: AsRef<Path>>(&self, bump: Bump, path: P) -> Result<(), failure::Error> {
        let mut manifest = self.clone();
        manifest.version = manifest.version.bump(bump);
        manifest.save(path)
    }
}
