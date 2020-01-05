//! Bumps selections module.

use cargo::core::Package;
use colored::*;

use crate::{
    ui,
    util::{Bump, Update, UPDATES},
};

/// Prompts for a package update kind.
pub fn prompt(pkg: &Package) -> std::io::Result<Option<&Update>> {
    let ver = pkg.version();
    let selection = ui::select_from_list(
        &format!("Select update kind for {}", pkg.name()),
        &update_choices(ver),
    )?;
    println!();
    Ok(selection.and_then(|selection| UPDATES.get(selection)))
}

fn update_choices(ver: &semver::Version) -> Vec<String> {
    vec![
        format!("docs v{}", ver),
        format!("chore v{}", ver),
        format!(
            "patch v{} -> v{}",
            format_bump(ver, Bump::Patch, true),
            format_bump(ver, Bump::Patch, false)
        ),
        format!(
            "minor v{} -> v{}",
            format_bump(ver, Bump::Minor, true),
            format_bump(ver, Bump::Minor, false)
        ),
        format!(
            "major v{} -> v{}",
            format_bump(ver, Bump::Major, true),
            format_bump(ver, Bump::Major, false)
        ),
    ]
}

pub fn format_bump(ver: &semver::Version, bump: Bump, pre_bump: bool) -> String {
    match bump {
        Bump::Patch => format!(
            "{}.{}.{}",
            ver.major,
            ver.minor,
            if pre_bump {
                ver.patch.to_string().green()
            } else {
                (ver.patch + 1).to_string().yellow()
            }
        ),
        Bump::Minor => format!(
            "{}.{}.{}",
            ver.major,
            if pre_bump {
                ver.minor.to_string().green()
            } else {
                (ver.minor + 1).to_string().yellow()
            },
            ver.patch
        ),
        Bump::Major => format!(
            "{}.{}.{}",
            if pre_bump {
                ver.major.to_string().green()
            } else {
                (ver.major + 1).to_string().yellow()
            },
            ver.minor,
            ver.patch
        ),
    }
}
