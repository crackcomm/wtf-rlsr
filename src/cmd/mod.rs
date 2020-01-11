pub(crate) mod exec;
pub(crate) mod release;
pub(crate) mod update_paths;

use std::path::PathBuf;

use structopt::StructOpt;

use crate::util::init::setup_opt;

/// Command line application options.
#[derive(Debug, StructOpt)]
#[structopt(name = "wtf-rlrsr", about = "WTF Releaser.")]
pub struct Opt {
    /// Workspace directory.
    #[structopt(parse(from_os_str), default_value = ".")]
    pub directory: PathBuf,

    /// Cache directory.
    #[structopt(parse(from_os_str), short = "c", default_value = "../cache")]
    pub cache_dir: PathBuf,

    /// Git remote.
    #[structopt(short = "r", default_value = "origin")]
    pub remote: String,

    /// Wtf-rlsr subcommand.
    #[structopt(subcommand)]
    pub cmd: Option<Command>,
}

/// Wtf-rlsr subcommand.
#[derive(Debug, StructOpt)]
pub enum Command {
    /// Release command.
    Release(release::Command),

    /// Release test command.
    ReleaseTest,

    /// Update command
    UpdatePaths(update_paths::Command),
}

/// Executes a command.
pub fn execute() -> Result<(), failure::Error> {
    let opt = setup_opt()?;
    match &opt.cmd {
        Some(cmd) => exec::execute(&opt, cmd),
        None => {
            Opt::clap().print_help()?;
            Ok(())
        }
    }
}
