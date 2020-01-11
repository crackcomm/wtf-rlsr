use std::ffi::OsStr;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::{util::glob_source, ws::Package};

/// Diff2Html command options.
pub struct Options {
    /// Diff2Html executable path.
    path: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            path: which::which("diff2html").unwrap(),
        }
    }
}

/// Spawns `git diff` and `diff2html` for specified package root.
pub fn spawn_for_pkgs(pkgs: &[&Package], opts: &Options) -> Result<(), failure::Error> {
    let files: Vec<_> = pkgs
        .iter()
        .map(|pkg| {
            let mut source = glob_source(pkg);
            source.push(pkg.manifest_path().to_path_buf());
            source
        })
        .flatten()
        .collect();
    if files.len() > 0 {
        trace!("Diff for files: {:?}", files);
        spawn_for_paths(files.iter(), opts)
    } else {
        Ok(())
    }
}

/// Spawns `git diff` and `diff2html` for specified paths.
pub fn spawn_for_paths<I: Iterator<Item = S>, S: AsRef<OsStr>>(
    paths: I,
    opts: &Options,
) -> Result<(), failure::Error> {
    Command::new("git").args(&["add", "-A"]).output()?;
    let mut command = Command::new("git");
    command.args(&["diff", "HEAD"]);
    for path in paths {
        command.arg(path);
    }
    let output = command.output()?;
    if !output.status.success() {
        panic!("Error running git diff.");
    }
    let diff = String::from_utf8(output.stdout)?;
    Command::new("git").args(&["reset"]).output()?;
    let mut child = Command::new(&opts.path)
        .arg("-i")
        .arg("stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(diff.as_bytes())?;
        stdin.flush()?;
    }
    child.wait()?;
    Ok(())
}
