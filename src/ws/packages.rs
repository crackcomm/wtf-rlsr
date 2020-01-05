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
        for member in &members {
            // pre-load git diff cache
            repo.diff(member)?;
        }
        let inner = members
            .clone()
            .into_iter()
            .map(|pkg| Package::new(pkg, repo, &members))
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
pub struct Package<'a> {
    pub diff: Option<Diff>,
    pub dependencies: Vec<Dependency>,
    inner: &'a CargoPackage,
    manifest_path: PathBuf,
}

impl<'a> Package<'a> {
    /// Updates package info with repository.
    pub fn new(
        pkg: &'a CargoPackage,
        repo: &mut Repository,
        members: &Vec<&'a CargoPackage>,
    ) -> Result<Self, git2::Error> {
        let diff = Some(repo.diff(pkg)?.clone());
        let dependencies = pkg
            .dependencies()
            .into_iter()
            .map(|inner| {
                let name = inner.package_name().as_str();
                let is_member = members.iter().any(|pkg| pkg.name().as_str() == name);
                let diff = repo.cached_diff(name).map(|diff| diff.clone());
                Dependency {
                    diff,
                    inner: inner.clone(),
                    is_member,
                }
            })
            .collect();
        let manifest_path = repo.rel_path(pkg.manifest_path());
        Ok(Package {
            inner: pkg,
            diff,
            dependencies,
            manifest_path,
        })
    }

    /// Returns true if package diff is not empty.
    pub fn is_changed(&self) -> bool {
        match &self.diff {
            Some(diff) => !diff.is_empty(),
            None => false,
        }
    }

    /// Returns true if package has changed deps.
    pub fn has_changed_deps(&self) -> bool {
        self.dependencies.iter().any(|dep| dep.is_changed())
    }

    /// Returns package dependencies.
    pub fn dependencies(&self) -> &Vec<Dependency> {
        &self.dependencies
    }

    /// Returns package manifest path.
    pub fn manifest_path(&self) -> &Path {
        self.manifest_path.as_ref()
    }
}

impl<'a> Deref for Package<'a> {
    type Target = CargoPackage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Cargo package dependency wrapper.
pub struct Dependency {
    /// Inner cargo package dependency.
    inner: CargoDependency,
    /// Workspace member package.
    is_member: bool,
    /// Dependency diff./
    diff: Option<Diff>,
}

impl Dependency {
    /// Returns dependency diff.
    pub fn diff(&self) -> Option<&Diff> {
        self.diff.as_ref()
    }

    /// Returns true if dependency diff is not empty.
    pub fn is_changed(&self) -> bool {
        match &self.diff {
            Some(diff) => !diff.is_empty(),
            None => false,
        }
    }

    /// Returns true if dependency is a member of workspace.
    pub fn is_member(&self) -> bool {
        self.is_member
    }
}

impl Deref for Dependency {
    type Target = CargoDependency;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<CargoDependency> for Dependency {
    fn from(inner: CargoDependency) -> Self {
        Dependency {
            inner,
            is_member: false,
            diff: None,
        }
    }
}
