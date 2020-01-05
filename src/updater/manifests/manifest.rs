//! Manifest extension traits.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use cargo::core::Package;
use semver::Version;

use crate::util::{Bump, BumpExt};

/// Workspace manifest extension trait.
pub trait WorkspaceManifestExt: ManifestExt {
    /// Inserts new git replace for a package.
    fn git_replace(&mut self, package: &Package, remote_url: &str, rev: &str) {
        self.lines_mut().push(format!(
            "\"{}:{}\" = {{ git = \"{}\", rev = \"{}\" }}",
            package.name(),
            package.version(),
            remote_url,
            rev
        ));
    }

    /// Bumps package version in manifest.
    fn bump_ver(&mut self, package: &Package, bump: &Bump) {
        change_replace(self.lines_mut(), package, bump);
    }
}

/// Package manifest extension trait.
pub trait PackageManifestExt: ManifestExt {
    /// Returns current package version.
    fn version(&self) -> &Version;

    /// Bumps package version in manifest.
    fn bump_ver(&mut self, bump: &Bump) {
        let ver = self.version().clone();
        let new = ver.bump(bump);
        change_first_ver(self.lines_mut(), &ver, &new);
    }

    /// Sets package version in manifest.
    fn update_ver(&mut self, new: &Version) {
        let ver = self.version().clone();
        change_first_ver(self.lines_mut(), &ver, new);
    }

    /// Updates dependency version in manifest.
    fn update_dep(&mut self, name: &str, old: &Version, new: &Version) {
        change_dep_ver(self.lines_mut(), name, old, new);
    }

    /// Updates dependency version in manifest.
    fn set_dep_path(&mut self, name: &str, path: &Path, ver: &Version) {
        set_dep_path(self.lines_mut(), name, path, ver);
    }
}

/// Manifest extension trait.
pub trait ManifestExt {
    /// Returns lines of a manifest.
    fn lines(&self) -> &Vec<String>;

    /// Returns mutable lines of a manifest.
    fn lines_mut(&mut self) -> &mut Vec<String>;

    /// Returns content of a manifest.
    fn content(&self) -> String {
        self.lines().join("\n")
    }

    /// Saves TOML manifest to a destination.
    fn save<P: AsRef<Path>>(&self, dest: P) -> std::io::Result<()> {
        let mut output = File::create(&dest)?;
        for line in self.lines() {
            output.write(line.as_bytes())?;
            output.write(&['\n' as u8])?;
        }
        Ok(())
    }
}

/// Changes package version in `replace` section.
fn change_replace(lines: &mut Vec<String>, package: &Package, bump: &Bump) {
    let old = format!(":{}\"", package.version());
    let new = format!(":{}\"", package.version().bump(bump));
    let find_name = format!("\"{}:{}\"", package.name(), package.version());
    for line in lines {
        if line.starts_with(&find_name) {
            *line = line.replace(&old, &new);
        }
    }
}

/// Replaces first version in toml file.
fn change_first_ver(lines: &mut Vec<String>, old: &Version, new: &Version) {
    let ver_line = format!("version = \"{}\"", old);
    for line in lines {
        if *line == ver_line {
            *line = format!("version = \"{}\"", new);
            break;
        }
    }
}

/// Replaces dependency version in toml file.
/// Works only for either `dependency = "x.x.x"`
/// and `dependency = { version = "x.x.x" }`.
fn change_dep_ver(lines: &mut Vec<String>, name: &str, old: &Version, new: &Version) {
    let old = format!("\"{}\"", old);
    let new = format!("\"{}\"", new);
    let find_name = format!("{} ", name);
    for line in lines {
        if line.starts_with(&find_name) {
            *line = line.replace(&old, &new);
        }
    }
}

/// Sets dependency path in toml file.
/// Works only for `dependency = "x.x.x"`.
fn set_dep_path(lines: &mut Vec<String>, name: &str, path: &Path, ver: &Version) {
    let find_name = format!("{} = \"", name);
    for line in lines {
        if line.starts_with(&find_name) {
            *line = format!("{} = {{ version = \"{}\", path = {:?} }}", name, ver, path);
        }
    }
}
