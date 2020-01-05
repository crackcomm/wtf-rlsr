//! Package source code visitor.

use syn::visit::{self, Visit};
use syn::{Ident, ItemExternCrate, Macro, PathSegment, UsePath};

use crate::ws::Workspace;

/// Package source code visitor.
pub(crate) struct PackageVisitor<'a, 'cfg> {
    workspace: &'a Workspace<'cfg>,
    pub packages: Vec<String>,
}

impl<'a, 'cfg> PackageVisitor<'a, 'cfg> {
    pub fn new(workspace: &'a Workspace<'cfg>) -> Self {
        PackageVisitor {
            workspace,
            packages: Vec::new(),
        }
    }

    pub fn dedup_packages(&mut self) {
        self.packages.sort();
        self.packages.dedup();
    }

    fn register_pkg(&mut self, ident: &Ident) {
        let name = ident.to_string().replace("_", "-");
        if let Some(pkg) = self.workspace.find_package(&name) {
            self.packages.push(pkg.name().to_string());
        }
    }
}

impl<'a, 'cfg, 'ast> Visit<'ast> for PackageVisitor<'a, 'cfg> {
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
        trace!(
            "Macro: {:?}",
            node.path.segments.first().unwrap().ident.to_string()
        );
        visit::visit_macro(self, node);
    }
}
