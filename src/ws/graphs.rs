//! Workspace dependencies.

use cargo::{core::Workspace as CargoWorkspace, util::graph::Graph};

/// Dependencies graph.
pub type DepGraph = Graph<String, Vec<String>>;

/// Workspace dependencies graph.
pub struct WorkspaceGraphs {
    pub dependants: DepGraph,
    pub dependencies: DepGraph,
}

/// Creates a graph of dependencies for a workspace.
pub(super) fn workspace_graph(workspace: &CargoWorkspace) -> WorkspaceGraphs {
    let mut dependencies = DepGraph::new();
    let mut dependants = DepGraph::new();
    for member in workspace.members() {
        dependants.add(member.name().to_string());
        dependencies.add(member.name().to_string());
    }
    for member in workspace.members() {
        let pkg_name = member.name().to_string();
        for dep in member.dependencies() {
            let dep_name = dep.package_name().to_string();
            let is_member = workspace
                .members()
                .any(|pkg| pkg.name().as_str() == dep_name);
            if is_member {
                dependencies.link(pkg_name.clone(), dep_name.clone());
                dependants.link(dep_name.clone(), pkg_name.clone());
            }
        }
    }
    WorkspaceGraphs {
        dependants,
        dependencies,
    }
}
