mod deps;
mod packages;
pub use self::deps::*;
pub use self::packages::*;

use std::path::PathBuf;

use cargo::core::{shell::Shell, Workspace as CargoWorkspace};
use cargo::util::{config::Config as CargoConfig, errors::CargoResult};

use crate::git::Repository;

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
    pub graphs: deps::WorkspaceGraphs,
    pub packages: Packages<'a>,
    pub directory: PathBuf,
}

impl<'a> Workspace<'a> {
    /// Creates a new workspace with cargo and git setup.
    pub fn new(cargo: &'a CargoWorkspace<'a>, repo: &mut Repository) -> Self {
        let graphs = deps::workspace_graph(&cargo);
        let packages = Packages::new(cargo.members(), repo);
        Workspace {
            graphs,
            packages,
            directory: cargo.config().cwd().to_path_buf(),
        }
    }

    /// Returns workspace packages.
    pub fn packages(&self) -> &Packages<'a> {
        &self.packages
    }

    /// Finds a package by name.
    pub fn find_package(&self, name: &str) -> Option<&Package> {
        self.packages.find_by_name(name)
    }
}
