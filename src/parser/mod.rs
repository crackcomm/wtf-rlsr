mod visitor;

use std::fs::File;
use std::io::Read;

use cargo::core::Package;
use failure::Error;
use syn::visit::Visit;

use self::visitor::PackageVisitor;

use crate::{util::glob_source, ws::Workspace};

/// Extracts used workspace dependencies in package root.
pub fn collect_workspace_deps(workspace: &Workspace, pkg: &Package) -> Result<Vec<String>, Error> {
    let files = glob_source(pkg);
    let mut visitor = PackageVisitor::new(workspace);
    for filename in files {
        let mut file = File::open(&filename).expect("Unable to open file");

        let mut src = String::new();
        file.read_to_string(&mut src).expect("Unable to read file");

        let syntax = syn::parse_file(&src).expect("Unable to parse file");
        // println!("syntax={:?}", syntax);
        visitor.visit_file(&syntax);
    }
    visitor.dedup_packages();
    Ok(visitor.packages)
}
