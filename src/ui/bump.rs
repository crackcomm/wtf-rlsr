//! Bumps selections module.

use cargo::core::Package;
use colored::*;

use crate::{
    ui,
    util::{Bump, BUMPS},
};

/// Prompts for a package bump kind.
pub fn prompt(pkg: &Package) -> std::io::Result<Option<&Bump>> {
    let ver = pkg.version();
    let bump_list = bump_choices(ver);
    let selection = ui::select_from_list(
        &format!("Select update kind for {}", pkg.name()),
        &bump_list,
    )?;
    println!();
    Ok(selection.and_then(|selection| BUMPS.get(selection)))
}

fn bump_choices(ver: &semver::Version) -> Vec<String> {
    vec![
        format!("chore v{}", ver),
        format!(
            "patch v{} -> v{}",
            format_bump(ver, &Bump::Patch, true),
            format_bump(ver, &Bump::Patch, false)
        ),
        format!(
            "minor v{} -> v{}",
            format_bump(ver, &Bump::Minor, true),
            format_bump(ver, &Bump::Minor, false)
        ),
        format!(
            "major v{} -> v{}",
            format_bump(ver, &Bump::Major, true),
            format_bump(ver, &Bump::Major, false)
        ),
    ]
}

pub fn format_bump(ver: &semver::Version, bump: &Bump, pre_bump: bool) -> String {
    match bump {
        Bump::Chore => format!("{}", ver),
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
