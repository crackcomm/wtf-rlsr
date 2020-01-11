//! Cargo workspace packages wrappers.

use std::ops::Deref;
use std::path::{Path, PathBuf};

use cargo::core::Dependency as CargoDependency;
use cargo::core::Package as CargoPackage;

use crate::git::{Diff, Repository};

/// Cargo packages wrapper.
pub struct Packages<'a> {
    inner: Vec<Package<'a>>,
}

impl<'a> Packages<'a> {
    pub fn new<I>(iter: I, repo: &mut Repository) -> Result<Self, git2::Error>
    where
        I: Iterator<Item = &'a CargoPackage>,
    {
        let members: Vec<_> = iter.collect();
        // pre-load git diff cache
        for member in &members {
            repo.diff(member)?;
        }
        let inner = members
            .clone()
            .into_iter()
            .map(|pkg| Package::new(pkg, repo))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Packages { inner })
    }

    /// Counts changed packages.
    pub fn changed(&self) -> usize {
        self.inner.iter().filter(|pkg| pkg.is_changed()).count()
    }

    /// Finds package by name.
    pub fn find_by_name(&self, name: &str) -> Option<&Package<'a>> {
        self.inner.iter().find(|pkg| pkg.name().as_str() == name)
    }
}

impl<'a> Deref for Packages<'a> {
    type Target = Vec<Package<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Cargo package wrapper.
#[derive(Debug, PartialEq, Eq)]
pub struct Package<'a> {
    pub diff: Option<Diff>,
    pub dependencies: Vec<Dependency>,
    inner: &'a CargoPackage,
    directory: PathBuf,
    manifest_path: PathBuf,
}

impl<'a> Package<'a> {
    /// Updates package info with repository.
    pub fn new(pkg: &'a CargoPackage, repo: &mut Repository) -> Result<Self, git2::Error> {
        let dependencies = pkg
            .dependencies()
            .into_iter()
            .map(|inner| {
                let name = inner.package_name().as_str();
                let is_changed = repo
                    .cached_diff(name)
                    .map(|diff| !diff.is_empty())
                    .unwrap_or(false);
                Dependency {
                    inner: inner.clone(),
                    is_changed,
                }
            })
            .collect();
        Ok(Package {
            diff: Some(repo.diff(pkg)?.clone()),
            manifest_path: repo.rel_path(pkg.manifest_path()),
            inner: pkg,
            dependencies,
            directory: repo.rel_path(pkg.manifest_path().parent().unwrap()),
        })
    }

    /// Returns true if package diff is not empty.
    pub fn is_changed(&self) -> bool {
        match &self.diff {
            Some(diff) => !diff.is_empty(),
            None => false,
        }
    }

    /// Returns package dependencies.
    pub fn dependencies(&self) -> &Vec<Dependency> {
        &self.dependencies
    }

    /// Returns package manifest path.
    pub fn manifest_path(&self) -> &Path {
        self.manifest_path.as_ref()
    }

    /// Returns package directory.
    pub fn directory(&self) -> &Path {
        self.directory.as_ref()
    }
}

impl<'a> Deref for Package<'a> {
    type Target = CargoPackage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Cargo package dependency wrapper.
#[derive(Debug, PartialEq, Eq)]
pub struct Dependency {
    /// Inner cargo package dependency.
    inner: CargoDependency,
    /// True if dependency is changed.
    is_changed: bool,
}

impl Dependency {
    /// Returns true if dependency diff is not empty.
    pub fn is_changed(&self) -> bool {
        self.is_changed
    }
}

impl Deref for Dependency {
    type Target = CargoDependency;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
