//! Package source code visitor.

use syn::visit::{self, Visit};
use syn::{Ident, ItemExternCrate, Macro, PathSegment, UsePath};

use crate::ws::{Package, Workspace};

/// Package source code visitor.
pub struct PackageVisitor<'a> {
    workspace: &'a Workspace<'a>,
    pub macros: Vec<String>,
    pub packages: Vec<&'a Package<'a>>,
}

impl<'a> PackageVisitor<'a> {
    pub fn new(workspace: &'a Workspace<'a>) -> Self {
        PackageVisitor {
            workspace,
            macros: Vec::new(),
            packages: Vec::new(),
        }
    }

    pub fn dedup(&mut self) {
        self.macros.sort();
        self.macros.dedup();
        self.packages
            .sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));
        self.packages
            .dedup_by(|a, b| a.name().as_str() == b.name().as_str());
    }

    fn register_pkg(&mut self, ident: &Ident) {
        let name = ident.to_string().replace("_", "-");
        if let Some(pkg) = self.workspace.find_package(&name) {
            self.packages.push(pkg);
        }
    }

    fn register_macro(&mut self, ident: &Ident) {
        // let name = ident.to_string().replace("_", "-");
        // if let Some(pkg) = self.workspace.find_package(&name) {
        //     self.packages.push(pkg);
        // }
    }
}

impl<'a, 'ast> Visit<'ast> for PackageVisitor<'a> {
    fn visit_item_extern_crate(&mut self, node: &'ast ItemExternCrate) {
        self.register_pkg(&node.ident);
        visit::visit_item_extern_crate(self, node);
    }

    fn visit_use_path(&mut self, node: &'ast UsePath) {
        self.register_pkg(&node.ident);
        visit::visit_use_path(self, node);
    }

    fn visit_path_segment(&mut self, node: &'ast PathSegment) {
        self.register_pkg(&node.ident);
        visit::visit_path_segment(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        // if node.path.segments.first().unwrap().ident == "macro_rules" {
        //     trace!(
        //         "Macro: {:?}",
        //         // node.path.segments.first().unwrap().ident.to_string()
        //         node
        //     );
        // }
        self.register_macro(&node.path.segments.first().unwrap().ident);
        visit::visit_macro(self, node);
    }
}
