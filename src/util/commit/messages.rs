use crate::util::Update;
use crate::ws::Package;

/// Creates a commit message.
pub fn message(
    package: &Package,
    update: &Update,
    header: &str,
    extra: Option<&Vec<String>>,
    is_dep: bool,
) -> String {
    let dep_commit = if is_dep {
        dep_update(package, update, header)
    } else {
        pkg_update(package, update, header)
    };
    let mut commit_lines = vec![dep_commit];
    if let Some(extra) = extra {
        commit_lines.push("".to_owned());
        commit_lines.extend(extra.clone());
    }
    for line in &commit_lines {
        trace!("Dependency commit message: {}", line);
    }
    commit_lines.join("\n")
}

/// Creates a package update commit message.
fn pkg_update(package: &Package, update: &Update, header: &str) -> String {
    format!(
        "{}({}): {} of {} {} ({})",
        update.commit_type(),
        package.name().to_string().replacen("-", "/", 1),
        update.commit_description(),
        package.name(),
        update.transition(package.version()),
        header
    )
}

/// Creates a dependency update commit message.
fn dep_update(package: &Package, update: &Update, header: &str) -> String {
    format!(
        "{}(*): {} of {} {} ({})",
        update.commit_type(),
        update.commit_description(),
        package.name(),
        update.transition(package.version()),
        header
    )
}
