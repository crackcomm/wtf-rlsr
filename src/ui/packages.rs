use colored::Colorize;
use dialoguer::Checkboxes;

use crate::{
    ui::select_from_list,
    util::Bump,
    ws::{Package, Workspace},
};

use super::default_theme;

/// Selects changed package from workspace.
pub fn select_changed<'a, 'i>(
    workspace: &'a Workspace<'i>,
) -> std::io::Result<Option<&'a Package<'i>>> {
    let packages: Vec<_> = workspace
        .packages
        .iter()
        .filter(|pkg| pkg.is_changed())
        .collect();
    let names: Vec<String> = packages
        .iter()
        .map(|pkg| {
            let name = pkg.name().as_str();
            let diff = pkg.diff.as_ref().unwrap();
            format!(
                "{} ({}, {}, {})",
                name,
                format!("{} files changed", diff.files_changed).bright_blue(),
                format!("{} insertions", diff.insertions).green(),
                format!("{} deletions", diff.deletions).red()
            )
        })
        .collect();
    Ok(select_from_list("Pick a package to commit", &names)?
        .and_then(|selection| packages.get(selection))
        .map(|s| *s))
}

/// Prompts to select dependencies to update.
pub fn select_update_deps<'a>(
    dependants: &Vec<&'a Package<'a>>,
    bump: &Bump,
) -> (Vec<&'a Package<'a>>, Vec<&'a Package<'a>>) {
    let (checkboxes, defaults_update) = dependant_choices(&dependants, bump);
    let selections = Checkboxes::with_theme(&default_theme())
        .with_prompt("Select dependencies to update in tree")
        .items(&checkboxes)
        .defaults(&defaults_update)
        .interact()
        .unwrap();
    let update_packages: Vec<_> = selections
        .clone()
        .into_iter()
        .map(|index| *dependants.get(index).unwrap())
        .collect();
    let packages_to_git: Vec<_> = dependants
        .iter()
        .enumerate()
        .filter_map(|(index, dep)| {
            if selections.contains(&index) {
                None
            } else {
                Some(*dep)
            }
        })
        .collect();
    (update_packages, packages_to_git)
}

fn dependant_choices<'a>(
    dependants: &Vec<&'a Package<'a>>,
    bump: &Bump,
) -> (Vec<String>, Vec<bool>) {
    let (mut checkboxes, mut defaults_update) = (Vec::new(), Vec::new());
    let default_update = match bump {
        Bump::Chore => panic!("Chore should not update dependants."),
        Bump::Patch | Bump::Minor => true,
        Bump::Major => false,
    };
    for pkg in dependants {
        let name = pkg.name().as_str();
        if !pkg.is_changed() {
            checkboxes.push(name.yellow().to_string());
            defaults_update.push(default_update);
        } else {
            let changed_deps: Vec<_> = pkg
                .dependencies
                .iter()
                .filter(|dep| dep.is_changed())
                .map(|dep| dep.package_name().to_string())
                .collect();
            if changed_deps.len() > 0 {
                checkboxes.push(format!(
                    "{} (changed: {})",
                    name.red(),
                    changed_deps.join(", ")
                ));
            } else {
                checkboxes.push(name.red().to_string());
            }
            defaults_update.push(default_update);
        }
    }
    (checkboxes, defaults_update)
}
