mod visitor;

use std::fs::File;
use std::io::Read;

use failure::Error;
use syn::visit::Visit;

use self::visitor::PackageVisitor;

use crate::{
    util::glob_source,
    ws::{Package, Workspace},
};

/// Members collection.
#[derive(Debug)]
pub struct Members {
    /// Macros members.
    pub macros: Vec<String>,
    /// Packages members.
    pub packages: Vec<String>,
}

/// Extracts used workspace dependencies in package root.
pub fn collect_members<'a>(workspace: &'a Workspace<'a>, pkg: &Package) -> Result<Members, Error> {
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
    visitor.dedup();
    let packages = visitor
        .packages
        .into_iter()
        .map(|pkg| pkg.name().to_string())
        .collect();
    Ok(Members {
        macros: visitor.macros,
        packages,
    })
}
