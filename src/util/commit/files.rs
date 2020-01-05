use std::path::{Path, PathBuf};

use crate::{
    git::CommitBuilder,
    util::{self, CleanPath},
};

pub fn restore_manifest<P: AsRef<Path>>(manifest_path: P) -> Result<(), failure::Error> {
    let source_dir = manifest_path.as_ref().parent().unwrap();
    let backup_toml = source_dir.join("Cargo.backup.toml");
    util::rename(backup_toml, manifest_path)?;
    Ok(())
}

pub fn move_index_manifest<P: AsRef<Path>>(manifest_path: P) -> Result<(), failure::Error> {
    let source_dir = manifest_path.as_ref().parent().unwrap();
    let backup_toml = source_dir.join("Cargo.backup.toml");
    let preview_index = source_dir.join("Cargo.preview-index.toml");
    util::rename(&manifest_path, &backup_toml)?;
    util::rename(preview_index, &manifest_path)?;
    Ok(())
}

pub fn add_preview_head<P: AsRef<Path>>(
    commit: &mut CommitBuilder,
    manifest_path: P,
) -> Result<(), failure::Error> {
    add_preview(commit, manifest_path, "head")
}

pub fn add_preview_index<P: AsRef<Path>>(
    commit: &mut CommitBuilder,
    manifest_path: P,
) -> Result<(), failure::Error> {
    add_preview(commit, manifest_path, "index")
}

fn add_preview<P: AsRef<Path>>(
    commit: &mut CommitBuilder,
    manifest_path: P,
    kind: &str,
) -> Result<(), failure::Error> {
    let source_dir = manifest_path.as_ref().parent().unwrap();
    let backup_toml = source_dir.join("Cargo.backup.toml");
    let preview_head = source_dir.join(format!("Cargo.preview-{}.toml", kind));
    util::rename(&manifest_path, &backup_toml)?;
    util::rename(preview_head, &manifest_path)?;
    commit.add_path(&manifest_path)?;
    util::rename(&backup_toml, &manifest_path)?;
    Ok(())
}

pub fn remove_files(
    commit: &mut CommitBuilder,
    files: &Vec<PathBuf>,
    dir: &PathBuf,
) -> Result<(), failure::Error> {
    for file in files {
        // Add removed file to commit
        commit.add_path(&file.clean_path())?;
        // Remove file in cached repo
        let dest = dir.join(&file);
        util::remove_file(dest)?;
    }
    Ok(())
}
pub fn add_files(
    commit: &mut CommitBuilder,
    files: &Vec<PathBuf>,
    dir: &PathBuf,
) -> Result<(), failure::Error> {
    for file in files {
        // Add changed file to commit
        commit.add_path(&file.clean_path())?;
        // Copy changed file to cached repo
        let dest = dir.join(&file);
        util::copy(&file, dest)?;
    }
    Ok(())
}
