//! Workspace publishing tools.

use cargo::{
    core::{Package as CargoPackage, Workspace as CargoWorkspace},
    ops::{publish, PublishOpts},
    util::Config as CargoConfig,
};
use failure::Error;

use crate::{
    util::Logger,
    ws::{Package, Workspace},
};

/// Publishes a package.
pub fn publish_pkg(pkg: &CargoPackage, config: &CargoConfig, dry_run: bool) -> Result<bool, Error> {
    let pub_opts = PublishOpts {
        dry_run,
        config: config,
        token: Some(std::env::var("CCI")?),
        index: None,
        verify: false,
        allow_dirty: true,
        jobs: Some(12),
        target: None,
        registry: None,
        features: vec![],
        all_features: true,
        no_default_features: false,
    };
    let pkg_workspace = match CargoWorkspace::new(pkg.manifest_path(), config) {
        Ok(workspace) => workspace,
        Err(err) => {
            println!(
                "Cannot open package {} workspace error: {:?}.",
                pkg.name(),
                err
            );
            return Ok(false);
        }
    };
    pkg_workspace.status("Publishing", pkg.name());
    if let Err(err) = publish(&pkg_workspace, &pub_opts) {
        let e = err.to_string();
        if e.contains("already uploaded") {
            pkg_workspace.status(
                "Publishing",
                format!("Package {} already published.", pkg.name()),
            );
            return Ok(true);
        }
        println!(
            "Package {} publish returned with error: {:?}.",
            pkg.name(),
            err
        );
        Ok(false)
    } else {
        Ok(true)
    }
}

/// Publishes packages with all dependants.
pub fn publish_pkg_deep(
    package: &Package<'_>,
    workspace: &Workspace<'_>,
    config: &CargoConfig,
    update_packages: &Vec<&Package<'_>>,
    published: &mut Vec<String>,
    dry_run: bool,
) -> Result<bool, failure::Error> {
    let name = package.name().to_string();
    if !publish_pkg(&package, &config, dry_run)? {
        return Ok(false);
    }
    for (name, _) in workspace.graphs.dependants.edges(&name) {
        let package = workspace.find_package(&name).unwrap();
        let name = package.name().to_string();
        if !published.contains(&name) {
            published.push(name.clone());
            trace!("Publish Dependency: {:?}", name);
            if !publish_pkg_deep(
                package,
                workspace,
                config,
                update_packages,
                published,
                dry_run,
            )? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
