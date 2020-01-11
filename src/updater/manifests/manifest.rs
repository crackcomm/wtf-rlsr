//! Manifest extension traits.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use semver::Version;

use crate::util::{Bump, BumpExt, CleanPath};
use crate::ws::{Dependency, Package};

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
    fn bump_replace_ver(&mut self, package: &Package, bump: Bump) {
        change_replace(self.lines_mut(), package, bump);
    }

    /// Bumps package version in manifest.
    fn set_replace(&mut self, package: &Package, path: &Path) {
        set_replace(self.lines_mut(), package, path);
    }
}

/// Package manifest extension trait.
pub trait PackageManifestExt: ManifestExt {
    /// Returns current package version.
    fn version(&self) -> &Version;

    /// Bumps package version in manifest.
    fn bump_ver(&mut self, bump: Bump) {
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

    /// Updates dependency version in manifest.
    fn set_dep_force(&mut self, dep: &Dependency, pkg: &Package, path: Option<&Path>) {
        set_dep_force(self.lines_mut(), dep, pkg, path);
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
fn change_replace(lines: &mut Vec<String>, package: &Package, bump: Bump) {
    let old = format!(":{}\"", package.version());
    let new = format!(":{}\"", package.version().bump(bump));
    let find_name = format!("\"{}:{}\"", package.name(), package.version());
    for line in lines {
        if line.starts_with(&find_name) {
            *line = line.replace(&old, &new);
        }
    }
}

/// Sets package version and path in `replace` section.
fn set_replace(lines: &mut Vec<String>, package: &Package, path: &Path) {
    let new_line = format!(
        "\"{}:{}\" = {{ path = {:?} }}",
        package.name(),
        package.version(),
        path.clean_path()
    );
    let find_name = format!("\"{}:", package.name());
    for line in lines.iter_mut() {
        if line.starts_with(&find_name) {
            *line = new_line;
            return;
        }
    }
    // if loop didnt return, insert new line
    lines.push(new_line);
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
    let old = format!("{:?}", old.to_string());
    let new = format!("{:?}", new.to_string());
    let find_name = format!("{} ", name);
    for line in lines {
        if line.starts_with(&find_name) {
            *line = line.replace(&old, &new);
        }
    }
}

/// Replaces dependency version in toml file.
/// Works only for either `dependency = "x.x.x"`
/// and `dependency = { version = "x.x.x" }`.
fn set_dep_force(lines: &mut Vec<String>, dep: &Dependency, pkg: &Package, path: Option<&Path>) {
    let find_name = format!("{} ", dep.package_name().as_str());
    for line in lines {
        if line.starts_with(&find_name) {
            *line = format!(
                "{} = {}",
                dep.package_name().as_str(),
                format_pkg_dep(dep, pkg, path),
            );
        }
    }
}

fn format_pkg_dep(dep: &Dependency, pkg: &Package, path: Option<&Path>) -> String {
    if path.is_none()
        && !dep.is_optional()
        && dep.uses_default_features()
        && dep.features().len() == 0
    {
        format!("{:?}", pkg.version().to_string())
    } else {
        let mut lines = Vec::new();
        lines.push(format!("version = {:?}", pkg.version().to_string()));
        if let Some(path) = path {
            lines.push(format!("path = {:?}", path.clean_path()));
        }
        if dep.is_optional() {
            lines.push("optional = true".to_owned());
        }
        if !dep.uses_default_features() {
            lines.push("default-features = false".to_owned());
        }
        if dep.features().len() > 0 {
            lines.push(format!(
                "features = [{}]",
                dep.features()
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }

        format!("{{ {} }}", lines.join(", "))
    }
}

/// Sets dependency path in toml file.
/// Works only for `dependency = "x.x.x"`.
fn set_dep_path(lines: &mut Vec<String>, name: &str, path: &Path, ver: &Version) {
    let find_name = format!("{} = \"", name);
    for line in lines {
        if line.starts_with(&find_name) {
            *line = format!(
                "{} = {{ version = \"{}\", path = {:?} }}",
                name,
                ver,
                path.clean_path()
            );
        }
    }
}
